/**
 * MonitorStatus enum
 * 
 * This enum is used to represent the status of a monitor. It can be one of the following:
 * - Ok: The monitor is working correctly
 * - Unknown: The monitor status is unknown
 * - Error: The monitor has encountered an error. The error message is stored in the message field
 * 
 */
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorStatus {
    Ok,
    Unknown,
    Error { message: String },
}