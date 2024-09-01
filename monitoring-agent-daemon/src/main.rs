mod common;
mod services;
mod api;

use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use clap::Parser;
use common::configuration::{DatabaseConfig, MonitoringConfig, ServerConfig};
use common::{ApplicationError, MonitorType};
use daemonize::Daemonize;
use log::{debug, error, info};
use actix_web::{web, App, HttpServer};
use openssl::pkey::{PKey, Private};
use openssl::ssl::{SslAcceptor, SslMethod};
use services::SchedulingService;
use tokio::runtime::Builder;
use tracing_subscriber::{filter, prelude::*};

use crate::common::ApplicationArguments;
use crate::api::StateApi;
use crate::services::{MonitoringService, DbService};

type StdioFilter = filter::Filtered<tracing_subscriber::fmt::Layer<tracing_subscriber::layer::Layered<filter::Filtered<tracing_subscriber::fmt::Layer<tracing_subscriber::Registry, tracing_subscriber::fmt::format::DefaultFields, tracing_subscriber::fmt::format::Format, Arc<File>>, filter::LevelFilter, tracing_subscriber::Registry>, tracing_subscriber::Registry>, tracing_subscriber::fmt::format::Pretty, tracing_subscriber::fmt::format::Format<tracing_subscriber::fmt::format::Pretty>>, filter::LevelFilter, tracing_subscriber::layer::Layered<filter::Filtered<tracing_subscriber::fmt::Layer<tracing_subscriber::Registry, tracing_subscriber::fmt::format::DefaultFields, tracing_subscriber::fmt::format::Format, Arc<File>>, filter::LevelFilter, tracing_subscriber::Registry>, tracing_subscriber::Registry>>;
type FileFilter = filter::Filtered<tracing_subscriber::fmt::Layer<tracing_subscriber::Registry, tracing_subscriber::fmt::format::DefaultFields, tracing_subscriber::fmt::format::Format, Arc<File>>, filter::LevelFilter, tracing_subscriber::Registry>;

/**
 * Application entry point.
 * 
 * main: The main function.
 * 
 * Returns the result of the application.
 * 
 */
fn main() -> Result<(), std::io::Error> {
    /*
     * Parse command line arguments.
     */
    let args: ApplicationArguments = ApplicationArguments::parse();
    /*
     * Initialize logging.
     */
    setup_logging(args.logfile.as_str(), &args.stdout_errorlevel, &args.file_errorlevel).map_err(|err| {
        error!("Error setting up logging: {:?}", err);
        std::io::Error::new(std::io::ErrorKind::Other, format!("Error setting up logging: {err:?}"))
    })?;
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

    let runtime = Builder::new_multi_thread()
        .worker_threads(monitoring_config.tokio_threads)
        .enable_all()
        .thread_stack_size(1024 * monitoring_config.tokio_stack_size)
        .build();
    match runtime {
        Ok(runtime) => {
            runtime.block_on(initialize(args, monitoring_config))
        }
        Err(err) => {
            error!("Error creating runtime: {:?}", err);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Error creating runtime"))
        }        
    }
 }
/**
 * Initialize the application.
 */
async fn initialize(args: ApplicationArguments, monitoring_config: MonitoringConfig) -> Result<(), std::io::Error> {    
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
    let database_service = init_database(monitoring_config).await;    
    /*
     * Initialize monitoring service.
     */
    let monitoring_service = MonitoringService::new();    
    /*
     * Start the scheduling service.
     */
    init_scheduling(monitoring_config, args, &monitoring_service, &database_service);
    /*
     * If this is a test, return.
     */
    if args.test {
        return Ok(());
    }
    let monitered_application_names = get_applications(monitoring_config);
    /*
     * Initialize the HTTP server.
     */
    init_http_server(monitoring_config, monitoring_service, database_service, monitered_application_names).await
}

/**
 * Initialize the database service.
 * 
 * `monitoring_config`: The monitoring configuration.
 * 
 * Returns the database service.
 * 
 */
async fn init_database(monitoring_config: &MonitoringConfig) -> Arc<Option<DbService>> {
    let database_config = monitoring_config.database.clone();
    let database_service: Arc<Option<DbService>> = if let Some(database_config) = database_config {
        Arc::new(initialize_database(&database_config, &monitoring_config.server).await)
    } else {
        info!("No database configuration found!");
        Arc::new(None)
    };
    database_service
}

