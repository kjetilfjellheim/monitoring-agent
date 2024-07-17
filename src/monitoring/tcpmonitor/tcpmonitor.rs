use std::sync::Arc;
use std::sync::Mutex;

use crate::common::ApplicationError;
use crate::monitoring::monitoring::MonitorStatus;

#[derive(Debug, Clone)]
pub struct TcpMonitor {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: Arc<Mutex<MonitorStatus>>
}

impl TcpMonitor {
    pub fn new(host: &str, port: &u16, name: &str) -> TcpMonitor {
        TcpMonitor {
            name: name.to_string(),
            host: host.to_string(),
            port: port.clone(),
            status: Arc::new(Mutex::new(MonitorStatus::Unknown))
        }
    }
    
    fn close_connection(tcp_stream: std::net::TcpStream) {
        match tcp_stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => { },
            Err(err) => {
                eprint!("Error closing connection: {:?}", err);
            }
        }
    }
    
    fn set_status(&mut self, status: MonitorStatus) {
        match self.status.lock() {
            Ok(mut monitor_status) => {
                *monitor_status = status;
            },
            Err(err) => {
                eprintln!("Error updating monitor status: {:?}", err);
            }
        }
    }

    pub async fn check(&mut self, host: &str, port: &u16) -> Result<(), ApplicationError> {
        match std::net::TcpStream::connect(format!("{}:{}", host, port)) {
            Ok(tcp_stream) => {
                TcpMonitor::close_connection(tcp_stream);   
                self.set_status(MonitorStatus::Ok);           
            },
            Err(err) => {
                self.set_status(MonitorStatus::Error { message: format!("Error connecting to {}:{} with error: {}", host, port, err) });
            }
        }
        Ok(())
    }

}
