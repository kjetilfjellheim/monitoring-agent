
use log::{debug, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::common::{MonitorStatus, Status};

/**
 * TCP Monitor.
 *
 * This struct represents a TCP monitor.
 *
 * name: The name of the monitor.
 * host: The host to monitor.
 * port: The port to monitor.
 * status: The status of the monitor.
 *
 */
#[derive(Debug, Clone)]
pub struct TcpMonitor {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
}

impl TcpMonitor {
    /**
     * Create a new TCP monitor.
     *
     * host: The host to monitor.
     * port: The port to monitor.
     * name: The name of the monitor.
     * status: The status of the monitor.
     *
     */
    pub fn new(
        host: &str,
        port: u16,
        name: &str,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
    ) -> TcpMonitor {
        debug!("Creating TCP monitor: {}", &name);

        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(Status::Unknown));
            }
            Err(err) => {
                error!("Error creating command monitor: {:?}", err);
            }
        }

        TcpMonitor {
            name: name.to_string(),
            host: host.to_string(),
            port,
            status: status.clone(),
        }
    }

    /**
     * Close the connection.
     *
     * `tcp_stream`: The TCP stream to close.
     *
     */
    fn close_connection(tcp_stream: &std::net::TcpStream) {
        match tcp_stream.shutdown(std::net::Shutdown::Both) {
            Ok(()) => {}
            Err(err) => {
                error!("Error closing connection: {:?}", err);
            }
        }
    }

    /**
     * Set the status of the monitor.
     *
     * status: The status to set.
     *
     */
    fn set_status(&mut self, status: &Status) {
        match self.status.lock() {
            Ok(mut monitor_lock) => {
                debug!(
                    "Setting monitor status for {} to: {:?}",
                    &self.name, &status
                );
                let Some(monitor_status) = monitor_lock.get_mut(&self.name) else {
                    error!("Monitor status not found for: {}", &self.name);
                    return;
                };
                monitor_status.set_status(status);
            }
            Err(err) => {
                error!("Error updating monitor status: {:?}", err);
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
    pub fn check(&mut self) {
        debug!("Checking monitor: {}", &self.name);
        match std::net::TcpStream::connect(format!("{}:{}", &self.host, &self.port)) {
            Ok(tcp_stream) => {
                TcpMonitor::close_connection(&tcp_stream);
                self.set_status(&Status::Ok);
            }
            Err(err) => {
                self.set_status(&Status::Error {
                    message: format!(
                        "Error connecting to {}:{} with error: {err}",
                        &self.host, &self.port,
                    ),
                });
            }
        }
        debug!("Monitor checked: {}", &self.name);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    /**
     * Test the check method. Testing toward Netbios port 139.
     */
    #[tokio::test]
    async fn test_check_port_139() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 139, "localhost", &status);
        monitor.check();
        assert_eq!(
            status.lock().unwrap().get("localhost").unwrap().status,
            Status::Ok
        );
    }

    /**
     * Test the check method. Testing toward port 65000.
     */
    #[tokio::test]
    async fn test_check_port_65000() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 65000, "localhost", &status);
        monitor.check();
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error connecting to localhost:65000 with error: Connection refused (os error 111)".to_string() });
    }

    /**
     * Test the `set_status` method.
     */
    #[test]
    fn test_set_status() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 65000, "localhost", &status);
        monitor.set_status(&Status::Ok);
        assert_eq!(
            status.lock().unwrap().get("localhost").unwrap().status,
            Status::Ok
        );
    }
}
