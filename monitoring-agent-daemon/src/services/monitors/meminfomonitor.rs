use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{error, info};
use monitoring_agent_lib::proc::ProcsMeminfo;
use tokio_cron_scheduler::Job;

use crate::{common::{configuration::DatabaseStoreLevel, ApplicationError, MonitorStatus, Status}, DbService};

use super::Monitor;

#[derive(Debug, Clone)]
pub struct MeminfoMonitor {
    /// The name of the monitor.
    pub name: String,
    /// Minimum free percentage memory.
    pub max_percentage_mem: Option<f64>,
    /// Minimum free percentage swap memory.
    pub max_percentage_swap: Option<f64>,
    /// The status of the monitor.
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,    
    /// The database service
    database_service: Arc<Option<DbService>>,
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
     * `max_percentage_mem`: The maximum percentage memory.
     * `max_percentage_swap`: The maximum percentage swap.
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
        max_percentage_mem: Option<f64>,
        max_percentage_swap: Option<f64>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        database_service: &Arc<Option<DbService>>,
        database_store_level: &DatabaseStoreLevel,
        store_current_meminfo: bool,
    ) -> MeminfoMonitor {

        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name.to_string(), Status::Unknown));
            }
            Err(err) => {
                error!("Error creating meminfo monitor: {:?}", err);
            }
        }

        MeminfoMonitor {
            name: name.to_string(),
            max_percentage_mem,
            max_percentage_swap,
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
    fn check_meminfo(&mut self, meminfo: &ProcsMeminfo) {    
        let percentage_mem_used = ProcsMeminfo::get_percent_used(meminfo.memfree, meminfo.memtotal);
        let percentage_swap_used = ProcsMeminfo::get_percent_used(meminfo.swapfree, meminfo.swaptotal);

        let free_percentage_mem_status = MeminfoMonitor::check_meminfo_values(self.max_percentage_mem, percentage_mem_used);
        let free_percentage_swap_status = MeminfoMonitor::check_meminfo_values(self.max_percentage_swap, percentage_swap_used);
        
        if free_percentage_mem_status != Status::Ok || free_percentage_swap_status != Status::Ok{
            self.set_status(&Status::Error {
                message: format!(
                    "Meminfo check failed: mem: {free_percentage_mem_status:?}, swap: {free_percentage_swap_status:?}"
                ),
            });
        } else {
            self.set_status(&Status::Ok);
        }
    }

    /**
     * Check the load average values.
     * 
     * `max`: The max load average.
     * `current`: The current load average.
     * 
     * Returns: The status of the check.
     * 
     */
    fn check_meminfo_values(max: Option<f64>, current: Option<f64>) -> Status {
        let Some(current) = current else { return Status::Ok };
        let Some(max) = max else { return Status::Ok };
            
        if current > max {
            return Status::Error {
                message: format!(
                    "Memory use {current:0.3}% is more than {max:0.3}%"
                ),
            };
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
    fn check_store_current_meminfo(&self, meminfo: &ProcsMeminfo) {
        if self.store_current_meminfo {
            self.store_current_meminfo(meminfo);
        }           
    }
    
    /**
     * Store the current memory use.
     * 
     * `meminfo`: The current load average.
     */
    fn store_current_meminfo(&self, meminfo: &ProcsMeminfo) {
        match self.database_service.as_ref() {            
            Some(database_service) => {
                match database_service.store_meminfo(meminfo) {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Error storing memory use: {:?}", err);
                    }
                }
            }
            None => {}
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
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::similar_names)]    
    pub fn get_meminfo_monitor_job(
        &mut self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating meminfo monitor: {}", &self.name);
        let mut meminfo_monitor = self.clone();       
        let job_result = Job::new(schedule, move |_uuid, _locked| {                
            meminfo_monitor.check();
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
    fn check(&mut self) {
        let meminfo = ProcsMeminfo::get_meminfo();
        match meminfo {
            Ok(meminfo) => {
                self.check_store_current_meminfo(&meminfo);
                self.check_meminfo(&meminfo);
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
    use std::{collections::HashMap, sync::{Arc, Mutex}};

    use super::Monitor;

    #[test]
    fn test_check() {
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            Some(100.0),
            Some(100.0),
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );
        monitor.check();
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
    #[test]
    fn test_check_max_meminfo() {
        // Test success. Memory lower on all
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            Some(80.0),
            Some(80.0),
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

        monitor.check_meminfo(&meminfo);

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
    #[test]
    fn test_check_max_meminfo_over() {
        // Test success. Memory lower on all
        let mut monitor = super::MeminfoMonitor::new(
            "test",
            Some(70.0),
            Some(15.0),
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

        monitor.check_meminfo(&meminfo);

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Error { message: "Meminfo check failed: mem: Error { message: \"Memory use 75.000% is more than 70.000%\" }, swap: Error { message: \"Memory use 50.000% is more than 15.000%\" }".to_string() });
    }


}