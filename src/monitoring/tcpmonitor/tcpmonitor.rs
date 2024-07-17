use std::sync::Arc;
use std::sync::Mutex;

use crate::common::ApplicationError;
use crate::monitoring::monitoring::MonitorStatus;

/**
 * TCP Monitor.
 * 
 * This struct represents a TCP monitor.
 * 
 * name: The name of the monitor.
 * host: The host to monitor.
 * port: The port to monitor.
 * status: The status of the monitor.
 
 */
#[derive(Debug, Clone)]
pub struct TcpMonitor {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: Arc<Mutex<MonitorStatus>>,
}

impl TcpMonitor {
    /**
     * Create a new TCP monitor.
     * 
     * host: The host to monitor.
     * port: The port to monitor.
     * name: The name of the monitor.
     * 
     */
    pub fn new(host: &str, port: &u16, name: &str) -> TcpMonitor {
        TcpMonitor {
            name: name.to_string(),
            host: host.to_string(),
            port: port.clone(),
            status: Arc::new(Mutex::new(MonitorStatus::Unknown)),
        }
    }

    /**
     * Close the connection.
     * 
     * tcp_stream: The TCP stream to close.
     * 
     */
    fn close_connection(tcp_stream: std::net::TcpStream) {
        match tcp_stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => {}
            Err(err) => {
                eprint!("Error closing connection: {:?}", err);
            }
        }
    }

    /**
     * Set the status of the monitor.
     * 
     * status: The status to set.
     * 
     */
    fn set_status(&mut self, status: MonitorStatus) {
        match self.status.lock() {
            Ok(mut monitor_status) => {
                *monitor_status = status;
            }
            Err(err) => {
                eprintln!("Error updating monitor status: {:?}", err);
            }
        }
    }

    /**
     * Check the monitor.
     * 
     * host: The host to monitor.
     * port: The port to monitor.
     * 
     */
    pub async fn check(&mut self) -> Result<(), ApplicationError> {
        match std::net::TcpStream::connect(format!("{}:{}", &self.host, &self.port)) {
            Ok(tcp_stream) => {
                TcpMonitor::close_connection(tcp_stream);
                self.set_status(MonitorStatus::Ok);
            }
            Err(err) => {
                self.set_status(MonitorStatus::Error {
                    message: format!("Error connecting to {}:{} with error: {}", &self.host, &self.port, err),
                });
            }
        }
        Ok(())
    }
}
