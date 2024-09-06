use std::sync::Arc;

use log::info;
use tokio_cron_scheduler::Job;
use tracing::error;

use crate::{common::ApplicationError, services::DbService};

/**
 * The `DbCleanupJob` struct contains the database service and the maximum time stored in the database.
 *  
 * The `DbCleanupJob` struct has the following fields:
 * * `db_service`: Database service if defined.
 * * `max_time_stored_db`: Max number of hours to store data in the database.
 * 
 */
#[derive(Debug, Clone)]
pub struct DbCleanupJob {
    pub db_service: Arc<Option<DbService>>,
    pub max_time_stored_db: u32
}

impl DbCleanupJob {
    /**
     * Create a new `DbCleanupJob` with the database service and the maximum time stored in the database.
     * 
     * # Arguments
     * * `db_service` - The database service.
     * * `max_time_stored_db` - The maximum time stored in the database.
     * 
     * # Returns
     * The `DbCleanupJob` with the database service and the maximum time stored in the database.
     */
    pub fn new(db_service: &Arc<Option<DbService>>, max_time_stored_db: u32) -> Self {
        DbCleanupJob {
            db_service: db_service.clone(),
            max_time_stored_db
        }
    }

    /**
     * Get the database cleanup job.
     * 
     * # Returns
     * The database cleanup job.
     */
    pub fn get_db_cleanup_job(&mut self) -> Result<Job, ApplicationError> {
        let db_cleanup_job = self.clone();
        let job_result = Job::new_async("0 */5 * * * *", move |_uuid, _locked| {
            let db_cleanup_job = db_cleanup_job.clone();
            Box::pin(async move {
                let _ = db_cleanup_job.delete().map_err(|err| {
                    error!("Error checking monitor: {:?}", err);
                });
            })
        });
        job_result.map_err(|err| ApplicationError::new(&format!("Error creating db cleanup job: {err:?}")))        
    }

    /**
     * Delete the old data from the database.
     * 
     * # Returns
     * The result of the database cleanup job.
     */
    fn delete(&self) -> Result<(), ApplicationError> {
        info!("Running db cleanup job");        
        let db_service = self.db_service.as_ref();
        match db_service {
            None => {
                Err(ApplicationError::new("Database service not found"))
            }
            Some(db_service) => {
                db_service.delete_old_data(self.max_time_stored_db)
            }
        }
    }
}