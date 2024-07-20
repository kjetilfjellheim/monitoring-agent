use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::{debug, error};

use crate::common::{ApplicationError, MonitorStatus, Status};
/**
 * Command Monitor.
 * 
 * This struct represents a command monitor. It is used to monitor the output of a command.
 *
 */
#[derive(Debug, Clone)]
pub struct CommandMonitor {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub expected: Option<String>,
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
}

impl CommandMonitor {
    /**
     * Create a new command monitor.
     * 
     * name: The name of the monitor.
     * command: The command to run.
     * args: The arguments to the command.
     * expected: The expected output of the command.
     * 
     */
    pub fn new(name: &str, command: &str, args: Option<Vec<String>>, expected: Option<String>, status: Arc<Mutex<HashMap<String, MonitorStatus>>>) -> CommandMonitor {
        status.lock().unwrap().insert(name.to_string(), MonitorStatus::new(Status::Unknown));

        CommandMonitor {
            name: name.to_string(),
            command: command.to_string(),
            args: args,
            expected: expected,
            status: status
        }
    }

    /**
     * Set the status of the monitor.
     * 
     * status: The new status.
     * 
     */
    fn set_status(&mut self, status: Status) {
        match self.status.lock() {
            Ok(mut monitor_lock) => {
                debug!("Setting monitor status for {} to: {:?}", &self.name, &status);
                let monitor_status = monitor_lock.get_mut(&self.name).unwrap();
                monitor_status.set_status(&status);
            }
            Err(err) => {
                error!("Error updating monitor status: {:?}", err);
            }
        }
    }

    /**
     * Check the monitor.
     * 
     */
    pub async fn check(
        &mut self
    ) -> Result<(), ApplicationError> {
        debug!("Checking monitor: {}", &self.name);
        let mut command = tokio::process::Command::new(&self.command);
        let command = match &self.args {
            Some(args) => command.args(args),
            None => &mut command
        };
        let command_result = command.output()
            .await
            .and_then(|output| {
                let output_resp = String::from_utf8_lossy(&output.stdout);
                debug!("Command output: {}", output_resp);
                if output.status.success() && self.expected.is_none() {                                      
                    self.set_status(Status::Ok);
                } else if output.status.success() && self.expected.is_some() && self.expected.as_ref().unwrap().eq(&output_resp) {
                    self.set_status(Status::Ok);
                } else {
                    self.set_status(Status::Error {
                        message: format!("Error running command: {:?}", output),
                    });
                }
                Ok(())
            });
        match command_result {
            Ok(_) => return Ok(()),
            Err(err) => {
                self.set_status(Status::Error {
                    message: format!("Error running command: {:?}", err),
                });
                return Err(ApplicationError::new(&format!("Error running command: {:?}", err)));
            }
        };            
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /**
     * Test the check method. Testing success commandL.
     */
    #[tokio::test]
    async fn test_check_() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new(
            "test",
            "ls",
            None,
            None,
            status.clone()            
        );
        monitor.check().await.unwrap();
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Ok);
    }   

    /**
     * Test the check method. Testing success commandL.
     */
    #[tokio::test]
    async fn test_check_systemctl() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new(
            "test",
            "systemctl",            
            Some(vec!["status".to_string(), "dbus.service".to_string()]),
            None,
            status.clone()
        );
        monitor.check().await.unwrap();
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Ok);
    }       

    /**
     * Test the check method. Testing non existing command.
     */
    #[tokio::test]
    async fn test_check_non_existing_command() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new(
            "test",
            "grumpy",
            None,
            None,
            status.clone()        
        );
        let _ = monitor.check().await;
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error running command: Os { code: 2, kind: NotFound, message: \"No such file or directory\" }".to_string() });
    }  
    
    /**
     * Test the check method. Testing systemctl srvice status.
     */
    #[tokio::test]
    async fn test_check_systemctl_service_is_active_command() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = CommandMonitor::new(
            "test",
            "systemctl",
            Some(vec!["show".to_string(), "dbus.service".to_string(), "--property=ActiveState".to_string()]),
            Some("ActiveState=active\n".to_string()),
            status.clone()                         
        );
        let _ = monitor.check().await;
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Ok);
    }  

       

}