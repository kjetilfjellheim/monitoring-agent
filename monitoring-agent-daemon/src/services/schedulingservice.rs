use std::{collections::HashMap, sync::Arc, time::Duration};

use log::info;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{common::{configuration::{MonitoringConfig, ThresholdLevel}, ApplicationError, DatabaseServiceType, MonitorStatusType}, services::jobs::NotificationJob};
use crate::services::{DbService, jobs::DbCleanupJob};
use super::monitors::{CertificateMonitor, CommandMonitor, DatabaseMonitor, HttpMonitor, LoadAvgMonitor, MeminfoMonitor, ProcessMonitor, SystemctlMonitor, TcpMonitor};

/**
 * Scheduling Service.
 * 
 * This struct represents the scheduling service.
 * 
 * `scheduler`: The job scheduler.
 * `monitoring_config`: The monitoring configuration.
 * `status`: The status of the monitors.
 * `database_service`: The database service.
 * `server_name`: The server name.
 * 
 */
pub struct SchedulingService {
    /// The job scheduler. Handles starting the jobs.
    scheduler: Option<JobScheduler>,
    /// The monitoring configuration.
    monitoring_config: Arc<MonitoringConfig>,
    /// The status of the monitors.
    status: MonitorStatusType,
    /// The database service.
    database_service: DatabaseServiceType,
    /// The server name.
    server_name: String,
}

impl SchedulingService {

