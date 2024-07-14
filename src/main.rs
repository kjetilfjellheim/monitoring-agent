mod common;
mod config;

use std::fs::OpenOptions;

use clap::Parser;
use common::ApplicationError;
use daemonize::Daemonize;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::ApplicationArguments;
use crate::config::{MonitoringConfig, Monitor};

fn main() {
    /*
     * Parse command line arguments.
     */
    let args = ApplicationArguments::parse();

    /*
     * stdout out and stderr file descriptors. 
     */
    let stdout = OpenOptions::new().read(true).write(true).append(true).create(true).open("/var/log/monitoring_agent.out").unwrap();
    let stderr = OpenOptions::new().read(true).write(true).append(true).create(true).open("/var/log/monitoring_agent.err").unwrap();

    /*
     * Daemonize the application.
     */
    let daemonize = Daemonize::new()
        .pid_file("/tmp/monitoring_agent.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .umask(0o770)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(move || { action(args);});

    /* 
     * Start the daemon.
     */ 
    match daemonize.start() {
        Ok(_) => {
            println!("Daemon started");
        },
        Err(e) => eprintln!("Error, {}", e),
    }
        
}

fn action(args: ApplicationArguments) {
    /*
    * Create a new monitoring configuration. Retrived from the configuration file.
    */
    println!("Getting monitoring configuration");
    let monitoring_config = MonitoringConfig::new(&args.config);
    /*
    * Match the monitoring configuration.
    */
    let monitoring_config = match monitoring_config {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {:?}", err.message);
            std::process::exit(1);
        }
    };
    /*
     * Start the scheduling of the monitoring jobs.
     */
    println!("Starting scheduling");
    let future_scheduling = schedule(&monitoring_config);
    /*
     * Block the main thread until the scheduling is done.
     */
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future_scheduling) {
        Ok(_) => {
            println!("Scheduling done");
        },
        Err(err) => {
            eprintln!("Error: {:?}", err.message);
            std::process::exit(1);
        
        }
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
async fn schedule(monitoring_config: &MonitoringConfig) -> Result<(), ApplicationError> {
    let scheduler = JobScheduler::new().await.unwrap();
    for monitor in monitoring_config.monitors.iter() {
        let monitor = monitor.clone();
        let job = get_job(monitor)?;
        add_scheduler(&scheduler, job).await?;
    }
    scheduler.start().await.unwrap();
    loop {
        print!("Daemon running");
        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 5)).await;
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
async fn add_scheduler(scheduler: &JobScheduler, job: Job) -> Result<(), ApplicationError> {
    match scheduler.add(job).await {
        Ok(_) => Ok(()),
        Err(err) => Err(ApplicationError::new(1, format!("Could not add job: {}", err).as_str())),
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
fn get_job(monitor: Monitor) -> Result<Job, ApplicationError> {
    match Job::new(monitor.schedule.as_str(), move |_a,b| {
        println!("Running monitor: {:?}", monitor.name);
    }) {
        Ok(job) => Ok(job),
        Err(err) => Err(ApplicationError::new(1, format!("Could not create job: {}", err).as_str())),
    }
}