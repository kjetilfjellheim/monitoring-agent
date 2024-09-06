use std::sync::Arc;

use crate::{common::configuration::ServerConfig, services::{DbService, MonitoringService}};

/**
 * State object for the API modules.
 */
#[allow(clippy::module_name_repetitions)]
pub struct StateApi {
    /// Monitoring service object.
    pub monitoring_service: MonitoringService,
    /// Database service object.
    pub database_service: Arc<Option<DbService>>,    
    /// Server configuration object.
    pub server_config: ServerConfig,
}

impl StateApi {
    /**
     * Constructor for `StateApi`
     * 
     * @param `monitoring_service` `MonitoringService` The monitoring service object.
     * @param `database_service` `Arc<DbService>` The database service object.
     * @param `server_config` `ServerConfig` The server configuration object.
     * 
     * @return `StateApi`
     * 
     */
    pub fn new(monitoring_service: MonitoringService, database_service: Arc<Option<DbService>>, server_config: ServerConfig) -> StateApi {
        StateApi {
            monitoring_service,
            database_service,
            server_config,
        }
    }
}
