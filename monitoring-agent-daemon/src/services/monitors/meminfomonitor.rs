use std::mem;

use log::{debug, error, info};
use monitoring_agent_lib::proc::ProcsMeminfo;
use tokio_cron_scheduler::Job;

use crate::common::{configuration::DatabaseStoreLevel, ApplicationError, DatabaseServiceType, MonitorStatus, MonitorStatusType, Status};

use super::Monitor;

/**
 * Meminfo monitor.
 * 
 * This struct represents a meminfo monitor.
 * 
 * `name`: The name of the monitor.
 * `description`: The description of the monitor.
 * `max_percentage_mem`: The maximum percentage memory.
 * `max_percentage_swap`: The maximum percentage swap.
 * `status`: The status of the monitor.
 * `database_service`: The database service.
 * `database_store_level`: The database store level.
 * `store_current_meminfo`: Store the current meminfo.
 */
#[derive(Debug, Clone)]
pub struct MeminfoMonitor {
    /// The name of the monitor.
    pub name: String,   
    /// Minimum free percentage memory.
    pub error_percentage_used_mem: Option<f64>,
    /// Minimum free percentage swap memory.
    pub error_percentage_used_swap: Option<f64>,
    /// Warn free percentage memory.
    pub warn_percentage_used_mem: Option<f64>,
    /// Warn free percentage swap memory.
    pub warn_percentage_used_swap: Option<f64>,    
    /// The status of the monitor.
    pub status: MonitorStatusType,    
    /// The database service
    database_service: DatabaseServiceType,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
    /// The current load average.
    store_current_meminfo: bool,              
}

impl MeminfoMonitor {

