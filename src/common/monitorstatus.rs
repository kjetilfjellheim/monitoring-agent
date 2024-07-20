use serde::{Deserialize, Serialize};

/**
 * MonitorStatus struct
 * 
 * This struct is used to represent the status of a monitor. It contains the following fields:
 * - status: The status of the monitor
 * - last_successful_time: The last time the monitor was successful
 * - last_error: The last error message
 * - last_error_time: The last time the monitor encountered an error
 * 
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub status: Status,
    pub last_successful_time: Option<std::time::SystemTime>,
    pub last_error: Option<String>,
    pub last_error_time: Option<std::time::SystemTime>,
}

impl MonitorStatus {
    pub fn new(status: Status) -> MonitorStatus {
        MonitorStatus {
            status,
            last_successful_time: None,
            last_error: None,
            last_error_time: None,
        }
    }

    pub fn set_status(&mut self, status: &Status) {
        match status {
            Status::Error { message } => {
                self.last_error_time = Some(std::time::SystemTime::now());
                self.last_error = Some(message.clone());
            }
            Status::Ok => {
                self.last_successful_time = Some(std::time::SystemTime::now());
            }
            _ => {}
        }
        self.status = status.clone();
    }

}
/**
 * MonitorStatus enum
 * 
 * This enum is used to represent the status of a monitor. It can be one of the following:
 * - Ok: The monitor is working correctly
 * - Unknown: The monitor status is unknown
 * - Error: The monitor has encountered an error. The error message is stored in the message field
 * 
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Unknown,
    Error { message: String },
}