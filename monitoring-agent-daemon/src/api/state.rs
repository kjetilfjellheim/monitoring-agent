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
}

impl StateApi {
    /**
     * Constructor for `MeminfoApi`
     * 
     * @param `monitoring_service` `MonitoringService` The monitoring service object.
     * 
     * @return `StateApi`
     * 
     */
    pub fn new(monitoring_service: MonitoringService, server_config: ServerConfig) -> StateApi {
        StateApi {
            monitoring_service,
            server_config,
        }
    }
}
