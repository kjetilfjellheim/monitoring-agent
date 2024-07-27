use std::{collections::HashMap, future::Future, sync::{Arc, Mutex}, time::{Duration, Instant}};

use log::{debug, error, info};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::common::{configuration::MonitoringConfig, ApplicationError, HttpMethod, MonitorStatus};

use super::monitors::{CommandMonitor, HttpMonitor, TcpMonitor};


pub struct SchedulingService {
    scheduler: Option<JobScheduler>,
    tcp_monitors: Vec<TcpMonitor>,
    http_monitors: Vec<HttpMonitor>,
    command_monitors: Vec<CommandMonitor>,    
    monitoring_config: MonitoringConfig,
}

impl SchedulingService {

    /**
     * Create a new scheduling service.
     *
     * result: The result of creating the scheduling service.
     */
    pub fn new(monitoring_config: &MonitoringConfig) -> SchedulingService {
        SchedulingService {
            scheduler: None,
            tcp_monitors: Vec::new(),
            http_monitors: Vec::new(),
            command_monitors: Vec::new(),
            monitoring_config: monitoring_config.clone(),
        }
    }

   /**
     * Start the monitoring service.
     *
     * `test`: Test the configuration. Starts the scheduling, but stops immediately.
     *
     * result: The result of starting the monitoring service.
     */
    pub async fn start(&mut self, test: bool) -> Result<(), ApplicationError> {       
        /*
         * Start the scheduling of the monitoring jobs.
         */
        let future_scheduling = self.add_jobs();
        /*
         * Block the main thread until the scheduling is done. If test is true, the scheduling will be ignored.
         * This is useful for testing the configuration file and for testing the code.
         */
        if !test {
            future_scheduling.await?;
        }
        Ok(())
    }

