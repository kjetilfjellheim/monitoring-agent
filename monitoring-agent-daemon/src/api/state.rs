use crate::services::MonitoringService;

/**
 * State object for the API modules.
 */
#[allow(clippy::module_name_repetitions)]
pub struct StateApi {
    /// Monitoring service object.
    pub monitoring_service: MonitoringService,
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
    pub fn new(monitoring_service: MonitoringService) -> StateApi {
        StateApi {
            monitoring_service
        }
    }
}
