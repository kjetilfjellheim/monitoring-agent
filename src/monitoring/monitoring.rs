use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::{ ApplicationArguments, MonitoringConfig };
use crate::common::ApplicationError;
use crate::monitoring::tcpmonitor::TcpMonitor;

/**
 * The main action of the application.
 * 
 * @param args: The application arguments.
 *  
 */
pub struct MonitoringService {
    scheduler: Option<JobScheduler>,
    status: Arc::<Mutex::<HashMap::<String, TcpMonitor>>>
}

impl MonitoringService {
    /**
     * Create a new monitoring service.
     */
    pub fn new() -> MonitoringService {    
        MonitoringService {
            scheduler: None,
            status: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    /**
     * Start the monitoring service.
     * 
     * @param application_arguments: The application arguments.
     * 
     * @return Result<(), ApplicationError>: The result of starting the monitoring service.
     *  If the monitoring service is started successfully, the result is Ok(()).
     * If the monitoring service fails to start, the result is an ApplicationError.
     * 
     * @throws ApplicationError: If the monitoring service fails to start.
     * 
     */
    pub fn start(&mut self, application_arguments: &ApplicationArguments) -> Result<(), ApplicationError> {         
        /*
         * Load the monitoring configuration.
         */
        let monitoring_config: MonitoringConfig = MonitoringConfig::new(&application_arguments.config)?;        
        /*
         * Start the scheduling of the monitoring jobs.
         */
        let future_scheduling = self.add_jobs(&monitoring_config);
        /*
         * Block the main thread until the scheduling is done.
         */
        match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => {
                runtime.block_on(future_scheduling)?;
            },
            Err(err) => {
                return Err(ApplicationError::new(format!("Could not create runtime: {}", err).as_str()));
            }
        }
        Ok(())
    }    
    
    /**
     * Schedule the monitoring jobs.
     * 
     * @param monitoring_config: The monitoring configuration. 
     * The configuration contains the monitors to be scheduled.
     * 
     * @return Result<(), ApplicationError>: The result of the scheduling.
     * If the scheduling is successful, the result is Ok(()).
     * If the scheduling fails, the result is an ApplicationError.
     * 
     * @throws ApplicationError: If the scheduling fails.
     */
    async fn add_jobs(&mut self, monitoring_config: &MonitoringConfig) -> Result<(), ApplicationError> {
        /*
         * Create a new job scheduler.
         */
        let scheduler: JobScheduler = match JobScheduler::new().await {
            Ok(scheduler) => scheduler,
            Err(err) => {
                return Err(ApplicationError::new(format!("Could not create scheduler: {}", err).as_str()));
            }
        };          
        for monitor in monitoring_config.monitors.iter() {
            let monitor_type = monitor.monitor.clone();
            let job = match monitor_type {
                crate::config::MonitorType::Tcp{host, port} => {
                    self.get_tcp_monitor_job(monitor.schedule.as_str(), monitor.name.as_str(), host.as_str(), &port)?                    
                },
                _ => {
                    return Err(ApplicationError::new("Unsupported monitor type"));
                }
            };      
            self.add_job(&scheduler, job).await?;      
        }
        /*
         * Start the scheduler.
         */
        match scheduler.start().await {
            Ok(_) => {
                self.scheduler = Some(scheduler);
            },
            Err(err) => {
                return Err(ApplicationError::new(format!("Could not start scheduler: {}", err).as_str()));
            }
        }        
        loop {        
            for (uuid, monitor) in self.status.lock().unwrap().iter() {
                println!("Monitor {}-{} status: {:?}", uuid, monitor.get_name(), monitor.status);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    /**
     * Add a job to the scheduler.
     * 
     * @param scheduler: The job scheduler.
     * @param job: The job to be added to the scheduler.
     * 
     * @return Result<(), ApplicationError>: The result of adding the job to the scheduler.
     * If the job is added successfully, the result is Ok(()).
     * If the job fails to be added, the result is an ApplicationError.
     * 
     * @throws ApplicationError: If the job fails to be added.
     */
    async fn add_job(&self, scheduler: &JobScheduler, job: Job) -> Result<(), ApplicationError> {
        match scheduler.add(job).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ApplicationError::new(format!("Could not add job: {}", err).as_str())),
        }
        
    }

    /**
     * Get a job from a monitor configuration.
     * 
     * @param monitor: The monitor from which to get the job.
     * 
     * @return Result<Job, ApplicationError>: The result of getting the job.
     * If the job is created successfully, the result is Ok(Job).
     * If the job fails to be created, the result is an ApplicationError.
     * 
     * @throws ApplicationError: If the job fails to be created.
     */
    fn get_tcp_monitor_job(&mut self, schedule: &str, name: &str, host: &str, port: &u16) -> Result<Job, ApplicationError> {  
        let name = name.to_string(); 
        let host = host.to_string();
        let port = port.clone();      
        let status = self.status.clone();
        match Job::new(schedule, move |uuid,_locked| {
            let mut status = match status.lock() {
                Ok(status) => status,
                Err(err) => {
                    eprintln!("Could not get status lock: {:?}. Job stopped.", err);
                    return;
                }
            };
            if !status.contains_key(name.as_str()) {
                status.insert(uuid.to_string(), TcpMonitor::new(host.as_str(), &port, name.as_str()));
            }
            let monitor =  match status.get_mut(&uuid.to_string()) {
                Some(monitor) => monitor,
                None => {
                    eprintln!("Could not get monitor. Job stopped.");
                    return;
                }
            };
            monitor.status = monitor.check();
        }) {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(format!("Could not create job: {}", err).as_str())),
        }
    }
}

pub trait MonitorTrait {
    fn check(&mut self) -> MonitorStatus;
    fn get_name(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum MonitorStatus {
    Ok,
    Unknown,
    Error { message: String }
}

