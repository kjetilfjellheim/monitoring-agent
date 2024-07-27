use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/**
 * `MonitorStatus` struct
 * 
 *  This struct is used to represent the status of a monitor in the service modules. It contains the following fields:
 * - `status`: The status of the monitor
 * - `last_successful_time`: The last time the monitor was successful
 * - `last_error`: The last error message
 * - `last_error_time`: The last time the monitor encountered an error
 *
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorStatus {
    /// The status of the monitor.
    #[serde(rename = "status")]
    pub status: Status,
    /// The last time the monitor was successful.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastSuccessfulTime")]
    pub last_successful_time: Option<DateTime<Utc>>,
    /// The last error message.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastError")]
    pub last_error: Option<String>,
    /// The last time the monitor encountered an error.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastErrorTime")]
    pub last_error_time: Option<DateTime<Utc>>,
}

impl MonitorStatus {
    /**
     * Create a new `MonitorStatus`.
     *
     * `status`: The status of the monitor.
     *
     */
    pub fn new(status: Status) -> MonitorStatus {
        MonitorStatus {
            status,
            last_successful_time: None,
            last_error: None,
            last_error_time: None,
        }
    }

    /**
     * Set the status of the monitor.
     *
     * `status`: The new status.
     *
     */
    pub fn set_status(&mut self, status: &Status) {
        match status {
            Status::Error { message } => {
                self.last_error_time = Some(chrono::Utc::now());
                self.last_error = Some(message.clone());
            }
            Status::Ok => {
                self.last_successful_time = Some(chrono::Utc::now());
            }
            Status::Unknown => {}
        }
        self.status = status.clone();
    }
}

/**
 * `MonitorStatus` enum
 *
 * This enum is used to represent the status of a monitor. It can be one of the following:
 * - Ok: The monitor is working correctly
 * - Unknown: The monitor status is unknown
 * - Error: The monitor has encountered an error. The error message is stored in the message field
 *
 */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    /// The monitor is working correctly.
    Ok,
    /// The monitor status is unknown.
    Unknown,
    /// The monitor has encountered an error. The error message is stored in the message field.
    Error { message: String },
}
