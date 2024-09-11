use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::services::DbService;

use super::MonitorStatus;

/**
 * The status of the monitors.
 */
pub type MonitorStatusType = Arc<Mutex<HashMap<String, MonitorStatus>>>;
/**
 * The database service.
 */
pub type DatabaseServiceType = Arc<Option<DbService>>;
