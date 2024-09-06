use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{debug, error, info};
use monitoring_agent_lib::proc::{ProcsProcess, ProcsStatm};
use tokio_cron_scheduler::Job;
use regex::Regex;

use crate::{common::{configuration::DatabaseStoreLevel, ApplicationError, MonitorStatus, Status}, services::DbService};
use super::Monitor;

/**
 * The `ProcessMonitor` struct represents a moniitor for checking processes.
 * 
 * `name`: The monitor name.
 * `description`: The description of the monitor.
 * `application_names`: The application names to monitor.
 * `pids`: The process ids to monitor.
 * `regexp`: The regular expression.
 * `max_mem_usage`: The max memory usage.
 * `status`: The status of the monitor.
 * `database_service`: The database service.
 * `database_store_level`: The database store level.
 * `store_current_statm`: Store the current statm.
 * 
 */
#[derive(Debug, Clone)]
pub struct ProcessMonitor {
    /// The monitor name.
    name: String,
    /// Application names to monitor.
    application_names: Option<Vec<String>>,
    /// Pids to monitor.
    pids: Option<Vec<u32>>,
    /// regexp.
    regexp: Option<String>,    
    /// The max memory usage.
    max_mem_usage: Option<u32>,
    /// The status of the monitor.
    status: Arc<Mutex<HashMap<String, MonitorStatus>>>,    
    /// The database service
    database_service: Arc<Option<DbService>>,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
    /// The current statm.
    store_current_statm: bool,       
}

impl ProcessMonitor {
    /**
     * Create a new `ProcessMonitor`.
     * 
     * `name`: The monitor name.
     * `application_names`: The application names to monitor.
     * `pids`: The process ids to monitor.
     * `regexp`: The regular expression. 
     * `max_mem_usage`: The max memory usage.
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     * `store_current_statm`: Store the current statm.
     * 
     * Returns a new `ProcessMonitor`.
     */
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        description: &Option<String>,
        application_names: Option<Vec<String>>,
        pids: Option<Vec<u32>>,
        regexp: Option<String>,
        max_mem_usage: Option<u32>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        database_service: &Arc<Option<DbService>>,
        database_store_level: &DatabaseStoreLevel,
        store_current_statm: bool
    ) -> ProcessMonitor {
        debug!("Creating Process monitor: {}", &name);
        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name, description, Status::Unknown));
            }
            Err(err) => {
                error!("Error creating systemctl monitor: {err:?}");
            }
        }            
        ProcessMonitor {
            name: name.to_string(),
            application_names,
            pids,
            regexp,
            max_mem_usage,
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
            store_current_statm
        }
    }

    /**
     * Get process job.
     *
     * `schedule`: The schedule for the job.
     */
    pub fn get_process_monitor_job(
        &mut self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating Process monitor: {}", &self.name);
        let process_monitor = self.clone();
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {
            let process_monitor = process_monitor.clone();
            Box::pin(async move {
                let _ = process_monitor.clone().check().await.map_err(|err| error!("Error checking process monitor: {err:?}"));
            })              
        });        
        match job_result {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {err}").as_str(),
            )),
        }
    }

    /**
     * Check the applications.
     * 
     * Returns: The result of the check.
     * 
     * # Errors
     * - If there is an error checking the application.
     * 
     */
    pub async fn check(&mut self) -> Result<(), ApplicationError> {
        let mut statuses: Vec<Status> = Vec::new();
        let processes = ProcsProcess::get_all_processes();
        let regexp: Option<Regex> = match self.regexp {
            Some(ref regexp) => {
                match Regex::new(regexp) {
                    Ok(regexp) => Some(regexp),
                    Err(err) => {
                        error!("Error creating regular expression: {err:?}");
                        None
                    }
                }
            }
            None => None
        };
        if let Ok(processes) = processes {
            for process in processes {
                if  Self::check_pids(&self.pids, &process) || 
                    Self::check_application_names(&self.application_names, &process) || 
                    Self::check_regexp(&regexp, &process) {
                    let check_process = self.check_process(&process).await;
                    match check_process {
                        Ok(status) => {
                            statuses.push(status);
                        }
                        Err(err) => {
                            error!("Error checking process: {err:?}");
                        }
                    }
                }
            }
        }   

        let new_status = Status::get_max_status(statuses);
        self.set_status(&new_status).await;
        Ok(())
    }    

    /**
     * Check regexp.
     * 
     * `regexp`: The regular expression.
     * `process`: The process.
     * 
     * Returns: The status of the check.
     */
    fn check_regexp(regexp: &Option<Regex>, process: &ProcsProcess) -> bool {
        if let Some(regexp) = regexp {
            if let Some(name) = &process.name {
                return regexp.is_match(name);
            }
        }
        false
    }

    /**
     * Check application names.
     * 
     * `application_names`: The application names.
     * `process`: The process.
     * 
     * Returns: The status of the check.
     */
    fn check_application_names(application_names: &Option<Vec<String>>, process: &ProcsProcess) -> bool {
        if let Some(application_names) = application_names {
            if let Some(name) = &process.name {
                return application_names.contains(name);
            }
        }
        false
    }

    /**
     * Check if pid is in the pids to monitor.
     * 
     * `pids`: The pids to monitor.
     * 
     * Returns: The status of the check.
     */
    fn check_pids(pids: &Option<Vec<u32>>, process: &ProcsProcess) -> bool {
        if let Some(pids) = pids {
            if let Some(pid) = process.pid {
                return pids.contains(&pid);
            }
        }
        false
    }

    /**
     * Check the process.
     * 
     * `process`: The process.
     *  
     * Returns: The status of the check.
     */
    async fn check_process(&self, process: &ProcsProcess) -> Result<Status, ApplicationError> {
        debug!("Checking process: {process:?}");
        let Some(pid) = process.pid else { return Ok(Status::Ok) };
        let name = process.name.clone().unwrap_or("Unknown".to_string());
        let statm = ProcsStatm::get_statm(pid);
        if let Ok(statm) = statm {
            self.store_statm_values(pid, &name, &statm).await;
            Ok(self.check_max(&statm))
        } else {
            info!("Error getting statm for process, this could because the process no longer exist: {process:?}");
            Ok(Status::Ok)
        }
    }

    /**
     * Check the max memory usage.
     * 
     * `statm`: The statm values.
     * 
     * Returns: The status of the check.
     */
    fn check_max(&self, statm: &ProcsStatm) -> Status {        
        let Some(resident) = statm.resident else { return Status::Ok };   
        let Some(pagesize) = statm.pagesize else { return Status::Ok };
        let Some(max_mem_usage) = self.max_mem_usage else { return Status::Ok };    
        if (resident * pagesize) > max_mem_usage {
            return Status::Error { message: format!("Process memory usage is over the limit: {:?} > {max_mem_usage:?}", (resident * pagesize))};
        }
        Status::Ok
    }
    
    /**
     * Store the statm values.
     * 
     * `pid`: The process id.
     * `app_name`: The application name.
     * `statm`: The statm values.
     */
    async fn store_statm_values(&self, pid: u32, app_name: &str, statm: &ProcsStatm) {
        if self.store_current_statm {
            self.store_current_statm_values(pid, app_name, statm).await;
        }
    }

    /**
     * Store the current statm values.
     * 
     * `pid`: The process id.
     * `app_name`: The application name.
     * `statm`: The statm values.
     * 
     * Returns: The result of storing the statm values.
     * 
     */
    async fn store_current_statm_values(&self, pid: u32, app_name: &str, statm: &ProcsStatm) {
        if let Some(database_service) = self.database_service.as_ref() {
            let store_result: Result<(), ApplicationError> = database_service.store_statm_values(app_name, &pid, statm).await;
            match store_result {
                Ok(()) => {
                    debug!("Stored statm values for process: {pid:?}");
                }
                Err(err) => {
                    error!("Error storing statm values for process: {pid:?}, err: {err:?}");
                }
            }
        }
    }

}

