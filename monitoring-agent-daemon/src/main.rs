mod common;
mod services;
mod api;

use std::fs::OpenOptions;

use clap::Parser;
use common::configuration::MonitoringConfig;
use daemonize::Daemonize;
use log::{error, info};
use log4rs::config::Deserializers;
use actix_web::{web, App, HttpServer};
use services::SchedulingService;

use crate::common::ApplicationArguments;
use crate::api::StateApi;
use crate::services::MonitoringService;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    /*
     * Parse command line arguments.
     */
    let args: ApplicationArguments = ApplicationArguments::parse();

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
     * Start the application.
     */
    if args.daemon {
        start_daemon_application( &monitoring_config, &args).await?;
        Ok(())
    } else {
        start_normal_application(&monitoring_config, &args).await?;
        Ok(())
    }
} 

async fn start_normal_application(monitoring_config: &MonitoringConfig, args: &ApplicationArguments) -> Result<(), std::io::Error> {
    /*
     * Initialize monitoring service.
     */
    let monitoring_service = MonitoringService::new(&monitoring_config);    
    /*
     * Start the scheduling service.
     */
    let cloned_monitoring_config = monitoring_config.clone();
    let cloned_args = args.clone();
    tokio::spawn(async move {
        let mut scheduling_service = SchedulingService::new(&cloned_monitoring_config);
        match scheduling_service.start(cloned_args.test).await {
            Ok(()) => {
                info!("Scheduling service started!");
            }
            Err(err) => {
                error!("Error starting scheduling service: {:?}", err);
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
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(StateApi::new(monitoring_service.clone())))
            .service(api::get_current_meminfo)   
            .service(api::get_current_cpuinfo)   
            .service(api::get_current_loadavg)   
            .service(api::get_processes)
            .service(api::get_process)
            .service(api::get_threads)
    })
    .bind((ip, port))?
    .run()
    .await
}

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
            error!("Error opening stdout file: {:?}", err);
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
            error!("Error opening stderr file: {:?}", err);
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
                start_privileged_action(&cloned_monitoring_config.clone(), &cloned_args.clone()).await;
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

async fn start_privileged_action(monitoring_config: &MonitoringConfig, args: &ApplicationArguments) {           
    async move {            
        /*
        * Initialize monitoring service.
        */
        let monitoring_service: MonitoringService = MonitoringService::new(&monitoring_config);    
        /*
        * Start the scheduling service.
        */
        let cloned_monitoring_config = monitoring_config.clone();
        let cloned_args = args.clone();           
        tokio::spawn(async move {
            let mut scheduling_service = SchedulingService::new(&cloned_monitoring_config);
            match scheduling_service.start(cloned_args.test).await {
                Ok(()) => {
                    info!("Scheduling service started!");
                }
                Err(err) => {
                    error!("Error starting scheduling service: {:?}", err);
                }
            };
        });
        /*
        * Start the HTTP server.
        */
        let cloned_monitoring_config = monitoring_config.clone();
        let ip = cloned_monitoring_config.clone().server.ip.clone();
        let port = cloned_monitoring_config.clone().server.port;
        let http_result = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(StateApi::new(monitoring_service.clone())))
                .service(api::get_current_meminfo)   
                .service(api::get_current_cpuinfo)   
                .service(api::get_current_loadavg)   
                .service(api::get_processes)
                .service(api::get_process)
                .service(api::get_threads)
        })
        .bind((ip, port));
        /*
         * If this is a test, return.
         */
        if args.test {
            return;
        }
        /*
         * Check http server setup.
         */        
        match http_result {
            Ok(http_server) => {
                info!("HTTP server started!");
                let _ = http_server.run().await;
            }
            Err(err) => {
                error!("Error starting HTTP server: {:?}", err);
            }
        }                  
    }.await;
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_normal_application() {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_integration_test.json".to_string(),
            daemon: false,
            test: true,
            stdout: String::new(),
            stderr: String::new(),
            pidfile: String::new(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        super::start_normal_application(&monitoring_config, &args).await;
    }

    #[tokio::test]
    async fn test_daemonize_application() {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_integration_test.json".to_string(),
            daemon: true,
            test: true,
            stdout: "/tmp/monitoring-agent.out".to_string(),
            stderr: "/tmp/monitoring-agent.err".to_string(),
            pidfile: "/tmp/monitoring-agent.pid".to_string(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        super::start_daemon_application(&monitoring_config, &args);
    }
}