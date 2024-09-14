use log::{debug, error, info};
use monitoring_agent_lib::proc::ProcsLoadavg;
use tokio_cron_scheduler::Job;

use crate::common::{configuration::{DatabaseStoreLevel, ThresholdLevel}, ApplicationError, DatabaseServiceType, MonitorStatus, MonitorStatusType, Status};

use super::Monitor;
/**
 * Values for the status of the monitor to identify
 * the highest state.
 */
const ERROR: u8 = 2;
const WARN: u8 = 1;
const OK: u8 = 0;

/**
 * Load average monitor.
 * 
 * This struct represents a load average monitor.
 * 
 * `name`: The name of the monitor.
 * `description`: The description of the monitor.
 * `loadavg1min_max`: The max load average for 1 minute.
 * `loadavg5min_max`: The max load average for 5 minutes.
 * `loadavg15min_max`: The max load average for 10 minutes.
 * `threshold_1min_level`: The threshold for the 1 minute load average.
 * `threshold_5min_level`: The threshold for the 5 minute load average.
 * `threshold_15min_level`: The threshold for the 15 minute load average.
 * `status`: The status of the monitor.
 * `database_service`: The database service.
 * `database_store_level`: The database store level.
 * `store_current_loadavg`: Store the current load average.
 * 
 */
#[derive(Debug, Clone)]
pub struct LoadAvgMonitor {
    /// The name of the monitor.
    pub name: String,
    /// Max load average for 1 minute.
    pub loadavg1min_max: Option<f32>,
    /// Max load average for 5 minutes.
    pub loadavg5min_max: Option<f32>,
    /// Max load average for 10 minutes.
    pub loadavg15min_max: Option<f32>,
    /// The threshold for the 1 minute load average.
    pub threshold_1min_level: ThresholdLevel,
    /// The threshold for the 5 minute load average.
    pub threshold_5min_level: ThresholdLevel,
    /// The threshold for the 15 minute load average.
    pub threshold_15min_level: ThresholdLevel,    
    /// The status of the monitor.
    pub status: MonitorStatusType,
    /// The database service.
    database_service: DatabaseServiceType,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
    /// The current load average.
    store_current_loadavg: bool,              
}

impl LoadAvgMonitor {

