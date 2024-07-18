use std::time::{ Duration, Instant };

use futures::Future;
use tokio_cron_scheduler::{ Job, JobScheduler };

use crate::common::ApplicationError;
use crate::config::HttpMethod;
use crate::config::MonitoringConfig;
use crate::monitoring::httpmonitor::HttpMonitor;
use crate::monitoring::tcpmonitor::TcpMonitor;

/**
 * Monitoring Service.
 *
 * This struct represents the monitoring service.
 *
 * scheduler: The job scheduler.
 * tcp_monitors: The TCP monitors.
 * http_monitors: The HTTP monitors.
 
 */
pub struct MonitoringService {
    scheduler: Option<JobScheduler>,
    tcp_monitors: Vec<TcpMonitor>,
    http_monitors: Vec<HttpMonitor>,
}

impl MonitoringService {
    /**
     * Create a new monitoring service.
     * 
     * result: The result of creating the monitoring service.
     */
    pub fn new() -> MonitoringService {
        MonitoringService {
            scheduler: None,
            tcp_monitors: Vec::new(),
            http_monitors: Vec::new(),
        }
    }

    /**
     * Start the monitoring service.
     * 
     * config_file: The configuration file.
     * test: Test the configuration. Starts the scheduling, but stops immediately.
     * 
     * result: The result of starting the monitoring service.
     */
    pub fn start(
        &mut self,
        config_file: &str,
        test: &bool
    ) -> Result<(), ApplicationError> {
        /*
         * Load the monitoring configuration.
         */
        let monitoring_config: MonitoringConfig =
            MonitoringConfig::new(&config_file)?;
        /*
         * Start the scheduling of the monitoring jobs.
         */
        let future_scheduling = self.add_jobs(&monitoring_config);
        /*
         * Block the main thread until the scheduling is done.
         */
        if !test {
            match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => {
                    runtime.block_on(future_scheduling)?;
                }
                Err(err) => {
                    return Err(ApplicationError::new(
                        format!("Could not create runtime: {}", err).as_str(),
                    ));
                }
            }
     }
        Ok(())
    }

    /**
     * Create and add jobs to the scheduler.
     * 
     * monitoring_config: The monitoring configuration.
     * 
     * result: The result of adding the jobs to the scheduler.
     * 
     * throws: ApplicationError: If the jobs fails to be added.
     */
    async fn add_jobs(
        &mut self,
        monitoring_config: &MonitoringConfig,
    ) -> Result<(), ApplicationError> {
        /*
         * Create a new job scheduler.
         */
        let scheduler: JobScheduler = match JobScheduler::new().await {
            Ok(scheduler) => scheduler,
            Err(err) => {
                return Err(ApplicationError::new(
                    format!("Could not create scheduler: {}", err).as_str(),
                ));
            }
        };
        for monitor in monitoring_config.monitors.iter() {
            self.create_and_add_job(monitor, &scheduler).await?;
        }
        /*
         * Start the scheduler.
         */
        match scheduler.start().await {
            Ok(_) => {
                self.scheduler = Some(scheduler);
            }
            Err(err) => {
                return Err(ApplicationError::new(
                    format!("Could not start scheduler: {}", err).as_str(),
                ));
            }
        }
        loop {
            self.log_monitors();

            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    }

    /**
     * Create and add a job to the scheduler.
     * 
     * monitor: The monitor configuration.
     * scheduler: The job scheduler.
     * 
     * result: The result of creating and adding the job to the scheduler.
     * 
     * throws: ApplicationError: If the job fails to be added.
     */
    async fn create_and_add_job(
        &mut self,
        monitor: &crate::config::Monitor,
        scheduler: &JobScheduler,
    ) -> Result<(), ApplicationError> {
        let monitor_type = monitor.monitor.clone();
        let job = match monitor_type {
            crate::config::MonitorType::Tcp { host, port } => {
                self.get_tcp_monitor_job(
                    monitor.schedule.as_str(),
                    monitor.name.as_str(),
                    host.as_str(),
                    &port,
                )
                .await?
            },
            crate::config::MonitorType::Http {
                url,
                method,
                body,
                headers,
            } => {
                self.get_http_monitor_job(
                    monitor.schedule.as_str(),
                    monitor.name.as_str(),
                    url.as_str(),
                    &method,
                    &body,
                    &headers,
                )
                .await?
            }
            _ => {
                return Err(ApplicationError::new("Unsupported monitor type"));
            }
        };
        self.add_job(scheduler, job).await?;
        Ok(())
    }

    /**
     * Log the monitors.
     */
    fn log_monitors(&self) {
        println!("Logging monitors {:?}", Instant::now());
        for tcp_monitor in self.tcp_monitors.iter() {
            self.log_tcp_monitor(tcp_monitor);
        }
        for http_monitor in self.http_monitors.iter() {
            self.log_http_monitor(http_monitor);
        }
    }

    /**
     * Add a job to the scheduler.
     * 
     * scheduler: The job scheduler.
     * job: The job to add.
     * 
     * result: The result of adding the job to the scheduler.
     * 
     * throws: ApplicationError: If the job fails to be added.
     */
    async fn add_job(&self, scheduler: &JobScheduler, job: Job) -> Result<(), ApplicationError> {
        match scheduler.add(job).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ApplicationError::new(
                format!("Could not add job: {}", err).as_str(),
            )),
        }
    }

    /**
     * Get a TCP monitor job.
     * 
     * schedule: The schedule.
     * name: The name of the monitor.
     * host: The host to monitor.
     * port: The port to monitor.
     * 
     * result: The result of getting the TCP monitor job.
     */
    async fn get_tcp_monitor_job(
        &mut self,
        schedule: &str,
        name: &str,
        host: &str,
        port: &u16,
    ) -> Result<Job, ApplicationError> {
        let tcp_monitor = TcpMonitor::new(host, port, name);
        self.tcp_monitors.push(tcp_monitor.clone());
        match Job::new_async(schedule, move |_uuid, _locked| {
            MonitoringService::check_tcp_monitor(&tcp_monitor)
        }) {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {}", err).as_str(),
            )),
        }
    }

    /**
     * Get an HTTP monitor job.
     * 
     * schedule: The schedule.
     * name: The name of the monitor.
     * url: The URL to monitor.
     * method: The HTTP method.
     * body: The body.
     * headers: The headers.
     * 
     * result: The result of getting the HTTP monitor job.
     * 
     * throws: ApplicationError: If the job fails to be created.
     */
    async fn get_http_monitor_job(
        &mut self,
        schedule: &str,
        name: &str,
        url: &str,
        method: &HttpMethod,
        body: &Option<String>,
        headers: &Option<std::collections::HashMap<String, String>>,
    ) -> Result<Job, ApplicationError> {
        let http_monitor = HttpMonitor::new(url, &method, body, headers, &name);
        self.http_monitors.push(http_monitor.clone());
        match Job::new_async(schedule, move |_uuid, _locked| {
            MonitoringService::check_http_monitor(&http_monitor)
        }) {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {}", err).as_str(),
            )),
        }
    }

    /**
     * Log the HTTP monitor.
     * 
     * http_monitor: The HTTP monitor.
     */
    fn log_http_monitor(&self, http_monitor: &HttpMonitor) {
        let lock = http_monitor.status.lock();
        match lock {
            Ok(lock) => {
                println!("Job {}: Status: {:?}", http_monitor.name, lock);
            }
            Err(err) => {
                eprintln!("Error getting lock: {:?}", err);
            }
        }
    }

    /**
     * Log the TCP monitor.
     * 
     * tcp_monitor: The TCP monitor.
     */
    fn log_tcp_monitor(&self, tcp_monitor: &TcpMonitor) {
        let lock = tcp_monitor.status.lock();
        match lock {
            Ok(lock) => {
                println!("Job {}: Status: {:?}", tcp_monitor.name, lock);
            }
            Err(err) => {
                eprintln!("Error getting lock: {:?}", err);
            }
        }
    }

    /**
     * Check the HTTP monitor.
     * 
     * http_monitor: The HTTP monitor.
     * 
     */
    fn check_http_monitor(
        http_monitor: &HttpMonitor
    ) -> std::pin::Pin<Box<impl Future<Output = ()>>> {
        Box::pin({
            let mut moved_http_monitor = http_monitor.clone();
            async move {
                let _ = moved_http_monitor
                    .check()
                    .await
                    .map_err(|err| eprintln!("Error: {}", err.message));
            }
        })
    }

    fn check_tcp_monitor(
        tcp_monitor: &TcpMonitor
    ) -> std::pin::Pin<Box<impl Future<Output = ()>>> {
        Box::pin({
            let mut moved_tcp_monitor = tcp_monitor.clone();
            async move {
                let _ = moved_tcp_monitor
                    .check()
                    .await
                    .map_err(|err| eprintln!("Error: {}", err.message));
            }
        })
    }

}

