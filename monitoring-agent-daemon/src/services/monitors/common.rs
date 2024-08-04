use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{debug, error};

use crate::{common::{configuration::DatabaseStoreLevel, MonitorStatus, Status}, services::DbService};

pub trait Monitor {
    
    /**
     * Get the name of the monitor.
     *
     * Returns: The name of the monitor.
     */
    fn get_name(&self) -> &str;

    /**
     * Get the status of the monitor.
     *
     * Returns: The status of the monitor.
     */
    fn get_status(&self) -> Arc<Mutex<HashMap<String, MonitorStatus>>>;

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> Arc<Option<DbService>>;

    /**
     * Get the database store level.
     *
     * Returns: The database store level.
     */
    fn get_database_store_level(&self) -> DatabaseStoreLevel;

    /**
     * Set the status of the monitor.
     *
     * `new_status`: The new status.
     *
     */
    async fn set_status(&mut self, new_status: &Status) {
        self.insert_monitor_status(new_status).await;
        let status = self.get_status();
        match status.lock() {
            Ok(mut monitor_lock) => {
                debug!(
                    "Setting monitor status for {} to: {:?}",
                    self.get_name(), &status
                );
                let Some(monitor_status) = monitor_lock.get_mut(self.get_name()) else {
                    error!("Monitor status not found for: {}", &self.get_name());
                    return;
                };
                monitor_status.set_status(new_status);
            }
            Err(err) => {
                error!("Error updating monitor status: {:?}", err);
            }
        };
    }

   /**
     * Insert the monitor status into the database.
     *
     * status: The status to insert.
     *
     */
    async fn insert_monitor_status(&mut self, status: &Status) {
        match self.get_database_store_level() {
            DatabaseStoreLevel::None => {
                return;
            }
            DatabaseStoreLevel::Errors => {
                if status == &Status::Ok || status == &Status::Unknown {
                    return;
                }
            }
            DatabaseStoreLevel::All => {
                // Continue                           
            }
        }
        let database_service = self.get_database_service();
        if database_service.is_some() {
            let database_service = database_service.as_ref();
            if database_service.is_some() {
                let database_service = database_service.as_ref().unwrap();
                match database_service.insert_monitor_status(
                    self.get_name(),
                    &status.clone(),
                ).await {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Error inserting monitor status: {:?}", err);
                    }
                }
            }
        }
    }  
}