/**
 * Implement the `Monitor` trait for `MemoryinfoMonitor`.
 */
impl super::Monitor for ProcessMonitor {
    /**
     * Get the name of the monitor.
     *
     * Returns: The name of the monitor.
     */
    fn get_name(&self) -> &str {
        &self.name
    }

    /**
     * Get the status of the monitor.
     *
     * Returns: The status of the monitor.
     */
    fn get_status(&self) -> Arc<Mutex<HashMap<String, MonitorStatus>>> {
        self.status.clone()
    }

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> Arc<Option<DbService>> {
        self.database_service.clone()
    }

    /**
     * Get the database store level.
     *
     * Returns: The database store level.
     */
    fn get_database_store_level(&self) -> DatabaseStoreLevel {
        self.database_store_level.clone()
    }
     
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_monitor_job() {
        let mut process_monitor = ProcessMonitor::new(
            "test_monitor",
            &None,
            Some(vec!["test_app".to_string()]),
            Some(vec![100u32]),
            Some("test".to_string()),
            Some(100),
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &DatabaseStoreLevel::None,
            false
        );
        let job_result = process_monitor.get_process_monitor_job("* * * * * *");
        assert!(job_result.is_ok());
    }

    #[tokio::test]
    async fn test_check() {
        let mut process_monitor = ProcessMonitor::new(
            "test_monitor",
            &None,
            Some(vec!["test_app".to_string()].clone()),
            Some(vec![100u32]),
            Some("test".to_string()),
            Some(100),
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &DatabaseStoreLevel::None,
            false
        );
        let check_result = process_monitor.check().await;
        assert!(check_result.is_ok());
    }

    #[tokio::test]
    async fn test_check_systemd_status_not_ok() {
        let mut process_monitor = ProcessMonitor::new(
            "systemd monitor",
            &None,
            Some(vec!["systemd".to_string()]),
            None,
            None,
            Some(1000),
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &DatabaseStoreLevel::None,
            false
        );
        let check_result = process_monitor.check().await;
        assert!(check_result.is_ok());
        assert!(process_monitor.get_status().lock().unwrap().get("systemd monitor").unwrap().status != Status::Ok);
    }

    #[tokio::test]
    async fn test_check_systemd_status_ok() {
        let mut process_monitor = ProcessMonitor::new(
            "systemd monitor",
            &None,
            Some(vec!["/sbin/init".to_string()]),
            None,
            None,
            Some(100000000),
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &DatabaseStoreLevel::None,
            false
        );
        let check_result = process_monitor.check().await;
        assert!(check_result.is_ok());
        assert!(process_monitor.get_status().lock().unwrap().get("systemd monitor").unwrap().status == Status::Ok);
    }    

}