#[cfg(test)]
mod test {

    use super::*;

    /**
     * Test the monitoring service with both tcp monitors and http monitors.
     */
    #[test]
    fn test_monitoring_service() {
        let mut monitoring_service = MonitoringService::new();
        monitoring_service.start("./resources/test/test_full_integration_test.json", &true).unwrap();
    }

    /**
     * Test the monitoring service with a tcp monitor.
     */
    #[test]
    fn test_monitoring_service_tcp() {
        let mut monitoring_service = MonitoringService::new();
        monitoring_service.start("./resources/test/test_simple_tcp.json", &true).unwrap();
    }

    /**
     * Test the monitoring service with an http monitor.
     */
    #[test]
    fn test_monitoring_service_http() {
        let mut monitoring_service = MonitoringService::new();
        monitoring_service.start("./resources/test/test_simple_http.json", &true).unwrap();
    }

    /**
     * Test the monitoring service with an unknown monitor.
     */
    #[test]
    fn test_monitoring_service_unknown() {
        let mut monitoring_service = MonitoringService::new();
        let result = monitoring_service.start("./resources/test/test_simple_unknown.json", &true);
        match result {
            Ok(_) => {
                assert!(false);
            }
            Err(err) => {
                assert_eq!("Could not parse config file: Line 6", err.message);
            }
            
        }
    }

}