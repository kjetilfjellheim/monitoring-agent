use log::info;
use log::{debug, error};
use tokio_cron_scheduler::Job;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::common::configuration::DatabaseStoreLevel;
use crate::common::{ApplicationError, MonitorStatus, Status};
use crate::services::DbService;

use super::Monitor;


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
    /// The name of the monitor.
    pub name: String,
    /// The host to monitor.
    pub host: String,
    /// The port of the host monitor.
    pub port: u16,
    /// The status of the monitor.
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
    /// The database service.
    database_service: Arc<Option<DbService>>,
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
    pub fn new(
        host: &str,
        port: u16,
        name: &str,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        database_service: &Arc<Option<DbService>>,
        database_store_level: &DatabaseStoreLevel,
    ) -> TcpMonitor {
        debug!("Creating TCP monitor: {}", &name);
        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(name.to_string(), MonitorStatus::new(name.to_string(), Status::Unknown));
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
        match tcp_stream.shutdown(std::net::Shutdown::Both) {
            Ok(()) => {}
            Err(err) => {
                error!("Error closing connection: {:?}", err);
            }
        }
    }

        /**
     * Get a TCP monitor job.
     *
     * `schedule`: The schedule.
     * `name`: The name of the monitor.
     * `host`: The host to monitor.
     * `port`: The port to monitor.
     *
     * `result`: The result of getting the TCP monitor job.
     */
    pub fn get_tcp_monitor_job(
        &mut self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating Tcp monitor: {}", &self.name);
        let tcp_monitor = self.clone();
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {
            let tcp_monitor = tcp_monitor.clone();
            Box::pin(async move {
                TcpMonitor::run_scheduled(tcp_monitor.clone()).await;
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
        match std::net::TcpStream::connect(format!("{}:{}", &self.host, &self.port)) {
            Ok(tcp_stream) => {
                TcpMonitor::close_connection(&tcp_stream);
                self.set_status(&Status::Ok).await;
            }
            Err(err) => {
                info!("Monitor status error: {} - {}", &self.name, err);
                self.set_status(&Status::Error {
                    message: format!(
                        "Error connecting to {}:{} with error: {err}",
                        &self.host, &self.port,
                    ),
                }).await;
            }
        }
    }    

    /**
     * Run the monitor.
     * 
     * `tcp_monitor`: The TCP monitor.
     */
    async fn run_scheduled(mut tcp_monitor: TcpMonitor) {
        tcp_monitor.check().await;
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
    fn get_status(&self) -> Arc<Mutex<HashMap<String, MonitorStatus>>> {
        self.status.clone()
    }

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> Arc<Option<DbService>> {
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
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 139, "localhost", &status, &Arc::new(None), &DatabaseStoreLevel::None);
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
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 65000, "localhost", &status, &Arc::new(None), &DatabaseStoreLevel::None);
        monitor.check().await;
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error connecting to localhost:65000 with error: Connection refused (os error 111)".to_string() });
    }

    /**
     * Test the `set_status` method.
     */
    #[tokio::test]
    async fn test_set_status() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new("localhost", 65000, "localhost", &status, &Arc::new(None), &DatabaseStoreLevel::None);
        monitor.set_status(&Status::Ok).await;
        assert_eq!(
            status.lock().unwrap().get("localhost").unwrap().status,
            Status::Ok
        );
    }

    #[test]
    fn test_get_tcp_monitor_job() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = TcpMonitor::new(
            "localhost",
            65000,
            "localhost",
            &status,
            &Arc::new(None),
            &DatabaseStoreLevel::None,
        );
        let job = monitor.get_tcp_monitor_job("0 0 * * * *");
        assert!(job.is_ok());
    }      
}
