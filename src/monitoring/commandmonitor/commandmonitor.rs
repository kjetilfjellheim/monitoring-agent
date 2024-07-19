use std::sync::{Arc, Mutex};

use log::{debug, error};

use crate::common::{ApplicationError, MonitorStatus};
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
    pub status: Arc<Mutex<MonitorStatus>>,
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
    pub fn new(name: &str, command: &str, args: Option<Vec<String>>, expected: Option<String>) -> CommandMonitor {
        CommandMonitor {
            name: name.to_string(),
            command: command.to_string(),
            args: args,
            expected: expected,
            status: Arc::new(Mutex::new(MonitorStatus::Unknown))
        }
    }

    /**
     * Set the status of the monitor.
     * 
     * status: The new status.
     * 
     */
    fn set_status(&mut self, status: MonitorStatus) {
        match self.status.lock() {
            Ok(mut monitor_status) => {
                debug!("Setting monitor status for {} to: {:?}", &self.name, &status);
                *monitor_status = status;
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
                    self.set_status(MonitorStatus::Ok);
                } else if output.status.success() && self.expected.is_some() && self.expected.as_ref().unwrap().eq(&output_resp) {
                    self.set_status(MonitorStatus::Ok);
                } else {
                    self.set_status(MonitorStatus::Error {
                        message: format!("Error running command: {:?}", output),
                    });
                }
                Ok(())
            });
        match command_result {
            Ok(_) => return Ok(()),
            Err(err) => {
                self.set_status(MonitorStatus::Error {
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
        let mut monitor = CommandMonitor::new(
            "test",
            "ls",
            None,
            None            
        );
        monitor.check().await.unwrap();
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Ok);
    }   

    /**
     * Test the check method. Testing success commandL.
     */
    #[tokio::test]
    async fn test_check_systemctl() {
        let mut monitor = CommandMonitor::new(
            "test",
            "systemctl",            
            Some(vec!["status".to_string(), "dbus.service".to_string()]),
            None
        );
        monitor.check().await.unwrap();
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Ok);
    }       

    /**
     * Test the check method. Testing non existing command.
     */
    #[tokio::test]
    async fn test_check_non_existing_command() {
        let mut monitor = CommandMonitor::new(
            "test",
            "grumpy",
            None,
            None           
        );
        let _ = monitor.check().await;
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Error { message: "Error running command: Os { code: 2, kind: NotFound, message: \"No such file or directory\" }".to_string() });
    }  
    
    /**
     * Test the check method. Testing systemctl srvice status.
     */
    #[tokio::test]
    async fn test_check_systemctl_service_is_active_command() {
        let mut monitor = CommandMonitor::new(
            "test",
            "systemctl",
            Some(vec!["show".to_string(), "dbus.service".to_string(), "--property=ActiveState".to_string()]),
            Some("ActiveState=active\n".to_string())                         
        );
        let _ = monitor.check().await;
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Ok);
    }  

       

}