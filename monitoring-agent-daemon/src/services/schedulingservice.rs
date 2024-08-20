use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};

use log::info;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::common::{configuration::MonitoringConfig, ApplicationError, MonitorStatus};
use crate::services::DbService;
use super::monitors::{CommandMonitor, HttpMonitor, LoadAvgMonitor, MeminfoMonitor, SystemctlMonitor, TcpMonitor, DatabaseMonitor, ProcessMonitor};

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
    monitoring_config: MonitoringConfig,
    /// The status of the monitors.
    status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
    /// The database service.
    database_service: Arc<Option<DbService>>,
    /// The server name.
    server_name: String,
}

impl SchedulingService {

    /**
     * Create a new scheduling service.
     *
     * result: The result of creating the scheduling service.
     */
    pub fn new(server_name: &str, monitoring_config: &MonitoringConfig, status: &Arc<Mutex<HashMap<String, MonitorStatus>>>, database_service: &Arc<Option<DbService>>) -> SchedulingService {
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
        let monitors = self.monitoring_config.clone().monitors;
        for monitor in monitors {
            self.create_and_add_job(&monitor, &scheduler).await?;
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
            crate::common::MonitorType::Tcp { host, port } => {
                let mut tcp_monitor = TcpMonitor::new(host.as_str(), port, &monitor.name, &self.status.clone(), &self.database_service.clone(), &monitor.store);
                let job = tcp_monitor.get_tcp_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
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
            } => { 
                let mut http_monitor = HttpMonitor::new(
                    url.as_str(),
                    method,
                    &body,
                    &headers,
                    &monitor.name,
                    use_builtin_root_certs,
                    accept_invalid_certs,
                    tls_info,
                    root_certificate,
                    identity,
                    identity_password,
                    &self.status,
                    &self.database_service.clone(),
                    &monitor.store,
                )?;
                let job = http_monitor.get_http_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
            crate::common::MonitorType::Command {
                command,
                args,
                expected,
            } => {
                let mut command_monitor = CommandMonitor::new(&monitor.name, command.as_str(), args, expected, &self.status, &self.database_service.clone(), &monitor.store);
                let job = command_monitor.get_command_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
            crate::common::MonitorType::LoadAvg {  
                threshold_1min,
                threshold_5min,
                threshold_15min,
                store_values,
            } => {               
                let mut loadavg_monitor = LoadAvgMonitor::new(&monitor.name, threshold_1min, threshold_5min, threshold_15min, &self.status, &self.database_service.clone(), &monitor.store, store_values);
                let job = loadavg_monitor.get_loadavg_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
            crate::common::MonitorType::Mem {max_percentage_mem, max_percentage_swap, store_values
            } => {
                let mut meminfo_monitor = MeminfoMonitor::new(&monitor.name, max_percentage_mem, max_percentage_swap, &self.status, &self.database_service.clone(), &monitor.store, store_values);
                let job = meminfo_monitor.get_meminfo_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
            crate::common::MonitorType::Systemctl { active 
            } => {
                let mut systemctl_monitor = SystemctlMonitor::new(&monitor.name, &self.status, &self.database_service.clone(), &monitor.store, active);
                let job = systemctl_monitor.get_systemctl_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
            crate::common::MonitorType::Database {database_config, max_query_time,
            } => {
                let mut database_monitor = DatabaseMonitor::new(
                    &monitor.name,
                    max_query_time,
                    &self.status,
                    &self.get_database_service(&self.database_service, &database_config).await?,
                    &monitor.store,
                );
                let job = database_monitor.get_database_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
            crate::common::MonitorType::Process { application_names, max_mem_usage, store_values 
            } => {
                let mut process_monitor = ProcessMonitor::new(&monitor.name, &application_names, max_mem_usage, &self.status, &self.database_service.clone(), &monitor.store, store_values);
                let job = process_monitor.get_process_monitor_job(monitor.schedule.as_str())?;
                self.add_job(scheduler, job).await
            },
        }?;   
        Ok(()) 
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
    async fn get_database_service(&self, database_service: &Arc<Option<DbService>>, database_config: &Option<crate::common::DatabaseConfig>) -> Result<Arc<Option<DbService>>, ApplicationError> {
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

    use crate::common::configuration::DatabaseStoreLevel;

    use super::*;

    /**
     * Test the monitoring service with both tcp monitors and http monitors.
     */
    #[tokio::test]
    async fn test_monitoring_service() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/test_full_configuration.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with a tcp monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_tcp() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_tcp.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an http monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_http() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_http.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an command monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_command() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_command.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an loadavg monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_loadavg() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_loadavg.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an memory use monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_meminfo() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_meminfo.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an systemd monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_systemctl() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_systemctl.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an mariadb monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_db_mariadb() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_db_mariadb.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }

    /**
     * Test the monitoring service with an postgres monitor.
     */
    #[tokio::test]
    async fn test_monitoring_service_db_postgres() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("./resources/test/configuration_import_test/test_simple_db_postgres.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.start(true).await;
        assert!(res.is_ok());
    }    

    #[tokio::test]
    async fn test_add_jobs() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_tcp.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.add_jobs().await;
        print!("{:?}", res);
    }

    #[tokio::test]
    async fn test_create_and_add_job_tcp_job() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_tcp.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Tcp {
                host: "localhost".to_string(),
                port: 80,
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_http_job() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_http.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
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
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }


    #[tokio::test]
    async fn test_create_and_add_job_systemctl_job() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_systemctl.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
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
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_command.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
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
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_loadavg.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::LoadAvg { 
                threshold_1min: Some(0.0),
                threshold_5min: Some(0.0),
                threshold_15min: Some(0.0),
                store_values: false,
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_create_and_add_job_meminfo_job() {
        let status = Arc::new(Mutex::new(HashMap::new()));
        let mut scheduling_service = SchedulingService::new("", &MonitoringConfig::new("resources/test/configuration_import_test/test_simple_meminfo.json").unwrap(), &status, &Arc::new(None));
        let res = scheduling_service.create_and_add_job(&crate::common::Monitor {
            name: "test".to_string(),
            schedule: "* * * * * *".to_string(),
            store: DatabaseStoreLevel::None,
            details: crate::common::MonitorType::Mem {
                max_percentage_mem: Some(0.0),
                max_percentage_swap: Some(0.0),
                store_values: false,
            },
        }, &JobScheduler::new().await.unwrap()).await;
        assert!(res.is_ok())
    }    

}

