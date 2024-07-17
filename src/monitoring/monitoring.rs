use std::time::Duration;

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::{ ApplicationArguments, MonitoringConfig };
use crate::common::ApplicationError;
use crate::monitoring::tcpmonitor::TcpMonitor;
use crate::monitoring::httpmonitor::HttpMonitor;
use crate::config::HttpMethod;


/**
 * The main action of the application.
 * 
 * @param args: The application arguments.
 *  
 */
pub struct MonitoringService {
    scheduler: Option<JobScheduler>,
    tcp_monitors: Vec<TcpMonitor>,
    http_monitors: Vec<HttpMonitor>,
}

impl MonitoringService {
    /**
     * Create a new monitoring service.
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
        match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
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
                    self.get_tcp_monitor_job(monitor.schedule.as_str(), monitor.name.as_str(), host.as_str(), &port).await?                                     
                },
                crate::config::MonitorType::Http{url, method, body, headers} => {
                    self.get_http_monitor_job(monitor.schedule.as_str(), monitor.name.as_str(), url.as_str(), &method, &body, &headers).await?
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
            for tcp_monitor in self.tcp_monitors.iter() {
                match tcp_monitor.status.lock() {
                    Ok(lock) => {
                        match &*lock {
                            MonitorStatus::Ok => {
                                println!("Gen {}: Ok", tcp_monitor.name);
                            },
                            MonitorStatus::Unknown => {
                                println!("Gen {}: Unknown", tcp_monitor.name);
                            },
                            MonitorStatus::Error { message } => {
                                println!("Gen {}: Error: {}", tcp_monitor.name, message);
                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("Error getting monitor status: {:?}", err);
                    }
                }
            }
            for http_monitor in self.http_monitors.iter() {
                match http_monitor.status.lock() {
                    Ok(lock) => {
                        match &*lock {
                            MonitorStatus::Ok => {
                                println!("Gen {}: Ok", http_monitor.name);
                            },
                            MonitorStatus::Unknown => {
                                println!("Gen {}: Unknown", http_monitor.name);
                            },
                            MonitorStatus::Error { message } => {
                                println!("Gen {}: Error: {}", http_monitor.name, message);
                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("Error getting monitor status: {:?}", err);
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(20)).await;
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
    async fn get_tcp_monitor_job(&mut self, schedule: &str, name: &str, host: &str, port: &u16) -> Result<Job, ApplicationError> {  
        let tcp_monitor = TcpMonitor::new(host, port, name);
        self.tcp_monitors.push(tcp_monitor.clone());
        let name = name.to_string().clone();
        let host = host.to_string().clone();
        let port = port.clone();
        match Job::new_async(schedule, move |a,b| {
            Box::pin({
            let mut moved_tcp_monitor = tcp_monitor.clone();
            let host = host.to_string().clone();
            let name = name.to_string().clone();
            async move {
                match moved_tcp_monitor.check(&host, &port).await {
                    Ok(_) => {
                        
                    },
                    Err(err) => {
                        eprintln!("{}: Error: {}", &name, err.message);
                    }
                }
            }
            })
        }) {
            Ok(job) => {
                Ok(job)
            },
            Err(err) => {
                Err(ApplicationError::new(format!("Could not create job: {}", err).as_str()))
            }
        }
    }

    async fn get_http_monitor_job(&mut self, schedule: &str, name: &str, url: &str, method: &HttpMethod, body: &Option<String>, headers: &Option<std::collections::HashMap<String, String>>) -> Result<Job, ApplicationError> {  
        let http_monitor = HttpMonitor::new(url, &method, body, headers, &name);
        self.http_monitors.push(http_monitor.clone());
        let name = name.to_string().clone();
        let url = url.to_string().clone();
        let method = method.clone();
        let headers = headers.clone();
        let body = body.clone();
        match Job::new_async(schedule, move |a,b| {
            Box::pin({
            let mut moved_http_monitor = http_monitor.clone();
            let name = name.to_string().clone();
            let url = url.to_string().clone();
            let method = method.clone();
            let headers = headers.clone();
            let body = body.clone();            
            async move {
                match moved_http_monitor.check(&method, &url, &headers, &body).await {
                    Ok(_) => {
                    },
                    Err(err) => {
                        eprintln!("{}: Error: {}", &name, err.message);
                    }
                }
            }
            })
        }) {
            Ok(job) => {
                Ok(job)
            },
            Err(err) => {
                Err(ApplicationError::new(format!("Could not create job: {}", err).as_str()))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MonitorStatus {
    Ok,
    Unknown,
    Error { message: String }
}

