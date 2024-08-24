use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{debug, error, info};
use tokio_cron_scheduler::Job;

use crate::{common::{configuration::DatabaseStoreLevel, ApplicationError, MonitorStatus, Status}, services::{monitors::Monitor, DbService}};

/**
 * Database monitor.
 * 
 * This struct represents a database monitor.
 * 
 * `name`: The name of the monitor.
 * `description`: The description of the monitor.
 * `query_max_time`: The max query time.
 * `status`: The status of the monitor.
 * `database_service`: The database service.
 * `database_store_level`: The database store level.
 */
#[derive(Debug, Clone)]
pub struct DatabaseMonitor {
    /// The name of the monitor.
    pub name: String, 
    /// Max query time.
    pub query_max_time: Option<u32>,
    /// The current status of the monitor.
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
    /// The database service.
    database_service: Arc<Option<DbService>>,   
    /// The database store level.
    database_store_level: DatabaseStoreLevel,        
}

impl DatabaseMonitor {
    /**
     * Create a new database monitor.
     *
     * `name`: The name of the monitor.
     * `query_max_time`: The max query time.
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     * 
     * Returns a new `DatabaseMonitor`.
     */
    pub fn new(
        name: &str,
        description: &Option<String>,
        query_max_time: Option<u32>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        database_service: &Arc<Option<DbService>>,
        database_store_level: &DatabaseStoreLevel,
    ) -> DatabaseMonitor {

        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name, description, Status::Unknown));
            }
            Err(err) => {
                error!("Error creating command monitor: {:?}", err);
            }
        }

        DatabaseMonitor {
            name: name.to_string(),
            query_max_time,
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
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
    pub fn get_database_monitor_job(
        &mut self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating database monitor: {}", &self.name);
        let database_monitor = self.clone();       
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {                
            let mut database_monitor = database_monitor.clone();
            Box::pin(async move {
                database_monitor.check().await;
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
     * Check the status of the database.
     */
    async fn check(&mut self) {
        debug!("Checking monitor: {}", &self.name);
        let Some(database_service) = &*self.database_service else {
            error!("Database service not found.");
                return;
        };
        let mut status = Status::Ok;
        if self.query_max_time.is_some() {
            let overtimed_query = match database_service.query_long_running_queries(self.query_max_time.unwrap()).await {
                Ok(query) => query,
                Err(err) => {
                    error!("Error checking query time: {:?}", err);
                    return;
                }
            };
            if !overtimed_query.is_empty() {
                status = Status::Error {
                    message: format!("Long queries found: {overtimed_query:?}"),
                };
            }
        }
        self.set_status(&status).await;
    
    } 

}

/**
 * Implement the `Monitor` trait for `HttpMonitor`.
 */
impl super::Monitor for DatabaseMonitor {
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
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_new() {
        let name = "test";
        let status = Arc::new(Mutex::new(HashMap::new()));
        let database_service = Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let database_monitor = DatabaseMonitor::new(name, &None, None, &status, &database_service, &database_store_level);
        assert_eq!(database_monitor.name, name);
    }

    #[test]
    fn test_get_database_monitor_job() {
        let name = "test";
        let status = Arc::new(Mutex::new(HashMap::new()));
        let database_service = Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let mut database_monitor = DatabaseMonitor::new(name, &None, None, &status, &database_service, &database_store_level);
        let job = database_monitor.get_database_monitor_job("* * * * * *");
        assert!(job.is_ok());
    }

    #[tokio::test]
    async fn test_check() {
        let name = "test";
        let status = Arc::new(Mutex::new(HashMap::new()));
        let database_service = Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let mut database_monitor = DatabaseMonitor::new(name, &None, None, &status, &database_service, &database_store_level);
        let check = database_monitor.check().await;
        assert_eq!(check, ());
    }

    #[test]
    fn test_get_name() {
        let name = "test";
        let status = Arc::new(Mutex::new(HashMap::new()));
        let database_service = Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let database_monitor = DatabaseMonitor::new(name, &None, None, &status, &database_service, &database_store_level);
        assert_eq!(database_monitor.get_name(), name);
    }

    #[test]
    fn test_get_status() {
        let name = "test";
        let status = Arc::new(Mutex::new(HashMap::new()));
        let database_service = Arc::new(None);
        let database_store_level = DatabaseStoreLevel::None;
        let database_monitor = DatabaseMonitor::new(name, &None, None, &status, &database_service, &database_store_level);
        assert_eq!(database_monitor.get_status().lock().unwrap().get("test").unwrap().status, Status::Unknown);
    }
}