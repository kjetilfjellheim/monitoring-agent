use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{debug, error, info};
use monitoring_agent_lib::proc::{CmdLine, Statm};
use tokio_cron_scheduler::Job;

use crate::{common::{configuration::DatabaseStoreLevel, ApplicationError, MonitorStatus, Status}, services::DbService};

use super::Monitor;

/**
 * The `ProcessMonitor` struct represents a moniitor for checking processes.
 */
#[derive(Debug, Clone)]
pub struct ProcessMonitor {
    /// The monitor name.
    name: String,
    /// Application names to monitor.
    application_names: Vec<String>,
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
     * `max_mem_usage`: The max memory usage.
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     * `store_current_statm`: Store the current statm.
     * 
     * Returns a new `ProcessMonitor`.
     */
    pub fn new(
        name: &str,
        application_names: &[String],
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
                lock.insert(name.to_string(), MonitorStatus::new(name.to_string(), Status::Unknown));
            }
            Err(err) => {
                error!("Error creating systemctl monitor: {err:?}");
            }
        }            
        ProcessMonitor {
            name: name.to_string(),
            application_names: application_names.to_vec(),
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
        for app_name in &self.application_names {
            let new_statuses = self.check_application(app_name).await?;
            statuses.extend(new_statuses);
        }
        let new_status = Status::get_max_status(statuses);
        self.set_status(&new_status).await;
        Ok(())
    }    

    /** 
     * Check the application.
     * 
     * `app_name`: The application name.
     * 
     * Returns: The statuses of the checks.
     * 
     * # Errors
     * - If there is an error reading the cmdlines.
     * 
     */
    async fn check_application(&self, app_name: &str) -> Result<Vec<Status>, ApplicationError> {
        debug!("Checking application: {app_name:?}");
        let cmdlines_result = CmdLine::read_by_application(app_name);
        match cmdlines_result {
            Ok(cmdlines) => { 
                Ok(self.check_processes(app_name, &cmdlines).await)
            },
            Err(err) => {
                Err(ApplicationError::new(
                    format!("Error reading cmdlines for application: {app_name:?}, err: {err:?}").as_str(),
                ))
            }
        }
    }

    /**
     * Check the processes.
     * 
     * `app_name`: The application name.
     * `cmdlines`: The command lines.
     * 
     * Returns: The statuses of the checks.
     * 
     */
    async fn check_processes(&self, app_name: &str, cmdlines: &Vec<CmdLine>) -> Vec<Status> {
        debug!("Checking processes: {cmdlines:?}");
        let mut statuses = Vec::new();
        for cmdline in cmdlines {
            statuses.push(self.check_process(app_name, cmdline).await);
        };
        statuses
    }

    /**
     * Check the process.
     * 
     * `app_name`: The application name.
     * `cmdline`: The command line.
     *  
     * Returns: The status of the check.
     */
    async fn check_process(&self, app_name: &str, cmdline: &CmdLine) -> Status {
        debug!("Checking process: {cmdline:?}");
        let statm = Statm::get_statm(cmdline.pid);
        if let Ok(statm) = statm {
            self.store_statm_values(app_name, cmdline, &statm).await;
            self.check_max(&statm)
        } else {
            info!("Error getting statm for process, this could because the process no longer exist: {cmdline:?}");
            Status::Ok
        }
    }

    /**
     * Check the max memory usage.
     * 
     * `statm`: The statm values.
     * 
     * Returns: The status of the check.
     */
    fn check_max(&self, statm: &Statm) -> Status {        
        if let Some(max_mem_usage) = self.max_mem_usage {
            if statm.size > self.max_mem_usage {
                return Status::Error { message: format!("Process memory usage is over the limit: {:?} > {max_mem_usage:?}", statm.size)};
            }
        }
        Status::Ok
    }
    
    /**
     * Store the statm values.
     * 
     * `app_name`: The application name.
     * `cmdline`: The command line.
     * `statm`: The statm values.
     */
    async fn store_statm_values(&self, app_name: &str, cmdline: &CmdLine, statm: &Statm) {
        if self.store_current_statm {
            self.store_current_statm_values(app_name, cmdline, statm).await;
        }
    }

    /**
     * Store the current statm values.
     * 
     * `app_name`: The application name.
     * `cmdline`: The command line.
     * `statm`: The statm values.
     * 
     * Returns: The result of storing the statm values.
     * 
     */
    async fn store_current_statm_values(&self, app_name: &str, cmdline: &CmdLine, statm: &Statm) {
        if let Some(database_service) = &*self.database_service {
            let store_result: Result<(), ApplicationError> = database_service.store_statm_values(app_name, &cmdline.pid, statm).await;
            match store_result {
                Ok(()) => {
                    info!("Stored statm values for process: {cmdline:?}");
                }
                Err(err) => {
                    error!("Error storing statm values for process: {cmdline:?}, err: {err:?}");
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
            &vec!["test_app".to_string()],
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
            &vec!["test_app".to_string()],
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
            &vec!["/sbin/init".to_string()],
            Some(0),
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
            &vec!["/sbin/init".to_string()],
            Some(1000000),
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
