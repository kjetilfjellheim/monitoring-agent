use log::info;
use log::{debug, error};
use tokio_cron_scheduler::Job;

use crate::common::configuration::DatabaseStoreLevel;
use crate::common::{ApplicationError, DatabaseServiceType, MonitorStatus, MonitorStatusType, Status};

use super::Monitor;


/**
 * TCP Monitor.
 *
 * This struct represents a TCP monitor.
 *
 * name: The name of the monitor.
 * description: The description of the monitor.
 * host: The host to monitor.
 * port: The port to monitor.
 * status: The status of the monitor.
 *
 */
#[derive(Debug, Clone)]
pub struct TcpMonitor {
    /// The name of the monitor.
    pub name: String,  
    /// The host to monitor.
    pub host: String,
    /// The port of the host monitor.
    pub port: u16,
    /// Number of retries if error occurs.
    retry: Option<u16>,
    /// The status of the monitor.
    pub status: MonitorStatusType,
    /// The database service.
    database_service: DatabaseServiceType,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: &str,
        port: u16,
        retry: Option<u16>,
        name: &str,
        description: &Option<String>,
        status: &MonitorStatusType,
        database_service: &DatabaseServiceType,
        database_store_level: &DatabaseStoreLevel,
    ) -> TcpMonitor {
        debug!("Creating TCP monitor: {}", &name);
        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name, description, Status::Unknown));
            }
            Err(err) => {
                error!("Error creating command monitor: {:?}", err);
            }
        }

        TcpMonitor {
            name: name.to_string(),
            host: host.to_string(),
            port,
            retry,
            status: status.clone(),
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
        }
    }

    /**
     * Close the connection.
     *
     * `tcp_stream`: The TCP stream to close.
     *
     */
    fn close_connection(tcp_stream: &std::net::TcpStream) {
        let _ = tcp_stream.shutdown(std::net::Shutdown::Both).map_err(|err| error!("Error closing connection: {:?}", err));
    }

    /**
     * Get a TCP monitor job.
     *
     * `tcp_monitor`: The TCP monitor.
     * `schedule`: The schedule.
     *
     * `result`: The result of getting the TCP monitor job.
     */
    pub fn get_tcp_monitor_job(
        tcp_monitor: Self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating Tcp monitor: {}", tcp_monitor.name);
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {
            Box::pin({
                let mut tcp_monitor = tcp_monitor.clone();
                async move {
                    tcp_monitor.check().await;
                }
            })              
        });        
        match job_result {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {err}").as_str(),
            )),
        }
    }

    /**
     * Check the monitor.
     */
    async fn check(&mut self) {
        debug!("Checking monitor: {}", &self.name);
        let status = self.connect();
        self.set_status(&status).await;       
    }    

    /**
     * Connect with retries.
     * 
     * 
     */
    fn connect(&self) -> Status {
        let tcp_stream = std::net::TcpStream::connect(format!("{}:{}", &self.host, &self.port));
        
        let mut current_err = match tcp_stream {
            Ok(tcp_stream) => {
                TcpMonitor::close_connection(&tcp_stream);
                return Status::Ok;
            },
            Err(err) => {
                Status::Error { message: format!("Error connection to {}:{}. Error: {err:?}", self.host, self.port) }
            },
        };
        
        if let Some(retry) = self.retry {
            for index in 1..=retry {
                let tcp_stream = std::net::TcpStream::connect(format!("{}:{}", &self.host, &self.port));
                match tcp_stream {
                    Ok(tcp_stream) => {
                        TcpMonitor::close_connection(&tcp_stream);
                        return Status::Warn { message: format!("Success after retries {index}. Previous err: {current_err:?}") };
                    },
                    Err(err) => {
                        current_err = Status::Error { message: format!("Error connection to {}:{} after {index} retries. Error: {err:?}", self.host, self.port) };
                    },
                };
            }
        } 
        current_err
    }

}

/**
 * Implement the `Monitor` trait for `TcpMonitor`.
 */
impl super::Monitor for TcpMonitor {
    /**
     * Get the name of the monitor.
     *
     * Returns: The name of the monitor.
     */
    fn get_name(&self) -> &str {
        &self.name
    }

    /**
     * Get the status of the monitor.
     *
     * Returns: The status of the monitor.
     */
    fn get_status(&self) -> MonitorStatusType {
        self.status.clone()
    }

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> DatabaseServiceType {
        self.database_service.clone()
    }

    /**
     * Get the database store level.
     *
     * Returns: The database store level.
     */
    fn get_database_store_level(&self) -> DatabaseStoreLevel {
        self.database_store_level.clone()
    }    
 
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::services::monitors::Monitor;

    /**
     * Test the check method. Testing toward Netbios port 139.
     */
    #[ignore = "This keeps failing during build actions in Github, temporarily disabled."]
    #[tokio::test]
    async fn test_check_port_139() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 139, None, "localhost", &None, &status, &std::sync::Arc::new(None), &DatabaseStoreLevel::None);
        monitor.check().await;
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
        let status: MonitorStatusType =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 65000, None, "localhost", &None, &status, &std::sync::Arc::new(None), &DatabaseStoreLevel::None);
        monitor.check().await;
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error connection to localhost:65000. Error: Os { code: 111, kind: ConnectionRefused, message: \"Connection refused\" }".to_string() });
    }

    /**
     * Test the `set_status` method.
     */
    #[tokio::test]
    async fn test_set_status() {
        let status: MonitorStatusType =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 65000, None, "localhost", &None, &status, &std::sync::Arc::new(None), &DatabaseStoreLevel::None);
        monitor.set_status(&Status::Ok).await;
        assert_eq!(
            status.lock().unwrap().get("localhost").unwrap().status,
            Status::Ok
        );
    }

    #[test]
    fn test_get_tcp_monitor_job() {
        let status: MonitorStatusType =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let monitor = TcpMonitor::new(
            "localhost",
            65000,
            None,
            "localhost",
            &None,
            &status,
            &std::sync::Arc::new(None),
            &DatabaseStoreLevel::None,
        );
        let job = TcpMonitor::get_tcp_monitor_job(monitor, "0 0 * * * *");
        assert!(job.is_ok());
    }      
}
