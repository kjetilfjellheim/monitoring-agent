/**
 * Modules for monitoring services. 
 * 
 * `commandmonitor`: Monitor that runs a command and checks the output.
 * `httpmonitor`: Monitor that checks the status of an HTTP service.
 * `tcpmonitor`: Monitor that checks the status of a TCP service. 
 */
mod commandmonitor;
mod httpmonitor;
mod tcpmonitor;

pub use commandmonitor::CommandMonitor;
pub use httpmonitor::HttpMonitor;
pub use tcpmonitor::TcpMonitor;
