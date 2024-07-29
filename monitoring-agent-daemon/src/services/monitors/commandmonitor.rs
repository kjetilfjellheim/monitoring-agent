use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{debug, error, info};

use crate::{common::{configuration::DatabaseStoreLevel, ApplicationError, MonitorStatus, Status}, services::MariaDbService};
use crate::services::monitors::Monitor;

/**
 * Command Monitor.
 *
 * This struct represents a command monitor. It is used to monitor the output of a command.
 *
 */
#[derive(Debug, Clone)]
pub struct CommandMonitor {
    /// The name of the monitor.
    pub name: String,
    /// The command to run on the system.
    pub command: String,
    /// The arguments to the command.
    pub args: Option<Vec<String>>,
    /// The expected output of the command. Used to check if the command ran successfully.
    pub expected: Option<String>,
    /// The current status of the monitor.
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
    /// The database service.
    database_service: Arc<Option<MariaDbService>>,   
    /// The database store level.
    database_store_level: DatabaseStoreLevel, 
}

impl CommandMonitor {
    /**
     * Create a new command monitor.
     *
     * name: The name of the monitor.
     * command: The command to run.
     * args: The arguments to the command.
     * expected: The expected output of the command.
     * status: The status of the monitor.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     *
     * Returns: A new command monitor.
     * 
     */
    pub fn new(
        name: &str,
        command: &str,
        args: Option<Vec<String>>,
        expected: Option<String>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        database_service: &Arc<Option<MariaDbService>>,
        database_store_level: DatabaseStoreLevel
    ) -> CommandMonitor {
        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(Status::Unknown));
            }
            Err(err) => {
                error!("Error creating command monitor: {:?}", err);
            }
        }

        CommandMonitor {
            name: name.to_string(),
            command: command.to_string(),
            args,
            expected,
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level,
        }
    } 

    /**
     * Check the monitor.
     * 
     * This method runs the command and checks the output.
     * 
     * Returns: Ok if the monitor ran successfully, an error otherwise.
     *
     */
    pub async fn check(&mut self) -> Result<(), ApplicationError> {
        debug!("Checking monitor: {}", &self.name);
        let mut command = tokio::process::Command::new(&self.command);
        let command = match &self.args {
            Some(args) => command.args(args),
            None => &mut command,
        };
        let command_result = command.output().await;
        match command_result {
            Ok(output) => {
                let output_resp = String::from_utf8_lossy(&output.stdout);
                debug!("Command output: {}", output_resp);
                if self.is_command_success(&output, &output_resp)
                {
                    self.set_status(&Status::Ok);
                } else {
                    info!("Monitor status error: {} - {:?}", &self.name, output);
                    self.set_status(&Status::Error {
                        message: format!("Error running command: {output:?}"),
                    });
                }
                Ok(())
            }
            Err(err) => {
                info!("Monitor status error: {} - {:?}", &self.name, err);
                self.set_status(&Status::Error {
                    message: format!("Error running command: {err:?}"),
                });
                Err(ApplicationError::new(&format!(
                    "Error running command: {err:?}"
                )))
            }
        }        
    }

    /**
     * Check if the command ran successfully.
     *
     * `output`: The output of the command.
     * `output_resp`: The response of the command.
     *
     * Returns true if the command ran successfully, false otherwise.
     */
    fn is_command_success(&mut self, output: &std::process::Output, output_resp: &str) -> bool {
            output.status.success()
                && (self.expected.is_none()
                    || (self.expected.is_some()
                        && self
                            .expected
                            .as_ref()
                            .unwrap_or(&String::new())
                            .eq(output_resp)))
    }
}


/**
 * Implement the `Monitor` trait for `HttpMonitor`.
 */
impl super::Monitor for CommandMonitor {
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
    fn get_database_service(&self) -> Arc<Option<MariaDbService>> {
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
    use std::os::unix::process::ExitStatusExt;

    use super::*;

    /**
     * Test the check method. Testing success commandL.
     */
    #[tokio::test]
    async fn test_check_ls() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new("test", "ls", None, None, &status, &Arc::new(None), DatabaseStoreLevel::None);
        monitor.check().await.unwrap();
        assert_eq!(
            status.lock().unwrap().get("test").unwrap().status,
            Status::Ok
        );
    }

    /**
     * Test the check method. Testing success commandL.
     */
    #[tokio::test]
    async fn test_check_systemctl() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new(
            "test",
            "systemctl",
            Some(vec!["status".to_string(), "dbus.service".to_string()]),
            None,
            &status,
            &Arc::new(None), 
            DatabaseStoreLevel::None
        );
        monitor.check().await.unwrap();
        assert_eq!(
            status.lock().unwrap().get("test").unwrap().status,
            Status::Ok
        );
    }

    /**
     * Test the check method. Testing non existing command.
     */
    #[tokio::test]
    async fn test_check_non_existing_command() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new("test", "grumpy", None, None, &status, &Arc::new(None), DatabaseStoreLevel::None);
        let _ = monitor.check().await;
        assert_eq!(status.lock().unwrap().get("test").unwrap().status, Status::Error { message: "Error running command: Os { code: 2, kind: NotFound, message: \"No such file or directory\" }".to_string() });
    }

    /**
     * Test the check method. Testing systemctl srvice status.
     */
    #[tokio::test]
    async fn test_check_systemctl_service_is_active_command() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new(
            "test",
            "systemctl",
            Some(vec![
                "show".to_string(),
                "dbus.service".to_string(),
                "--property=ActiveState".to_string(),
            ]),
            Some("ActiveState=active\n".to_string()),
            &status,
            &Arc::new(None),
            DatabaseStoreLevel::None
        );
        let _ = monitor.check().await;
        assert_eq!(
            status.lock().unwrap().get("test").unwrap().status,
            Status::Ok
        );
    }

    #[test]
    fn test_is_command_success_exitstatus_0() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new("test", "ls", None, None, &status, &Arc::new(None), DatabaseStoreLevel::None);
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert_eq!(monitor.is_command_success(&output, ""), true);
    }

    #[test]
    fn test_is_command_success_exitstatus_1() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new("test", "ls", None, None, &status, &Arc::new(None), DatabaseStoreLevel::None);
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert_eq!(monitor.is_command_success(&output, ""), false);
    }
}