/**
 * Initialize the scheduling service.
 * 
 * `monitoring_config`: The monitoring configuration.
 * `args`: The application arguments.
 * `monitoring_service`: The monitoring service.
 * `database_service`: The database service.
 * 
 */
fn init_scheduling(monitoring_config: &MonitoringConfig, args: &ApplicationArguments, monitoring_service: &MonitoringService, database_service: &Arc<Option<DbService>>) {
    let cloned_monitoring_config = monitoring_config.clone();
    let cloned_args = args.clone();
    let monitor_statuses = monitoring_service.get_status();
    let server_name = monitoring_config.server.name.clone();
    let cloned_database_service = database_service.clone();
    tokio::spawn(async move {
        let mut scheduling_service = SchedulingService::new(&server_name, &cloned_monitoring_config, &monitor_statuses, &cloned_database_service);
        match scheduling_service.start(cloned_args.test).await {
            Ok(()) => {
                info!("Scheduling service started!");
            }
            Err(err) => {
                error!("Error starting scheduling service: {err:?}");
            }
        };
    });
}

/**
 * Initialize the HTTP server.
 * 
 * `monitoring_config`: The monitoring configuration.
 * `monitoring_service`: The monitoring service.
 * `database_service`: The database service.
 * `monitered_application_names`: The monitored application names.
 * 
 * Returns the result of initializing the HTTP server.
 */
async fn init_http_server(monitoring_config: &MonitoringConfig, monitoring_service: MonitoringService, database_service: Arc<Option<DbService>>, monitered_application_names: Vec<String>) -> Result<(), std::io::Error> {
    /*
     * Start the HTTP server.
     */
    let ip = monitoring_config.server.ip.clone();
    let port = monitoring_config.server.port;
    let cloned_monitoring_config = monitoring_config.clone();
    info!("Starting HTTP server on {}:{}", ip, port);
    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(StateApi::new(monitoring_service.clone(), database_service.clone(), cloned_monitoring_config.server.clone(), &monitered_application_names)))
            .service(api::get_current_meminfo)   
            .service(api::get_historical_meminfo)
            .service(api::get_current_cpuinfo)   
            .service(api::get_current_loadavg)  
            .service(api::get_historical_loadavg) 
            .service(api::get_processes)
            .service(api::get_historical_statm)
            .service(api::get_process)
            .service(api::get_threads)
            .service(api::get_monitor_status)
            .service(api::get_current_statm)
            .service(api::get_stat)
            .service(api::get_ping)
    });
    let http_server = http_server.workers(monitoring_config.server.workers);
    let http_server = match monitoring_config.server.tls_config.clone() {
        Some(tls_config) => {
            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("Error creating SSL acceptor: {err:?}")))?;
            let private_key = load_private_key(&tls_config.identity, tls_config.identity_password).map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("Error loading private key: {err:?}")))?;
            builder.set_certificate_chain_file(tls_config.certificate).map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("Error setting certificate chain file: {err:?}")))?;        
            builder.set_private_key(&private_key).map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("Error setting private key: {err:?}")))?;
            http_server.bind_openssl((ip, port), builder)?
        }
        None => {
            http_server.bind((ip, port))?
        }
    };
    http_server.run()
    .await
}

/**
 * Get applications that are being monitored.
 * 
 * `monitoring_config`: The monitoring configuration.
 * 
 * Returns the applications.
 * 
 */
fn get_applications(monitoring_config: &MonitoringConfig) -> Vec<String> {
    monitoring_config
        .monitors
        .iter()
        .flat_map(|monitor| match monitor.details.clone() {
            MonitorType::Process { application_names, max_mem_usage: _, store_values } => {
                if store_values {
                    application_names.clone()
                } else {
                    vec![]
                }
            }
            _ => {
                vec![]
            }
        })
        .collect()
}

