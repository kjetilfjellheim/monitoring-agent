use crate::{common::configuration::ServerConfig, services::MonitoringService};

/**
 * State object for the API modules.
 */
#[allow(clippy::module_name_repetitions)]
pub struct StateApi {
    /// Monitoring service object.
    pub monitoring_service: MonitoringService,
    /// Server configuration object.
    pub server_config: ServerConfig,
    /// Monitored application names.
    pub monitered_application_names: Vec<String>,
}

impl StateApi {
    /**
     * Constructor for `MeminfoApi`
     * 
     * @param `monitoring_service` `MonitoringService` The monitoring service object.
     * @param `server_config` `ServerConfig` The server configuration object.
     * @param `monitered_application_names` `&Vec<String>` The monitored application names.
     * 
     * @return `StateApi`
     * 
     */
    pub fn new(monitoring_service: MonitoringService, server_config: ServerConfig, monitered_application_names: &[String]) -> StateApi {
        StateApi {
            monitoring_service,
            server_config,
            monitered_application_names: monitered_application_names.iter().map(|f| f.chars().take(15).collect()).collect(),
        }
    }
}
