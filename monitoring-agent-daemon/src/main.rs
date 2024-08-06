mod common;
mod services;
mod api;

use std::fs::OpenOptions;
use std::sync::Arc;

use clap::Parser;
use common::configuration::{DatabaseConfig, MonitoringConfig, ServerConfig};
use daemonize::Daemonize;
use log::{error, info};
use log4rs::config::Deserializers;
use actix_web::{web, App, HttpServer};
use services::SchedulingService;

use crate::common::ApplicationArguments;
use crate::api::StateApi;
use crate::services::{MonitoringService, DbService};

/**
 * Application entry point.
 * 
 * main: The main function.
 * 
 * Returns the result of the application.
 * 
 */
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    /*
     * Parse command line arguments.
     */
    let args: ApplicationArguments = ApplicationArguments::parse();
    /*
     * Initialize logging.
     */
    match log4rs::init_file(&args.loggingfile, Deserializers::default()) {
        Ok(()) => {
            info!("Logging initialized!");
        }
        Err(err) => {
            error!("Error initializing logging: {:?}", err);
        }
    }
    /*
     * Load configuration.
     */
    let monitoring_config = match MonitoringConfig::new(&args.config) {
        Ok(monitoring_config) => {
            info!("Configuration loaded!");
            Ok(monitoring_config)
        }
        Err(err) => {
            error!("Error loading configuration: {:?}", err);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Error loading configuration"))
        }
    }?;
    /*
     * Start the application.
     */
    if args.daemon {
        start_daemon_application( &monitoring_config, &args).await?;
        Ok(())
    } else {
        start_application(&monitoring_config, &args).await?;
        Ok(())
    }
} 

/**
 * Start the application.
 * 
 * `monitoring_config`: The monitoring configuration.
 * `args`: The application arguments.
 * 
 * Returns the result of starting the application.
 * 
 */
async fn start_application(monitoring_config: &MonitoringConfig, args: &ApplicationArguments) -> Result<(), std::io::Error> {
    /*
     * Initialize database service.
     */
    let database_config = monitoring_config.database.clone();
    let database_service: Arc<Option<DbService>> = if let Some(database_config) = database_config {
        Arc::new(initialize_database(&database_config, &monitoring_config.server).await)
    } else {
        info!("No database configuration found!");
        Arc::new(None)
    };
        
    /*
     * Initialize monitoring service.
     */
    let monitoring_service = MonitoringService::new();    
    /*
     * Start the scheduling service.
     */
    let cloned_monitoring_config = monitoring_config.clone();
    let cloned_args = args.clone();
    let monitor_statuses = monitoring_service.get_status();
    let server_name = monitoring_config.server.name.clone();
    tokio::spawn(async move {
        let mut scheduling_service = SchedulingService::new(&server_name, &cloned_monitoring_config, &monitor_statuses, &database_service.clone());
        match scheduling_service.start(cloned_args.test).await {
            Ok(()) => {
                info!("Scheduling service started!");
            }
            Err(err) => {
                error!("Error starting scheduling service: {err:?}");
            }
        };
    });
    /*
     * Start the HTTP server.
     */
    let ip = monitoring_config.server.ip.clone();
    let port = monitoring_config.server.port;
    /*
        * If this is a test, return.
        */
    if args.test {
        return Ok(());
    }
    info!("Starting HTTP server on {}:{}", ip, port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(StateApi::new(monitoring_service.clone())))
            .service(api::get_current_meminfo)   
            .service(api::get_current_cpuinfo)   
            .service(api::get_current_loadavg)   
            .service(api::get_processes)
            .service(api::get_process)
            .service(api::get_threads)
            .service(api::get_monitor_status)
    })
    .bind((ip, port))?
    .run()
    .await
}

/**
 * Initialize the database service.
 * 
 * `database_config`: The database configuration.
 * 
 * Returns the database service.
 * 
 */
async fn initialize_database(database_config: &DatabaseConfig, server_config: &ServerConfig) -> Option<DbService> {
    match DbService::new(database_config, &server_config.name).await {
        Ok(database_service) => {
            info!("Database service initialized!");
            Some(database_service)
        }
        Err(err) => {
            error!("Error initializing database service: {:?}", err);
            None
        }
    }
}

/**
 * Start the daemon application.
 * 
 * `monitoring_config`: The monitoring configuration.
 * `args`: The application arguments.
 * 
 * Returns the result of starting the daemon application.
 */
async fn start_daemon_application(monitoring_config: &MonitoringConfig, args: &ApplicationArguments) -> Result<(), std::io::Error> {
    /*
     * Open stdout for logging daemon output.
     */
    let stdout = match OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&args.stdout)
    {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening stdout file: {err:?}");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error opening stdout file"));
        }
    };
    /*
     * Open stderr for logging daemon errors.
     */
    let stderr = match OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&args.stderr)
    {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening stderr file: {err:?}");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error opening stderr file"));
        }
    };

    /*
     * Create daemonize object.
     */
    let cloned_monitoring_config = monitoring_config.clone();
    let cloned_args = args.clone();    
    let daemonize = Daemonize::new()
        .pid_file(&args.pidfile)
        .chown_pid_file(true)
        .umask(770)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(move || {
            async move {                               
                let result = start_application(&cloned_monitoring_config.clone(), &cloned_args.clone()).await;
                match result {
                    Ok(()) => {
                        info!("Daemon started!");
                    }
                    Err(err) => {
                        error!("Error starting daemon: {err:?}");
                    }
                }
            }
        });
    /*
     * Start the daemon.
     */
    match daemonize.start() {
        Ok(daemon) => {
            daemon.await;
            info!("Started daemon!");
        }
        Err(err) => {
            error!("Error starting daemon: {:?}", err);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_main_normal() -> Result<(), std::io::Error> {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_configuration.json".to_string(),
            daemon: false,
            test: true,
            stdout: String::new(),
            stderr: String::new(),
            pidfile: String::new(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        start_application(&monitoring_config, &args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_main_daemon() -> Result<(), std::io::Error> {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_configuration.json".to_string(),
            daemon: true,
            test: true,
            stdout: String::new(),
            stderr: String::new(),
            pidfile: String::new(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        start_application(&monitoring_config, &args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_normal_application() {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_configuration.json".to_string(),
            daemon: false,
            test: true,
            stdout: String::new(),
            stderr: String::new(),
            pidfile: String::new(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        let result = super::start_application(&monitoring_config, &args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_daemonize_application() {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_configuration.json".to_string(),
            daemon: true,
            test: true,
            stdout: "/tmp/monitoring-agent.out".to_string(),
            stderr: "/tmp/monitoring-agent.err".to_string(),
            pidfile: "/tmp/monitoring-agent.pid".to_string(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        let result = super::start_daemon_application(&monitoring_config, &args).await;
        assert!(result.is_ok());
    }
}