    /**
     * Create a new scheduling service.
     *
     * result: The result of creating the scheduling service.
     */
    pub fn new(server_name: &str, monitoring_config: &Arc<MonitoringConfig>, status: &MonitorStatusType, database_service: &DatabaseServiceType) -> SchedulingService {
        SchedulingService {
            scheduler: None,
            monitoring_config: monitoring_config.clone(),
            status: status.clone(),
            database_service: database_service.clone(),
            server_name: server_name.to_string(),
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
        info!("Creating job scheduler");
        let scheduler: JobScheduler = match JobScheduler::new().await {
            Ok(scheduler) => scheduler,
            Err(err) => {
                return Err(ApplicationError::new(
                    format!("Could not create scheduler: {err}").as_str(),
                ));
            }
        };
        /*
         * Create and add jobs to the scheduler.
         */
        for monitor in self.monitoring_config.monitors.clone() {
            self.create_and_add_job(&monitor, &scheduler).await?;
        }

        /*
         * Create a cleanup job.
         */
        let cleanup_config = self.monitoring_config.cleanup_config.clone();
        if let Some(cleanup_config) = cleanup_config {
            let max_time_stored_db = cleanup_config.max_time_stored_db;
            if let Some(max_time_stored_db) = max_time_stored_db {
                let mut cleanup_job = DbCleanupJob::new(&self.database_service, max_time_stored_db);
                let job = cleanup_job.get_db_cleanup_job()?;
                self.add_job(&scheduler, job).await?;
            }
        }
        /*
         * Create a notification job.
         */
        let notification_config = self.monitoring_config.notification_config.clone();
        if let Some(notification_config) = notification_config {
            let mut notification = NotificationJob::new(&self.status, &notification_config.url, notification_config.recipients, notification_config.from.as_str(), notification_config.reply_to.as_str(), notification_config.resend_after, &notification_config.schedule)?;
            let job = notification.get_notification_job()?;
            self.add_job(&scheduler, job).await?;
        }        
        /*
         * Start the scheduler.
         */
        info!("Starting scheduler");
        match scheduler.start().await {
            Ok(()) => {
                info!("Scheduler started");
                self.scheduler = Some(scheduler);
            }
            Err(err) => {
                return Err(ApplicationError::new(
                    format!("Could not start scheduler: {err}").as_str(),
                ));
            }
        }
        loop {
            info!("Scheduler is awake");
            tokio::time::sleep(Duration::from_secs(300)).await;
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
    ) -> Result<(), ApplicationError> {
        let monitor_type = monitor.details.clone();
        match monitor_type {
            crate::common::MonitorType::Tcp { host, port, retry } => {
                self.create_and_schedule_tcp_monitor(host, port, retry, monitor, scheduler).await?
            },
            crate::common::MonitorType::Http { url, method, body, headers, use_builtin_root_certs, accept_invalid_certs,
                tls_info, root_certificate, identity, identity_password, retry} => { 
                self.create_and_schedule_http_monitor(url, method, body, headers, monitor, use_builtin_root_certs, accept_invalid_certs, tls_info, root_certificate, identity, identity_password, retry, scheduler).await?
            },
            crate::common::MonitorType::Command { command, args, expected, } => {
                self.create_and_schedule_command_monitor(monitor, command, args, expected, scheduler).await?
            },
            crate::common::MonitorType::LoadAvg { threshold_1min, threshold_5min, threshold_15min, threshold_1min_level, threshold_5min_level, threshold_15min_level, store_values, } => {               
                self.create_and_schedule_loadavg_monitor(monitor, threshold_1min, threshold_5min, threshold_15min, threshold_1min_level, threshold_5min_level, threshold_15min_level, store_values, scheduler).await?
            },
            crate::common::MonitorType::Mem {error_percentage_used_mem, error_percentage_used_swap, warn_percentage_used_mem, warn_percentage_used_swap, store_values } => {
                self.create_and_schedule_memory_monitor(monitor, error_percentage_used_mem, error_percentage_used_swap, warn_percentage_used_mem, warn_percentage_used_swap, store_values, scheduler).await?
            },
            crate::common::MonitorType::Systemctl { active 
            } => {
                self.create_and_schedule_systemctl_monitor(monitor, active, scheduler).await?
            },
            crate::common::MonitorType::Database {database_config, max_query_time, } => {
                self.create_and_schedule_database_monitor(monitor, max_query_time, database_config, scheduler).await?
            },
            crate::common::MonitorType::Process { application_names, pids, regexp, threshold_mem_warn, threshold_mem_error, store_values } => {
                self.create_and_schedule_process_monitor(monitor, application_names, pids, regexp, threshold_mem_warn, threshold_mem_error, store_values, scheduler).await?
            },
            crate::common::MonitorType::Certificate { certificates, threshold_days_warn, threshold_days_error } => {
                self.create_and_schedule_certificate_monitor(monitor, certificates, threshold_days_warn, threshold_days_error, scheduler).await?
            },
        }?;   
        Ok(()) 
    }

    /**
     * Create and schedule a certificate monitor.
     * 
     * `monitor`: The monitor configuration.
     * `certificates`: The certificates.
     * `threshold_days_warn`: The threshold days warning.
     * `threshold_days_error`: The threshold days error.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the certificate monitor.
     * 
     * Errors:
     * - If the certificate monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    async fn create_and_schedule_certificate_monitor(&mut self, monitor: &crate::common::Monitor, certificates: Vec<String>, threshold_days_warn: u32, threshold_days_error: u32, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let certificate_monitor = CertificateMonitor::new(
            &monitor.name,
            &monitor.description,
            &self.status,
            certificates,
            threshold_days_warn,
            threshold_days_error,
            &self.database_service.clone(),
            &monitor.store,
        );
        let job = CertificateMonitor::get_certificate_job(certificate_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a process monitor.
     * 
     * `monitor`: The monitor configuration.
     * `application_names`: The application names.
     * `pids`: The process IDs.
     * `regexp`: The regular expression.
     * `threshold_mem_warn`: The threshold memory warning.
     * `threshold_mem_error`: The threshold memory error.
     * `store_values`: Store the values.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the process monitor.
     * 
     * Errors:
     * - If the process monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    #[allow(clippy::too_many_arguments)]
    async fn create_and_schedule_process_monitor(&mut self, monitor: &crate::common::Monitor, application_names: Option<Vec<String>>, pids: Option<Vec<u32>>, regexp: Option<String>, threshold_mem_warn: Option<u64>, threshold_mem_error: Option<u64>, store_values: bool, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let process_monitor = ProcessMonitor::new(&monitor.name, &monitor.description, application_names, pids, regexp, threshold_mem_warn, threshold_mem_error, &self.status, &self.database_service.clone(), &monitor.store, store_values);
        let job = ProcessMonitor::get_process_monitor_job(process_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a database monitor.
     * 
     * `monitor`: The monitor configuration.
     * `max_query_time`: The maximum query time.
     * `database_config`: The database configuration.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the database monitor.
     * 
     * Errors:
     * - If the database monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    async fn create_and_schedule_database_monitor(&mut self, monitor: &crate::common::Monitor, max_query_time: Option<u32>, database_config: Option<crate::common::DatabaseConfig>, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let database_monitor = DatabaseMonitor::new(
            &monitor.name,
            &monitor.description,
            max_query_time,
            &self.status,
            &self.get_database_service(&self.database_service, &database_config).await?,
            &monitor.store,
        );
        let job = DatabaseMonitor::get_database_monitor_job(database_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a systemctl monitor.
     * 
     * `monitor`: The monitor configuration.
     * `active`: The active services.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the systemctl monitor.
     * 
     * Errors:
     * - If the systemctl monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    async fn create_and_schedule_systemctl_monitor(&mut self, monitor: &crate::common::Monitor, active: Vec<String>, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let systemctl_monitor = SystemctlMonitor::new(&monitor.name, &monitor.description, &self.status, &self.database_service.clone(), &monitor.store, active);
        let job = SystemctlMonitor::get_systemctl_monitor_job(systemctl_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a memory use monitor.
     * 
     * `monitor`: The monitor configuration.
     * `error_percentage_used_mem`: The maximum percentage of memory.
     * `error_percentage_used_swap`: The maximum percentage of swap.
     * `warn_percentage_used_mem`: The warn percentage of memory.
     * `warn_percentage_used_swap`: The warn percentage of swap.
     * `store_values`: Store the values.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the memory use monitor.
     * 
     * Errors:
     * - If the memory use monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    #[allow(clippy::too_many_arguments)]
    async fn create_and_schedule_memory_monitor(&mut self, monitor: &crate::common::Monitor, error_percentage_used_mem: Option<f64>, error_percentage_used_swap: Option<f64>, warn_percentage_used_mem: Option<f64>, warn_percentage_used_swap: Option<f64>, store_values: bool, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let meminfo_monitor = MeminfoMonitor::new(&monitor.name, &monitor.description, error_percentage_used_mem, error_percentage_used_swap, warn_percentage_used_mem, warn_percentage_used_swap, &self.status, &self.database_service.clone(), &monitor.store, store_values);
        let job = MeminfoMonitor::get_meminfo_monitor_job(meminfo_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a loadavg monitor.
     * 
     * `monitor`: The monitor configuration.
     * `threshold_1min`: The threshold for the 1 minute load average.
     * `threshold_5min`: The threshold for the 5 minute load average.
     * `threshold_15min`: The threshold for the 15 minute load average.
     * `store_values`: Store the values.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the loadavg monitor.
     * 
     * Errors:
     * - If the loadavg monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    #[allow(clippy::too_many_arguments)]    
    #[allow(clippy::similar_names)]
    async fn create_and_schedule_loadavg_monitor(&mut self, monitor: &crate::common::Monitor, threshold_1min: Option<f32>, threshold_5min: Option<f32>, threshold_15min: Option<f32>, threshold_1min_level: ThresholdLevel, threshold_5min_level: ThresholdLevel, threshold_15min_level: ThresholdLevel, store_values: bool, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let loadavg_monitor = LoadAvgMonitor::new(&monitor.name, &monitor.description, threshold_1min, threshold_5min, threshold_15min, threshold_1min_level, threshold_5min_level, threshold_15min_level, &self.status, &self.database_service.clone(), &monitor.store, store_values);
        let job = LoadAvgMonitor::get_loadavg_monitor_job(loadavg_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a command monitor.
     * 
     * `monitor`: The monitor configuration.
     * `command`: The command to run.
     * `args`: The arguments to the command.
     * `expected`: The expected output of the command.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the command monitor.
     * 
     * Errors:
     * - If the command monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    async fn create_and_schedule_command_monitor(&mut self, monitor: &crate::common::Monitor, command: String, args: Option<Vec<String>>, expected: Option<String>, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let command_monitor = CommandMonitor::new(&monitor.name, &monitor.description, command.as_str(), args, expected, &self.status, &self.database_service.clone(), &monitor.store);
        let job = CommandMonitor::get_command_monitor_job(command_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }

    /**
     * Create and schedule an HTTP monitor.
     * 
     * `url`: The URL to monitor.
     * `method`: The HTTP method.
     * `body`: The body of the request.
     * `headers`: The headers of the request.
     * `monitor`: The monitor configuration.
     * `use_builtin_root_certs`: Use the builtin root certificates.
     * `accept_invalid_certs`: Accept invalid certificates.
     * `tls_info`: Get TLS information.
     * `root_certificate`: The root certificate.
     * `identity`: The identity.
     * `identity_password`: The identity password.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the HTTP monitor.
     * 
     * Errors:
     * - If the HTTP monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    #[allow(clippy::too_many_arguments)]
    async fn create_and_schedule_http_monitor(&mut self, url: String, method: crate::common::HttpMethod, body: Option<String>, headers: Option<HashMap<String, String>>, monitor: &crate::common::Monitor, use_builtin_root_certs: bool, accept_invalid_certs: bool, tls_info: bool, root_certificate: Option<String>, identity: Option<String>, identity_password: Option<String>, retry: Option<u16>, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let http_monitor = HttpMonitor::new(
            url.as_str(),
            method,
            &body,
            &headers,
            &monitor.name,
            &monitor.description.clone(),
            use_builtin_root_certs,
            accept_invalid_certs,
            tls_info,
            root_certificate,
            identity,
            identity_password,
            retry,
            &self.status,
            &self.database_service.clone(),
            &monitor.store,
        )?;
        let job = HttpMonitor::get_http_monitor_job(http_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Create and schedule a TCP monitor.   
     * 
     * `host`: The host to monitor.
     * `port`: The port to monitor.
     * `monitor`: The monitor configuration.
     * `scheduler`: The job scheduler.
     * 
     * `result`: The result of creating and scheduling the TCP monitor.
     * 
     * Errors:
     * - If the TCP monitor fails to be created.
     * - If the job fails to be added.
     * - If the job fails to be scheduled.
     */
    async fn create_and_schedule_tcp_monitor(&self, host: String, port: u16, retry: Option<u16>, monitor: &crate::common::Monitor, scheduler: &JobScheduler) -> Result<Result<(), ApplicationError>, ApplicationError> {
        let tcp_monitor = TcpMonitor::new(host.as_str(), port, retry, &monitor.name, &monitor.description, &self.status.clone(), &self.database_service.clone(), &monitor.store);
        let job = TcpMonitor::get_tcp_monitor_job(tcp_monitor, monitor.schedule.as_str())?;
        Ok(self.add_job(scheduler, job).await)
    }
    
    /**
     * Get the database service.
     *
     * `database_service`: The database service.
     * `database_config`: The database configuration.
     *
     * `result`: The result of getting the database service.
     *
     * throws: `ApplicationError`: If the database service fails to be created.
     */
    async fn get_database_service(&self, database_service: &DatabaseServiceType, database_config: &Option<crate::common::DatabaseConfig>) -> Result<DatabaseServiceType, ApplicationError> {
        match database_config {
            Some(database_config) => {
                let database_service = DbService::new(database_config, &self.server_name).await?;
                Ok(Arc::new(Some(database_service)))
            },
            None => Ok(database_service.clone()),
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

}


#[cfg(test)]
mod test {

    use std::sync::Arc;

    use crate::common::configuration::{DatabaseStoreLevel, ThresholdLevel};

    use super::*;

    /**
     * Test the monitoring service with both tcp monitors and http monitors.
     */
    #[tokio::test]
    async fn test_monitoring_service() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/test_full_configuration.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with a tcp monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_tcp() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_tcp.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an http monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_http() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_http.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an command monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_command() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_command.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an loadavg monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_loadavg() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_loadavg.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an memory use monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_meminfo() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_meminfo.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an systemd monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_systemctl() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_systemctl.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an mariadb monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_db_mariadb() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_db_mariadb.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an postgres monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_db_postgres() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_db_postgres.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }    

    #[tokio::test]
    async fn test_add_jobs() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_tcp.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.add_jobs().await;
        print!("{:?}", res);
    }

    #[tokio::test]
    async fn test_create_and_add_job_tcp_job() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_tcp.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            description: None,
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Tcp {
                host: "localhost".to_string(),
                port: 80,
                retry: None,
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_http_job() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_http.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            description: None,
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Http {
                url: "http://localhost".to_string(),
                method: crate::common::HttpMethod::Get,
                body: Some("".to_string()),
                headers: Some(HashMap::new()),
                use_builtin_root_certs: false,
                accept_invalid_certs: false,
                tls_info: false,
                root_certificate: None,
                identity: None,
                identity_password: None,
                retry: None
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_systemctl_job() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_systemctl.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            description: None,
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Systemctl { 
                active: vec!["ssh".to_string()],
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_command_job() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_command.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            description: None,
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Command {
                command: "ls".to_string(),
                args: Some(vec!["-l".to_string()]),
                expected: Some("total".to_string()),
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_loadavg_job() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_loadavg.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            description: None,
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::LoadAvg { 
                threshold_1min: Some(0.0),
                threshold_5min: Some(0.0),
                threshold_15min: Some(0.0),
                threshold_1min_level: ThresholdLevel::Warn,
                threshold_5min_level: ThresholdLevel::Warn,
                threshold_15min_level: ThresholdLevel::Warn,
                store_values: false,
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_meminfo_job() {
        let status = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &Arc::new(MonitoringConfig::new("resources/test/configuration_import_test/test_simple_meminfo.json").unwrap()), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            description: None,
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Mem {
                error_percentage_used_mem: Some(0.0),
                error_percentage_used_swap: Some(0.0),
                warn_percentage_used_mem: Some(0.0),
                warn_percentage_used_swap: Some(0.0),                
                store_values: false,
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }    

}