    /**
     * Create a new `MeminfoMonitor`.
     * 
     * `name`: The name of the monitor.
     * `error_percentage_used_mem`: The maximum percentage memory.
     * `error_percentage_used_swap`: The maximum percentage swap.
     * `warn_percentage_used_mem`: The warn percentage memory.
     * `warn_percentage_used_swap`: The warn percentage swap. 
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     * `store_current_meminfo`: Store the current load average.
     * 
     * Returns: A new `MeminfoMonitor`.
     * 
     */
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::similar_names)]    
    pub fn new(
        name: &str,
        description: &Option<String>,
        error_percentage_used_mem: Option<f64>,
        error_percentage_used_swap: Option<f64>,
        warn_percentage_used_mem: Option<f64>,
        warn_percentage_used_swap: Option<f64>,        
        status: &MonitorStatusType,
        database_service: &DatabaseServiceType,
        database_store_level: &DatabaseStoreLevel,
        store_current_meminfo: bool,
    ) -> MeminfoMonitor {

        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name, description, Status::Unknown));
            }
            Err(err) => {
                error!("Error creating meminfo monitor: {:?}", err);
            }
        }

        MeminfoMonitor {
            name: name.to_string(),
            error_percentage_used_mem,
            error_percentage_used_swap,
            warn_percentage_used_mem,
            warn_percentage_used_swap,
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
            store_current_meminfo,
        }
    }

    /**
     * Check the memory use.
     * 
     * `meminfo`: The memory use.
     * 
     */
    #[allow(clippy::similar_names)]         
    async fn check_meminfo(&mut self, meminfo: &ProcsMeminfo) {    
        let percentage_mem_used = ProcsMeminfo::get_percent_used(meminfo.memfree, meminfo.memtotal);
        let percentage_swap_used = ProcsMeminfo::get_percent_used(meminfo.swapfree, meminfo.swaptotal);

        let free_percentage_mem_status = MeminfoMonitor::check_meminfo_values(self.error_percentage_used_mem, self.warn_percentage_used_mem, percentage_mem_used);
        let free_percentage_swap_status = MeminfoMonitor::check_meminfo_values(self.error_percentage_used_swap, self.warn_percentage_used_swap, percentage_swap_used);
        
        if mem::discriminant(&free_percentage_mem_status) == mem::discriminant(&Status::Error { message: String::new()}) || 
           mem::discriminant(&free_percentage_swap_status) == mem::discriminant(&Status::Error { message: String::new()}) {
            self.set_status(&Status::Error {
                message: format!(
                    "Memory check failed: {free_percentage_mem_status:?}, swap: {free_percentage_swap_status:?}"
                ),
            }).await;
            return;
        }
        if mem::discriminant(&free_percentage_mem_status) == mem::discriminant(&Status::Warn { message: String::new()}) || 
           mem::discriminant(&free_percentage_swap_status) == mem::discriminant(&Status::Warn { message: String::new()}) {
            self.set_status(&Status::Warn {
                message: format!(
                    "Memory check failed: {free_percentage_mem_status:?}, swap: {free_percentage_swap_status:?}"
                ),
            }).await;
            return;
        }                 
        self.set_status(&Status::Ok).await;        
    }

    /**
     * Check the memory values.
     * 
     * `error`: The error threshold.
     * `warn`: The warning threshold menory use.
     * `current`: The current load average.
     * 
     * Returns: The status of the check.
     * 
     */
    fn check_meminfo_values(error: Option<f64>, warn: Option<f64>, current: Option<f64>) -> Status {
        let Some(current) = current else { return Status::Ok };
        if let Some(error) = error {    
            if current > error {
                return Status::Error {
                    message: format!(
                        "Error memory use {current:0.2}% is more than {error:0.2}%"
                    ),
                };
            }
        }
        if let Some(warn) = warn {
            if current > warn {
                return Status::Warn {
                    message: format!(
                        "Warning memory use {current:0.2}% is more than {warn:0.2}%"
                    ),
                };
            }            
        }
        Status::Ok       
    }

    /**
     * Check and store the currentmeminfo.
     * 
     * `meminfo`: The current memory use.
     * 
     * 
     */
    async fn check_store_current_meminfo(&self, meminfo: &ProcsMeminfo) {
        if self.store_current_meminfo {
            self.store_current_meminfo(meminfo).await;
        }           
    }
    
    /**
     * Store the current memory use.
     * 
     * `meminfo`: The current load average.
     */
    async fn store_current_meminfo(&self, meminfo: &ProcsMeminfo) {
        if let Some(database_service) = self.database_service.as_ref() {
            match database_service.store_meminfo(meminfo).await {
                Ok(()) => {}
                Err(err) => {
                    error!("Error storing memory use: {:?}", err);
                }
            }
        }    
    }

    /**
     * Get meminfo monitor job.
     * 
     * `schedule`: The schedule for the job.
     * 
     * Returns: The meminfo monitor job.
     * 
     */
    pub fn get_meminfo_monitor_job(
        memory_monitor: Self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating meminfo monitor: {}", &memory_monitor.name);
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {                
            let mut memory_monitor = memory_monitor.clone();
            Box::pin(async move {
                memory_monitor.check().await;
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
        let meminfo = ProcsMeminfo::get_meminfo();
        match meminfo {
            Ok(meminfo) => {
                self.check_store_current_meminfo(&meminfo).await;
                self.check_meminfo(&meminfo).await;
            }
            Err(err) => {
                error!("Error getting meminfo: {:?}", err);
            }
        }                    
    }    

}

/**
 * Implement the `Monitor` trait for `MemoryinfoMonitor`.
 */
impl super::Monitor for MeminfoMonitor {
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
    use std::{collections::HashMap, sync::{Arc, Mutex}};

    use crate::{common::MonitorStatusType, services::monitors::MeminfoMonitor};

    use super::Monitor;

    #[tokio::test]
    async fn test_check() {
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            &None,
            Some(100.0),
            Some(100.0),
            Some(90.0),
            Some(90.0),            
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );
        monitor.check().await;
        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Ok);
    }

    /**
     * Test the check_max_meminfo function.
     * 
     * Test the following scenarios:
     * - Memory is lower on all.
     */
    #[tokio::test]
    async fn test_check_ok_meminfo() {
        // Test success. Memory lower on all
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            &None,
            Some(80.0),
            Some(80.0),
            Some(60.0),
            Some(60.0),            
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let meminfo = monitoring_agent_lib::proc::ProcsMeminfo {
            memtotal: Some(32000),
            memfree: Some(16000),
            memavailable: Some(16000),
            swaptotal: Some(10000),
            swapfree: Some(5000),
        };

        monitor.check_meminfo(&meminfo).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Ok);
    }

    /**
     * Test the check_max_meminfo function.
     * 
     * Test the following scenarios:
     * - Memory is warning level.
     */
    #[tokio::test]
    async fn test_check_warn_meminfo() {
        // Test success. Memory lower on all
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            &None,
            Some(95.0),
            Some(95.0),
            Some(80.0),
            Some(80.0),            
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let meminfo = monitoring_agent_lib::proc::ProcsMeminfo {
            memtotal: Some(12222772),
            memfree: Some(1356992),
            memavailable: Some(8980516),
            swaptotal: Some(10000),
            swapfree: Some(5000),
        };

        monitor.check_meminfo(&meminfo).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Warn { message: "Memory check failed: Warn { message: \"Warning memory use 88.90% is more than 80.00%\" }, swap: Ok".to_string() });
    }        

    /**
     * Test the check_max_meminfo function.
     * 
     * Test the following scenarios:
     * - Memory is error level.
     */
    #[tokio::test]
    async fn test_check_error_meminfo() {
        // Test success. Memory lower on all
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            &None,
            Some(95.0),
            Some(95.0),
            Some(80.0),
            Some(80.0),            
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let meminfo = monitoring_agent_lib::proc::ProcsMeminfo {
            memtotal: Some(12222772),
            memfree: Some(1200),
            memavailable: Some(8980516),
            swaptotal: Some(10000),
            swapfree: Some(5000),
        };

        monitor.check_meminfo(&meminfo).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Error { message: "Memory check failed: Error { message: \"Error memory use 99.99% is more than 95.00%\" }, swap: Ok".to_string() });
    }

    /**
     * Test the check_max_meminfo function.
     * 
     * Test the following scenarios:
     * - Memory is lower on all.
     */
    #[tokio::test]
    async fn test_check_max_meminfo_over() {
        // Test success. Memory lower on all
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            &None,
            Some(70.0),
            Some(15.0),
            Some(60.0),
            Some(10.0),            
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let meminfo = monitoring_agent_lib::proc::ProcsMeminfo {
            memtotal: Some(20000),
            memfree: Some(5000),
            memavailable: Some(16000),
            swaptotal: Some(10000),
            swapfree: Some(5000),
        };

        monitor.check_meminfo(&meminfo).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Error { message: "Memory check failed: Error { message: \"Error memory use 75.00% is more than 70.00%\" }, swap: Error { message: \"Error memory use 50.00% is more than 15.00%\" }".to_string() });
    }

    #[test]
    fn test_get_meminfo_monitor_job() {
        let status: MonitorStatusType =
            Arc::new(Mutex::new(HashMap::new()));
        let monitor = MeminfoMonitor::new(
            "test",
            &None,
            Some(100.0),
            Some(100.0),
            Some(100.0),
            Some(100.0),            
            &status,
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );
        let job = MeminfoMonitor::get_meminfo_monitor_job(monitor ,"0 0 * * * *");
        assert!(job.is_ok());
    }  
}