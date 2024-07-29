/**
 * This module contains the services that are used by the monitoring agent daemon.
 * 
 * `monitors`: The monitors contains monitors used to verify the status of the system.
 * `monitoringservice`: Handles the web service requests.
 * `schedulingservice`: Handles the scheduling of the monitoring tasks.
 * `databaseservice`: Handles the database operations.
 */
mod monitors;
mod monitoringservice;
mod schedulingservice;
mod databaseservice;

pub use monitoringservice::MonitoringService;
pub use schedulingservice::SchedulingService;
pub use databaseservice::MariaDbService;

