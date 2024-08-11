use chrono::{DateTime, Utc};

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
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorStatus {
    /// The name of the monitor.
    pub name: String,
    /// The status of the monitor.
    pub status: Status,
    /// The last time the monitor was successful.
    pub last_successful_time: Option<DateTime<Utc>>,
    /// The last error message.
    pub last_error: Option<String>,
    /// The last time the monitor encountered an error.
    pub last_error_time: Option<DateTime<Utc>>,
}

impl MonitorStatus {
    /**
     * Create a new `MonitorStatus`.
     *
     * `status`: The status of the monitor.
     *
     */
    pub fn new(name: String, status: Status) -> MonitorStatus {
        MonitorStatus {
            name,
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
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// The monitor is working correctly.
    Ok,
    /// The monitor status is unknown.
    Unknown,
    /// The monitor has encountered an error. The error message is stored in the message field.
    Error { message: String },
}

impl Status {
    /**
     * Get the maximum status from a list of statuses.
     * Ok < Error (Unknown is ignored)
     *
     * `statuses`: The list of statuses.
     *
     */
    pub fn get_max_status(statuses: Vec<Status>) -> Status {
        let mut max_status = Status::Ok;
        for status in statuses {
            if let Status::Error { .. } = status {
                max_status = status;
            }                
        }
        max_status
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_monitorstatus_new() {
        let name = "test_monitor";
        let status = Status::Ok;
        let monitorstatus = MonitorStatus::new(name.to_string(), status.clone());
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
        let mut monitorstatus = MonitorStatus::new(name.to_string(), status.clone());
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
        assert!(monitorstatus.last_error_time.is_some());
    }

    #[test]
    fn test_status_get_max_status() {
        let statuses = vec![Status::Ok, Status::Error { message: "test error".to_string() }];
        let max_status = Status::get_max_status(statuses);
        assert_eq!(max_status, Status::Error { message: "test error".to_string() });

        let statuses = vec![Status::Ok, Status::Unknown];
        let max_status = Status::get_max_status(statuses);
        assert_eq!(max_status, Status::Ok);
    }

}