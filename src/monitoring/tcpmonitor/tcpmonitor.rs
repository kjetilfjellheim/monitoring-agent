use std::sync::Arc;
use std::sync::Mutex;

use tokio_cron_scheduler::Job;

use crate::common::ApplicationError;
use crate::monitoring::monitoring::MonitorTrait;
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
    
    fn check(host: &str, port: &u16) -> MonitorStatus {
        match std::net::TcpStream::connect(format!("{}:{}", host, port)) {
            Ok(tcp_stream) => {
                TcpMonitor::close_connection(tcp_stream);   
                MonitorStatus::Ok           
            },
            Err(err) => {
                MonitorStatus::Error { message: format!("Error connecting to {}:{} with error: {}", host, port, err) }
            }
        }
    }
}

impl MonitorTrait for TcpMonitor {

    fn get_job(&mut self, schedule: &str) -> Result<Job, ApplicationError> {
        println!("Creating TCP monitor {}:{} job...", self.host, self.port);
        let host = self.host.clone();
        let port = self.port.clone();
        let status = self.status.clone();
        match Job::new(schedule, move |_uuid,_locked| {
            let new_status = TcpMonitor::check(&host, &port);
            match status.lock() {
                Ok(mut monitor_status) => {
                    *monitor_status = new_status;
                },
                Err(err) => {
                    eprintln!("Error updating monitor status: {:?}", err);
                }
            }
        }) {
            Ok(job) => {
                return Ok(job);
            },
            Err(err) => {
                return Err(ApplicationError::new(format!("Could not create job: {}", err).as_str()));
            }
        };
    }    
    
    fn get_status(&self) -> MonitorStatus {
        match self.status.lock() {
            Ok(status) => {
                return status.clone();
            },
            Err(err) => {
                eprintln!("Error getting monitor status: {:?}", err);
                return MonitorStatus::Unknown;
            }
        }
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }
}