    /**
     * Create a new load average monitor.
     * 
     * `name`: The name of the monitor.
     * `loadavg1min_max`: The max load average for 1 minute.
     * `loadavg5min_max`: The max load average for 5 minutes.
     * `loadavg15min_max`: The max load average for 10 minutes.
     * `threshold_1min_level`: The threshold for the 1 minute load average.
     * `threshold_5min_level`: The threshold for the 5 minute load average.
     * `threshold_15min_level`: The threshold for the 15 minute load average.
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     * `store_current_loadavg`: Store the current load average.
     * 
     * Returns: A new load average monitor.
     * 
     */
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::similar_names)]    
    pub fn new(
        name: &str,
        description: &Option<String>,
        loadavg1min_max: Option<f32>,
        loadavg5min_max: Option<f32>,
        loadavg15min_max: Option<f32>,
        threshold_1min_level: ThresholdLevel,
        threshold_5min_level: ThresholdLevel,
        threshold_15min_level: ThresholdLevel,
        status: &MonitorStatusType,
        database_service: &DatabaseServiceType,
        database_store_level: &DatabaseStoreLevel,
        store_current_loadavg: bool,
    ) -> LoadAvgMonitor {

        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name, description, Status::Unknown));
            }
            Err(err) => {
                error!("Error creating loadavg monitor: {:?}", err);
            }
        }

        LoadAvgMonitor {
            name: name.to_string(),
            loadavg1min_max,
            loadavg5min_max,
            loadavg15min_max,
            threshold_1min_level,
            threshold_5min_level,
            threshold_15min_level,
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
            store_current_loadavg,
        }
    }

    /**
     * Check the load average.
     * 
     * `loadavg`: The current load average.
     * 
     */
    #[allow(clippy::similar_names)]         
    async fn check_loadavg(&mut self, loadavg: &ProcsLoadavg) {    
        let status_1min = Self::check_loadavg_values(self.loadavg1min_max, loadavg.loadavg1min, self.threshold_1min_level);
        let status_5min = Self::check_loadavg_values(self.loadavg5min_max, loadavg.loadavg5min, self.threshold_5min_level);
        let status_15min = Self::check_loadavg_values(self.loadavg15min_max, loadavg.loadavg15min, self.threshold_15min_level);        
        let max_level = Self::get_max_error(&status_1min, &status_5min, &status_15min);
        self.set_monitor_status(max_level, &status_1min, &status_5min, &status_15min).await;   

    }

    /**
     * Set the status of the monitor based on the max level.
     * 
     * `max_level`: The status of the monitor.
     * `status_1min`: The status of the 1 minute load average.
     * `status_5min`: The status of the 5 minute load average.
     * `status_15min`: The status of the 15 minute load average.
     * 
     */
    #[allow(clippy::similar_names)]
    async fn set_monitor_status(&mut self, max_level: u8, status_1min: &Status, status_5min: &Status, status_15min: &Status) {
        match max_level {
            ERROR => {
                self.set_status(&Status::Error {
                    message: format!(
                        "Load average check failed: 1min: {status_1min:?}, 5min: {status_5min:?}, 15min: {status_15min:?}"
                    ),
                }).await;
            }
            WARN => {
                self.set_status(&Status::Warn {
                    message: format!(
                        "Load average check failed: 1min: {status_1min:?}, 5min: {status_5min:?}, 15min: {status_15min:?}"
                    ),
                }).await;
            }
            OK => {
                self.set_status(&Status::Ok).await;
            }
            _ => {
                self.set_status(&Status::Unknown).await;
            }
        }
    }
    
    /**
     * Get the max level of the status.
     * 
     * `status_1min`: The status of the 1 minute load average.
     * `status_5min`: The status of the 5 minute load average.
     * `status_15min`: The status of the 15 minute load average.
     * 
     * Returns: The max error level.
     */
    #[allow(clippy::similar_names)]
    fn get_max_error(status_1min: &Status, status_5min: &Status, status_15min: &Status) -> u8 {
        let mut max = 0;
        Self::check_max_status(status_1min, &mut max);
        Self::check_max_status(status_5min, &mut max);
        Self::check_max_status(status_15min, &mut max);
        max
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
    fn check_loadavg_values(max: Option<f32>, current: Option<f32>, threshold_level : ThresholdLevel) -> Status {
        let Some(current) = current else { return Status::Ok };
        let Some(max) = max else { return Status::Ok };
            
        if current > max {
            match threshold_level {
                ThresholdLevel::Error => {
                    return Status::Error {
                        message: format!(
                            "Load average {current} is greater than max load average {max}"
                        ),
                    };
                }
                ThresholdLevel::Warn => {
                    return Status::Warn {
                        message: format!(
                            "Load average {current} is greater than max load average {max}"
                        ),
                    };
                }
            }
        }
        Status::Ok       
    }

    /**
     * Check and store the current load average.
     * 
     * `loadavg`: The current load average.
     */
    async fn check_store_current_loadavg(&self, loadavg: &ProcsLoadavg) {
        if self.store_current_loadavg {
            self.store_current_loadavg(loadavg).await;
        }           
    }
    
    /**
     * Store the current load average.
     * 
     * `loadavg`: The current load average.
     */
    async fn store_current_loadavg(&self, loadavg: &ProcsLoadavg) {
        let database_service = self.database_service.as_ref();
        if let Some(database_service) = database_service {
            let _ = database_service.store_loadavg(loadavg).await.map_err(|err | error!("Error storing load average: {:?}", err));
        }
    }

    /**
     * Get a loadavg monitor job.
     * 
     * `schedule`: The schedule.
     * `name`: The name of the monitor.
     * `threshold_1min`: The threshold for the 1 minute load average.
     * `threshold_5min`: The threshold for the 5 minute load average.
     * `threshold_15min`: The threshold for the 10 minute load average.
     * `store_values`: Store the values in the database.
     * `status`: The status.
     * `database_store_level`: The database store level.
     * 
     * `result`: The result of getting the loadavg monitor job.
     * 
     * throws: `ApplicationError`: If the job fails to be created.
     * 
     */
    pub fn get_loadavg_monitor_job(
        loadavg_monitor: Self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating Loadavg monitor: {}", &loadavg_monitor.name);
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {                
            let mut loadavg_monitor = loadavg_monitor.clone();
            Box::pin(async move {
                loadavg_monitor.check().await;
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
        let loadavg = ProcsLoadavg::get_loadavg();
        match loadavg {
            Ok(loadavg) => {
                self.check_store_current_loadavg(&loadavg).await;
                self.check_loadavg(&loadavg).await;
            }
            Err(err) => {
                error!("Error getting load average: {:?}", err);
            }
        }                    
    }    

    /**
     * Check if the current status is higher than the current.
     * 
     * `status`: The status of the monitor.
     * `current`: The current status value.
     * 
     */
    fn check_max_status(status: &Status, current: &mut u8) {
        match status {
            Status::Error { message: _ } => {
                *current = std::cmp::max(*current, ERROR);
            }
            Status::Warn { message: _ } => {
                *current = std::cmp::max(*current, WARN);
            }
            _ => {}
        }
    }    

}



/**
 * Implement the `Monitor` trait for `LoadAvgMonitor`.
 */
impl super::Monitor for LoadAvgMonitor {
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
    use crate::{common::{configuration::{DatabaseStoreLevel, ThresholdLevel}, MonitorStatusType}, services::monitors::LoadAvgMonitor};

    use super::Monitor;

    /**
     * Test the check_loadavg_values function.'
     * 
     * Test the following scenarios:
     * - Load average is equal to max load average.
     * - Load average is higher than max load average.
     * - Load average is max has value, but none is retrieved.
     * - Load average is max is None, but load average has value.
     * - Load average and max load average are None.
     */
    #[test]
    fn test_check_loadavg_values() {
        let status = LoadAvgMonitor::check_loadavg_values(Some(1.0), Some(1.0), ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = LoadAvgMonitor::check_loadavg_values(Some(1.0), Some(2.0), ThresholdLevel::Error);
        assert_eq!(status, super::Status::Error {
            message: "Load average 2 is greater than max load average 1".to_string()
        });

        let status: crate::common::Status = LoadAvgMonitor::check_loadavg_values(Some(1.0), None, ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = LoadAvgMonitor::check_loadavg_values(None, Some(1.0), ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = LoadAvgMonitor::check_loadavg_values(None, None, ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);
    }

    /**
     * Test the check_loadavg function.
     * 
     * Test the following scenarios:
     * - Load average is lower on all.
     */
    #[tokio::test]
    async fn test_check_loadavg_lower_on_all() {
        // Test success. Loadaverage lower on all
        let mut monitor = super::LoadAvgMonitor::new(
            "test",
            &None,
            Some(1.0),
            Some(2.0),
            Some(3.0),
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let loadavg = monitoring_agent_lib::proc::ProcsLoadavg {
            loadavg1min: Some(1.0),
            loadavg5min: Some(2.0),
            loadavg15min: Some(3.0),
            current_running_processes: Some(1),
            total_number_of_processes: Some(10)
        };

        monitor.check_loadavg(&loadavg).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Ok);
    }

    /**
     * Test the check_loadavg function.
     * 
     * Test the following scenarios:
     * - Load average is higher on 1 minute.
     */
    #[tokio::test]
    async fn test_check_loadavg_1min_higher() {
        // Test success. Loadaverage lower on all
        let mut monitor = super::LoadAvgMonitor::new(
            "test",
            &None,
            Some(1.0),
            Some(2.0),
            Some(3.0),
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let loadavg = monitoring_agent_lib::proc::ProcsLoadavg {
            loadavg1min: Some(1.1),
            loadavg5min: Some(2.0),
            loadavg15min: Some(3.0),
            current_running_processes: Some(1),
            total_number_of_processes: Some(10)
        };

        monitor.check_loadavg(&loadavg).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Error { message: "Load average check failed: 1min: Error { message: \"Load average 1.1 is greater than max load average 1\" }, 5min: Ok, 15min: Ok".to_string() } );
    }

    /**
     * Test the check_loadavg function.
     * 
     * Test the following scenarios:
     * - Load average is higher on 5 minutes.
     */
    #[tokio::test]
    async fn test_check_loadavg_5min_higher() {
        // Test success. Loadaverage lower on all
        let mut monitor = LoadAvgMonitor::new(
            "test",
            &None,
            Some(1.0),
            Some(2.0),
            Some(3.0),
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let loadavg = monitoring_agent_lib::proc::ProcsLoadavg {
            loadavg1min: Some(1.0),
            loadavg5min: Some(2.1),
            loadavg15min: Some(3.0),
            current_running_processes: Some(1),
            total_number_of_processes: Some(10)
        };

        monitor.check_loadavg(&loadavg).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Error { message: "Load average check failed: 1min: Ok, 5min: Error { message: \"Load average 2.1 is greater than max load average 2\" }, 15min: Ok".to_string() } );
    }

    /**
     * Test the check_loadavg function.
     * 
     * Test the following scenarios:
     * - Load average is higher on 10 minutes.
     */ 
    #[tokio::test]
    async fn test_check_loadavg_15min_higher() {
        // Test success. Loadaverage lower on all
        let mut monitor = LoadAvgMonitor::new(
            "test",
            &None,
            Some(1.0),
            Some(2.0),
            Some(3.0),
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            &Arc::new(Mutex::new(HashMap::new())),
            &Arc::new(None),
            &super::DatabaseStoreLevel::None,
            false,
        );

        let loadavg = monitoring_agent_lib::proc::ProcsLoadavg {
            loadavg1min: Some(1.0),
            loadavg5min: Some(2.0),
            loadavg15min: Some(3.1),
            current_running_processes: Some(1),
            total_number_of_processes: Some(10)
        };

        monitor.check_loadavg(&loadavg).await;

        let status = monitor.get_status();
        let status = status.lock().unwrap();
        assert_eq!(status.get("test").unwrap().status, super::Status::Error { message: "Load average check failed: 1min: Ok, 5min: Ok, 15min: Error { message: \"Load average 3.1 is greater than max load average 3\" }".to_string() } );
    }     

    #[test]
    fn test_get_loadavg_monitor_job() {
        let status: MonitorStatusType =
            Arc::new(Mutex::new(HashMap::new()));
        let monitor = LoadAvgMonitor::new(
            "test",
            &None,
            Some(1.0),
            Some(2.0),
            Some(3.0),
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            ThresholdLevel::Error,
            &status,
            &Arc::new(None),
            &DatabaseStoreLevel::None,
            false,    
        );
        let job = LoadAvgMonitor::get_loadavg_monitor_job(monitor, "0 0 * * * *");
        assert!(job.is_ok());
    }   

    #[test]
    fn test_check_max_status() {
        let mut max = 0;
        let status = super::Status::Error { message: "Error".to_string() };
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 2);

        let mut max = 0;
        let status = super::Status::Warn { message: "Warn".to_string() };
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 1);

        let mut max = 0;
        let status = super::Status::Ok;
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 0);

        let mut max = 1;
        let status = super::Status::Error { message: "Error".to_string() };
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 2);

        let mut max = 1;
        let status = super::Status::Warn { message: "Warn".to_string() };
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 1);

        let mut max = 1;
        let status = super::Status::Ok;
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 1);  

        let mut max = 2;
        let status = super::Status::Error { message: "Error".to_string() };
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 2);

        let mut max = 2;
        let status = super::Status::Warn { message: "Warn".to_string() };
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 2);

        let mut max = 2;
        let status = super::Status::Ok;
        super::LoadAvgMonitor::check_max_status(&status, &mut max);
        assert_eq!(max, 2);              
    }

    #[test]
    fn test_get_max_error() {
        let status_1min = super::Status::Error { message: "Error".to_string() };
        let status_5min = super::Status::Warn { message: "Warn".to_string() };
        let status_15min = super::Status::Ok;
        let max = super::LoadAvgMonitor::get_max_error(&status_1min, &status_5min, &status_15min);
        assert_eq!(max, 2);

        let status_1min = super::Status::Warn { message: "Warn".to_string() };
        let status_5min = super::Status::Error { message: "Error".to_string() };
        let status_15min = super::Status::Ok;
        let max = super::LoadAvgMonitor::get_max_error(&status_1min, &status_5min, &status_15min);
        assert_eq!(max, 2);

        let status_1min = super::Status::Ok;
        let status_5min = super::Status::Error { message: "Error".to_string() };
        let status_15min = super::Status::Warn { message: "Warn".to_string() };
        let max = super::LoadAvgMonitor::get_max_error(&status_1min, &status_5min, &status_15min);
        assert_eq!(max, 2);

        let status_1min = super::Status::Ok;
        let status_5min = super::Status::Warn { message: "Warn".to_string() };
        let status_15min = super::Status::Warn { message: "Error".to_string() };
        let max = super::LoadAvgMonitor::get_max_error(&status_1min, &status_5min, &status_15min);
        assert_eq!(max, 1);

        let status_1min = super::Status::Ok;
        let status_5min = super::Status::Ok;
        let status_15min = super::Status::Warn { message: "Error".to_string() };
        let max = super::LoadAvgMonitor::get_max_error(&status_1min, &status_5min, &status_15min);
        assert_eq!(max, 1);

        let status_1min = super::Status::Ok;
        let status_5min = super::Status::Ok;
        let status_15min = super::Status::Ok;
        let max = super::LoadAvgMonitor::get_max_error(&status_1min, &status_5min, &status_15min);
        assert_eq!(max, 0);        
    }

    #[test]
    fn test_check_loadavg_values_verify() {
        let status = super::LoadAvgMonitor::check_loadavg_values(Some(1.0), Some(1.0), ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = super::LoadAvgMonitor::check_loadavg_values(Some(1.0), Some(2.0), ThresholdLevel::Error);
        assert_eq!(status, super::Status::Error { message: "Load average 2 is greater than max load average 1".to_string() });

        let status: crate::common::Status = super::LoadAvgMonitor::check_loadavg_values(Some(1.0), None, ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = super::LoadAvgMonitor::check_loadavg_values(None, Some(1.0), ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = super::LoadAvgMonitor::check_loadavg_values(None, None, ThresholdLevel::Error);
        assert_eq!(status, super::Status::Ok);

        let status = super::LoadAvgMonitor::check_loadavg_values(Some(1.0), Some(2.0), ThresholdLevel::Warn);
        assert_eq!(status, super::Status::Warn { message: "Load average 2 is greater than max load average 1".to_string() });

    }
}