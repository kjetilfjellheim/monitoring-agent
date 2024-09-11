use log::{debug, error};

use crate::common::{configuration::DatabaseStoreLevel, DatabaseServiceType, MonitorStatusType, Status};

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
    fn get_status(&self) -> MonitorStatusType;

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> DatabaseServiceType;

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
                    self.get_name(), &new_status
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_monitorstatus_new() {
        let name = "test_monitor";
        let status = Status::Ok;
        let monitorstatus = crate::common::MonitorStatus::new(name, &None, status.clone());
        assert_eq!(monitorstatus.name, name);
        assert_eq!(monitorstatus.status, status);
        assert_eq!(monitorstatus.last_successful_time, None);
        assert_eq!(monitorstatus.last_error, None);
        assert_eq!(monitorstatus.last_error_time, None);
    }

    #[test]
    fn test_monitorstatus_set_status() {
        let name = "test_monitor";
        let status = Status::Ok;
        let mut monitorstatus = crate::common::MonitorStatus::new(name, &None, status.clone());
        monitorstatus.set_status(&status);
        assert_eq!(monitorstatus.status, status);
        assert!(monitorstatus.last_successful_time.is_some());
        assert_eq!(monitorstatus.last_error, None);
        assert_eq!(monitorstatus.last_error_time, None);

        let status = Status::Error {
            message: "test error".to_string(),
        };
        monitorstatus.set_status(&status);
        assert_eq!(monitorstatus.status, status);
        assert!(monitorstatus.last_successful_time.is_some());
        assert_eq!(monitorstatus.last_error, Some("test error".to_string()));
    }

}