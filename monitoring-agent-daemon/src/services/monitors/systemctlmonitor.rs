use std::collections::HashSet;

use log::{debug, error, info};
use tokio_cron_scheduler::Job;

use crate::common::{configuration::DatabaseStoreLevel, ApplicationError, DatabaseServiceType, MonitorStatus, MonitorStatusType, Status};

use super::Monitor;

const SYSTEMD_ACTIVE_STATUS: &str = "active";

/**
 * Systemctl monitor.
 * 
 * This struct represents a systemctl monitor.
 * 
 * `name`: The name of the monitor.
 * `description`: The description of the monitor.
 * `status`: The status of the monitor.
 * `database_service`: The database service.
 * `database_store_level`: The database store level.
 * `active`: The services to monitor.
 * 
 */
#[derive(Debug, Clone)]
pub struct SystemctlMonitor {
    /// The name of the monitor.
    pub name: String,   
    /// The status of the monitor.
    pub status: MonitorStatusType,
    /// The database service.
    database_service: DatabaseServiceType,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
    /// The services to monitor.
    active: Vec<String>,
}

impl SystemctlMonitor {
    /**
     * Create a new Systemctl monitor.
     *
     * `name`: The name of the monitor.
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     * `active`: The services to monitor.
     *
     */
    pub fn new(
        name: &str,
        description: &Option<String>,
        status: &MonitorStatusType,
        database_service: &DatabaseServiceType,
        database_store_level: &DatabaseStoreLevel,
        active: Vec<String>,
    ) -> SystemctlMonitor {
        debug!("Creating Systemctl monitor: {}", &name);
        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name, description, Status::Unknown));
            }
            Err(err) => {
                error!("Error creating systemctl monitor: {err:?}");
            }
        }        
        SystemctlMonitor {
            name: name.to_string(),
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
            active,
        }
    }

    /**
     * Get systemctl job.
     *
     * `systemctl_monitor`: The systemctl monitor.
     * `schedule`: The schedule for the job.
     */
    pub fn get_systemctl_monitor_job(
        systemctl_monitor: Self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating Systemctl monitor: {}", &systemctl_monitor.name);
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {
            Box::pin({
                let mut systemctl_monitor = systemctl_monitor.clone();
                async move {
                    systemctl_monitor.check().await;
                }
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
     * Check the monitor.
     */
    async fn check(&mut self) {
        debug!("Checking monitor: {}", &self.name);
        let output = tokio::process::Command::new("systemctl")
            .arg("--all")
            .output()
            .await
            .expect("failed to execute process");
        let command_output = String::from_utf8_lossy(&output.stdout);        
        let non_active = self.get_nonactive_status(&command_output);
        let error_message = format!("Non-active services: {non_active:?}");
        if non_active.is_empty() {
            self.set_status(&Status::Ok).await;            
        } else {
            self.set_status(&Status::Error { message: error_message }).await;
        }        
    }

    /**
     * Get the non-active status.
     * 
     * `command_output`: The command output.
     * 
     * Returns: The non-active services.
     */
    fn get_nonactive_status(&self, command_output: &str) -> Vec<String> {
        let mut non_active: Vec<String> = vec![];
        let active_set: HashSet<String> = HashSet::from_iter(self.active.clone());
        for line in command_output.lines() {
            let data = line.split_whitespace().collect::<Vec<&str>>();
            if data.len() >= 3 && active_set.contains(&(data[0].replace(".service", "").clone()).to_string()) && data[2] != SYSTEMD_ACTIVE_STATUS {
                non_active.push(data[0].to_string());
            }                
        }
        non_active
    }

}

/**
 * Implement the `Monitor` trait for `SystemctlMonitor`.
 */
impl super::Monitor for SystemctlMonitor {
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
    fn get_status(&self) -> MonitorStatusType {
        self.status.clone()
    }

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> DatabaseServiceType {
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
    
    use std::fs;

    use super::*;
    use crate::services::monitors::common::Monitor;

    /**
     * Verifies that the command monitor can be run.
     */
    #[tokio::test]
    async fn test_check() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let database_service = std::sync::Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let active = vec![  ];
        let systemctl_monitor = SystemctlMonitor::new(
            "test",
            &None,
            &status,
            &database_service,
            &database_store_level,
            active,
        );
        systemctl_monitor.clone().check().await;
        let status = systemctl_monitor.get_status();
        let lock = status.lock().unwrap();
        let monitor_status = lock.get("test").unwrap();
        assert_eq!(&monitor_status.status, &Status::Ok);
    }

    /**
     * Test the Systemctl monitor without checking any.
     */
    #[tokio::test]
    async fn test_systemctl_monitor_with_no_active() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let database_service = std::sync::Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let active = vec![ "ssh".to_string() ];
        let systemctl_monitor = SystemctlMonitor::new(
            "test",
            &None,
            &status,
            &database_service,
            &database_store_level,
            active,
        );
        let command_str = String::from_utf8(fs::read("resources/test/systemctl_test.out").unwrap()).unwrap();
        let non_active = systemctl_monitor.get_nonactive_status(command_str.as_str());
        assert_eq!(non_active.len(), 0);
    }

    /**
     * Test the Systemctl monitor checking uuidd inactive.
     */
    #[tokio::test]
    async fn test_systemctl_monitor_with_uuidd_inactive() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let database_service = std::sync::Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let active = vec![ "uuidd".to_string() ];
        let systemctl_monitor = SystemctlMonitor::new(
            "test",
            &None,
            &status,
            &database_service,
            &database_store_level,
            active,
        );
        let command_str = String::from_utf8(fs::read("resources/test/systemctl_uuidd_inactive.out").unwrap()).unwrap();
        let non_active = systemctl_monitor.get_nonactive_status(command_str.as_str());
        assert_eq!(non_active.len(), 1);
    }    

    #[test]
    fn test_get_systemctl_monitor_job() {
        let status: MonitorStatusType =
        std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let monitor = SystemctlMonitor::new(
            "test",
            &None,
            &status,
            &std::sync::Arc::new(None),
            &DatabaseStoreLevel::None,
            vec![],
        );
        let job = SystemctlMonitor::get_systemctl_monitor_job(monitor, "0 0 * * * *");
        assert!(job.is_ok());
    }  

}