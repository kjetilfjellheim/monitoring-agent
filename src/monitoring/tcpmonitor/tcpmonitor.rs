use std::sync::Arc;
use std::sync::Mutex;
use log::{ debug, error };

use crate::common::ApplicationError;
use crate::common::MonitorStatus;

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
        debug!("Creating TCP monitor: {}", &name);
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
     * host: The host to monitor.
     * port: The port to monitor.
     * 
     */
    pub async fn check(&mut self) -> Result<(), ApplicationError> {
        debug!("Checking monitor: {}", &self.name);
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
        debug!("Monitor checked: {}", &self.name);
        Ok(())
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
        let mut monitor = TcpMonitor::new(
            "localhost",
            &139,
            "localhost",
        );
        monitor.check().await.unwrap();
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Ok);
    }

    /**
     * Test the check method. Testing toward port 65000.
     */
    #[tokio::test]
    async fn test_check_port_65000() {
        let mut monitor = TcpMonitor::new(
            "localhost",
            &65000,
            "localhost",
        );
        monitor.check().await.unwrap();
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Error { message: "Error connecting to localhost:65000 with error: Connection refused (os error 111)".to_string() });
    }    

    /**
     * Test the set_status method.
     */
    #[test]
    fn test_set_status() {
        let mut monitor = TcpMonitor::new(
            "localhost",
            &65000,
            "localhost",
        );
        monitor.set_status(MonitorStatus::Ok);
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Ok);
    }

}