    /**
     * Create and add jobs to the scheduler.
     *
     * result: The result of adding the jobs to the scheduler.
     *
     * throws: `ApplicationError`: If the jobs fails to be added.
     */
    async fn add_jobs(
        &mut self,
    ) -> Result<(), ApplicationError> {
        /*
         * Create a new job scheduler.
         */
        let scheduler: JobScheduler = match JobScheduler::new().await {
            Ok(scheduler) => scheduler,
            Err(err) => {
                return Err(ApplicationError::new(
                    format!("Could not create scheduler: {err}").as_str(),
                ));
            }
        };
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        /*
         * Create and add jobs to the scheduler.
         */
        let monitors = self.monitoring_config.clone().monitors;
        for monitor in monitors {
            self.create_and_add_job(&monitor, &scheduler, &status)
                .await?;
        }                 
        /*
         * Start the scheduler.
         */
        match scheduler.start().await {
            Ok(()) => {
                self.scheduler = Some(scheduler);
            }
            Err(err) => {
                return Err(ApplicationError::new(
                    format!("Could not start scheduler: {err}").as_str(),
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
     * `monitor`: The monitor configuration.
     * `scheduler`: The job scheduler.
     *
     * `result`: The result of creating and adding the job to the scheduler.
     *
     * throws: `ApplicationError`: If the job fails to be added.
     */
    async fn create_and_add_job(
        &mut self,
        monitor: &crate::common::Monitor,
        scheduler: &JobScheduler,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
    ) -> Result<(), ApplicationError> {
        let monitor_type = monitor.details.clone();
        let job = match monitor_type {
            crate::common::MonitorType::Tcp { host, port } => self.get_tcp_monitor_job(
                monitor.schedule.as_str(),
                monitor.name.as_str(),
                host.as_str(),
                port,
                status,
            )?,
            crate::common::MonitorType::Http {
                url,
                method,
                body,
                headers,
                use_builtin_root_certs,
                accept_invalid_certs,
                tls_info,
                root_certificate,
                identity,
                identity_password,
            } => self.get_http_monitor_job(
                monitor.schedule.as_str(),
                monitor.name.as_str(),
                url.as_str(),
                method,
                &body,
                &headers,
                use_builtin_root_certs,
                accept_invalid_certs,
                tls_info,
                root_certificate,
                identity,
                identity_password,
                status,
            )?,
            crate::common::MonitorType::Command {
                command,
                args,
                expected,
            } => self.get_command_monitor_job(
                monitor.schedule.as_str(),
                monitor.name.as_str(),
                &command,
                &args,
                &expected,
                status,
            )?,
        };
        self.add_job(scheduler, job).await?;
        Ok(())
    }

    /**
     * Log the monitors.
     */
    fn log_monitors(&self) {
        info!("Logging monitors {:?}", Instant::now());
        for tcp_monitor in &*self.tcp_monitors {
            SchedulingService::log_tcp_monitor(tcp_monitor);
        }
        for http_monitor in &*self.http_monitors {
            SchedulingService::log_http_monitor(http_monitor);
        }
        for command_monitor in &*self.command_monitors {
            SchedulingService::log_command_monitor(command_monitor);
        }
    }

    /**
     * Add a job to the scheduler.
     *
     * `scheduler`: The job scheduler.
     * `job`: The job to add.
     *
     * `result`: The result of adding the job to the scheduler.
     *
     * throws: `ApplicationError`: If the job fails to be added.
     */
    async fn add_job(&self, scheduler: &JobScheduler, job: Job) -> Result<(), ApplicationError> {
        match scheduler.add(job).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ApplicationError::new(
                format!("Could not add job: {err}").as_str(),
            )),
        }
    }      

    fn get_command_monitor_job(
        &mut self,
        schedule: &str,
        name: &str,
        command: &str,
        args: &Option<Vec<String>>,
        expected: &Option<String>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
    ) -> Result<Job, ApplicationError> {
        debug!("Creating Command monitor: {}", &name);
        let command_monitor =
            CommandMonitor::new(name, command, args.clone(), expected.clone(), status);
        self.command_monitors.push(command_monitor.clone());
        match Job::new_async(schedule, move |_uuid, _locked| {
            SchedulingService::check_command_monitor(&command_monitor)
        }) {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {err}").as_str(),
            )),
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
    fn get_tcp_monitor_job(
        &mut self,
        schedule: &str,
        name: &str,
        host: &str,
        port: u16,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
    ) -> Result<Job, ApplicationError> {
        debug!("Creating Tcp monitor: {}", &name);
        let tcp_monitor = TcpMonitor::new(host, port, name, status);
        self.tcp_monitors.push(tcp_monitor.clone());
        match Job::new_async(schedule, move |_uuid, _locked| {
            SchedulingService::check_tcp_monitor(&tcp_monitor)
        }) {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {err}").as_str(),
            )),
        }
    }

    /**
     * Get an HTTP monitor job.
     *
     * `schedule`: The schedule.
     * `name`: The name of the monitor.
     * `url`: The URL to monitor.
     * `method`: The HTTP method.
     * `body`: The body.
     * `headers`: The headers.
     *
     * result: The result of getting the HTTP monitor job.
     *
     * throws: `ApplicationError`: If the job fails to be created.
     */
    #[allow(clippy::too_many_arguments)]
    fn get_http_monitor_job(
        &mut self,
        schedule: &str,
        name: &str,
        url: &str,
        method: HttpMethod,
        body: &Option<String>,
        headers: &Option<std::collections::HashMap<String, String>>,
        use_builtin_root_certs: bool,
        accept_invalid_certs: bool,
        tls_info: bool,
        root_certificate: Option<String>,
        identity: Option<String>,
        identity_password: Option<String>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
    ) -> Result<Job, ApplicationError> {
        debug!("Creating http monitor: {}", &name);
        let http_monitor = HttpMonitor::new(
            url,
            method,
            body,
            headers,
            name,
            use_builtin_root_certs,
            accept_invalid_certs,
            tls_info,
            root_certificate,
            identity,
            identity_password,
            status,
        )?;
        self.http_monitors.push(http_monitor.clone());
        match Job::new_async(schedule, move |_uuid, _locked| {
            SchedulingService::check_http_monitor(&http_monitor)
        }) {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {err}").as_str(),
            )),
        }
    }

    /**
     * Log the HTTP monitor.
     *
     * `http_monitor`: The HTTP monitor.
     */
    fn log_http_monitor(http_monitor: &HttpMonitor) {
        let lock = http_monitor.status.lock();
        match lock {
            Ok(lock) => {
                info!("Job {}: Status: {:?}", http_monitor.name, lock);
            }
            Err(err) => {
                error!("Error getting lock: {:?}", err);
            }
        }
    }

    /**
     * Log the TCP monitor.
     *
     * `tcp_monitor`: The TCP monitor.
     */
    fn log_tcp_monitor(tcp_monitor: &TcpMonitor) {
        let lock = tcp_monitor.status.lock();
        match lock {
            Ok(lock) => {
                info!("Job {}: Status: {:?}", tcp_monitor.name, lock);
            }
            Err(err) => {
                error!("Error getting lock: {:?}", err);
            }
        }
    }

    /**
     * Log the Command monitor.
     *
     * `command_monitor`: The command monitor.
     */
    fn log_command_monitor(command_monitor: &CommandMonitor) {
        let lock = command_monitor.status.lock();
        match lock {
            Ok(lock) => {
                info!("Job {}: Status: {:?}", command_monitor.name, lock);
            }
            Err(err) => {
                error!("Error getting lock: {:?}", err);
            }
        }
    }

    /**
     * Check the HTTP monitor.
     *
     * `http_monitor`: The HTTP monitor.
     *
     * result: Future of the check.
     */
    fn check_http_monitor(
        http_monitor: &HttpMonitor,
    ) -> std::pin::Pin<Box<impl Future<Output = ()>>> {
        Box::pin({
            let mut moved_http_monitor = http_monitor.clone();
            async move {
                let _ = moved_http_monitor
                    .check()
                    .await
                    .map_err(|err| error!("Error: {}", err.message));
            }
        })
    }

    /**
     * Check the TCP monitor.
     *
     * `tcp_monitor`: The TCP monitor.
     *
     * result: Future of the check.
     */
    fn check_tcp_monitor(tcp_monitor: &TcpMonitor) -> std::pin::Pin<Box<impl Future<Output = ()>>> {
        Box::pin({
            let mut moved_tcp_monitor = tcp_monitor.clone();
            async move {
                let () = moved_tcp_monitor.check();
            }
        })
    }

    /**
     * Check the command monitor.
     *
     * `command_monitor`: The command monitor.
     *
     * result: Future of the check.
     */
    fn check_command_monitor(
        command_monitor: &CommandMonitor,
    ) -> std::pin::Pin<Box<impl Future<Output = ()>>> {
        Box::pin({
            let mut moved_command_monitor = command_monitor.clone();
            async move {
                let _ = moved_command_monitor
                    .check()
                    .await
                    .map_err(|err| error!("Error: {}", err.message));
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
    #[tokio::test]
    async fn test_monitoring_service() {
        let mut scheduling_service = SchedulingService::new(&MonitoringConfig::new("./resources/test/test_full_integration_test.json").unwrap());
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with a tcp monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_tcp() {
        let mut scheduling_service = SchedulingService::new(&MonitoringConfig::new("./resources/test/test_simple_tcp.json").unwrap());
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an http monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_http() {
        let mut scheduling_service = SchedulingService::new(&MonitoringConfig::new("./resources/test/test_simple_http.json").unwrap());
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an command monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_command() {
        let mut scheduling_service = SchedulingService::new(&MonitoringConfig::new("./resources/test/test_simple_command.json").unwrap());
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

}

