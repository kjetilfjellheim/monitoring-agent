mod common;
mod services;
mod api;

use clap::Parser;
use common::configuration::MonitoringConfig;
use log::{error, info};
use log4rs::config::Deserializers;
use actix_web::{web, App, HttpServer};

use crate::common::ApplicationArguments;
use crate::api::StateApi;
use crate::services::MonitoringService;

#[actix_web::main]
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
     * Initialize monitoring service.
     */
    let monitoring_service = MonitoringService::new(&monitoring_config);

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
     * Start the HTTP server.
     */
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(StateApi::new(monitoring_service.clone())))
            .service(api::get_current_meminfo)   
            .service(api::get_current_cpuinfo)   
            .service(api::get_current_loadavg)   
    })
    .bind((monitoring_config.server.ip, monitoring_config.server.port))?
    .run()
    .await

} 