/** 
 * Load the encrypted private key.
 * 
 * Returns the private key.
 * 
 * # Errors
 * Error loading private key.
 * 
*/
fn load_private_key(private_key_path: &str, private_key_password: Option<String>) -> Result<PKey<Private>, ApplicationError> {
    let mut file = File::open(private_key_path).map_err(|err| ApplicationError::new(format!("Failed to open file: {err:?}").as_str()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|err| ApplicationError::new(format!("Failed to read file: {err:?}").as_str()))?;
    match private_key_password {
        Some(private_key_password) => {
            PKey::private_key_from_pem_passphrase(&buffer, private_key_password.as_bytes()).map_err(|err| ApplicationError::new(format!("Failed to load private key: {err:?}").as_str()))
        }
        None => {
            PKey::private_key_from_pem(&buffer).map_err(|err| ApplicationError::new(format!("Failed to load private key: {err:?}").as_str()))
        }
    }
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
     * Create daemonize object.
     */
    let cloned_monitoring_config = monitoring_config.clone();
    let cloned_args = args.clone();    
    let daemonize = Daemonize::new()
        .pid_file(&args.pidfile)
        .chown_pid_file(true)
        .umask(770)        
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
     * If this is a test, return.
     */
    if args.test {
        debug!("Test mode, returning!");
        return Ok(());
    }
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

/**
 * Setup logging.
 * 
 * `file_path`: The file path for logging.
 * 
 * Returns the result of setting up logging.
 * 
 * # Errors
 * Error creating file appender.
 * Error creating log configuration.
 * Error initializing log configuration.
 * 
 */
fn setup_logging(file_path: &str, stdout_errlevel: &str, file_errlevel: &str) -> Result<(), ApplicationError> {

    // Convert filter from arguments to filter,
    let stdout_level_filter = filter::LevelFilter::from_str(stdout_errlevel).map_err(|err| ApplicationError::new(format!("Invalid level given for stdout arguments: {err:?}").as_str()))?;
    let file_level_filter = filter::LevelFilter::from_str(file_errlevel).map_err(|err| ApplicationError::new(format!("Invalid level given for stdout arguments: {err:?}").as_str()))?;

    // Stdout logger.
    let stdout_log = get_stdout_logger(stdout_level_filter);                

    // A layer that logs events to a file.
    let file = File::create(file_path).map_err(|err| ApplicationError::new(format!("Error creating file appender: {err:?}").as_str()))?;
    let file_log = get_file_logger(file, file_level_filter);  

    tracing_subscriber::registry()
        .with(file_log)
        .with(stdout_log)
        .init();
    Ok(())
}

/**
 * Get stdout logger.
 * 
 * `stdout_level_filter` Stdout level filter
 * 
 * Returns logger
 */
fn get_stdout_logger(stdout_level_filter: filter::LevelFilter) -> StdioFilter {
    tracing_subscriber::fmt::layer()
        .with_thread_ids(false)
        .with_thread_names(true)
        .with_target(false)
        .with_level(true)
        .with_file(false)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_line_number(false)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .pretty()
        .with_filter(stdout_level_filter)
}

/**
 * Get file logger
 * 
 * `file` The file to log to.
 * `file_level_filter` The level filter
 * 
 * Returns  logger
 */
fn get_file_logger(file: File, file_level_filter: filter::LevelFilter) -> FileFilter {
    tracing_subscriber::fmt::layer()
        .with_thread_ids(false)
        .with_thread_names(true)
        .with_target(false)
        .with_level(true)
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_file(false)
        .with_line_number(false)        
        .with_writer(Arc::new(file))
        .with_filter(file_level_filter)
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
            file_errorlevel: "info".to_string(),
            stdout_errorlevel: "info".to_string(),
            pidfile: String::new(),
            logfile: "/tmp/monitoring-agent.log".to_string(),
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
            file_errorlevel: "info".to_string(),
            stdout_errorlevel: "info".to_string(),            
            pidfile: String::new(),
            logfile: "/tmp/monitoring-agent.log".to_string(),
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
            file_errorlevel: "info".to_string(),
            stdout_errorlevel: "info".to_string(),
            pidfile: String::new(),
            logfile: "/tmp/monitoring-agent.log".to_string(),
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
            file_errorlevel: "info".to_string(),
            stdout_errorlevel: "info".to_string(),
            pidfile: "/tmp/monitoring-agent.pid".to_string(),
            logfile: "/tmp/monitoring-agent.log".to_string(),
        };
        let monitoring_config = MonitoringConfig::new(&args.config).unwrap();
        let result = super::start_daemon_application(&monitoring_config, &args).await;
        assert!(result.is_ok());
    }
}