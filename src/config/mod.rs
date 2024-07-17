/**
 * Configuration module
 */
mod args;
mod monitor;

pub use crate::config::args::ApplicationArguments;
pub use crate::config::monitor::{HttpMethod, Monitor, MonitorType, MonitoringConfig};
