use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::common::ApplicationError;

/**
 * Monitor type.
 *
 * This enum represents the different types of monitors that can be used.
 *
 * `Tcp`: Monitor a TCP connection.
 * `Http`: Monitor an HTTP connection.
 * `Sql`: Monitor a SQL connection.
 * `Command`: Monitor a command.
 * `LoadAvg`: Monitor the load average of the system. Can only be one.
 *
 */
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MonitorType {
    Tcp {
        host: String,
        port: u16,
    },
    Http {
        url: String,
        method: HttpMethod,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        #[serde(default = "default_as_true", alias = "useBuiltinRootCerts")]
        use_builtin_root_certs: bool,
        #[serde(default = "default_as_false", rename = "acceptInvalidCerts")]
        accept_invalid_certs: bool,
        #[serde(default = "default_as_false", rename = "tlsInfo")]
        tls_info: bool,
        #[serde(skip_serializing_if = "Option::is_none", rename = "rootCertificate")]
        root_certificate: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "identity")]
        identity: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "identityPassword")]
        identity_password: Option<String>,
    },
    Command {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected: Option<String>,
    },
    LoadAvg {
        #[serde(skip_serializing_if = "Option::is_none", rename = "threshold1min")]
        threshold_1min: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "threshold5min")]
        threshold_5min: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "threshold10min")]
        threshold_10min: Option<f32>,
        #[serde(rename = "storeValues", default = "default_as_false")]
        store_values: bool,    
    },  
    Mem {
        #[serde(skip_serializing_if = "Option::is_none", rename = "maxPercentageMemUsed")]
        max_percentage_mem: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "maxPercentageSwapUsed")]
        max_percentage_swap: Option<f64>,        
        #[serde(rename = "storeValues", default = "default_as_false")]
        store_values: bool,    
    },   
    Systemctl {
        #[serde(rename = "active")]
        active: Vec<String>,
    },
    Database {
        /// Database config. If not given then use the global database config.
        #[serde(skip_serializing_if = "Option::is_none", rename = "config")]
        database_config: Option<DatabaseConfig>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "maxQueryTime")]
        max_query_time: Option<u32>,          
    },    
}

/**
 * HTTP methods.
 */
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Option,
    Head,
}

/**
 * Monitor struct.
 *
 * This struct represents a monitor configuration.
 *
 * name: Monitor name.
 * schedule: Monitor cron schedule.
 * monitor: Monitor type.
 *
 */
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Monitor {
    /// The name of the monitor.
    #[serde(rename = "name")]
    pub name: String,
    /// The schedule of the monitor.
    #[serde(rename = "schedule")]
    pub schedule: String,
    /// The details of the monitor.
    #[serde(rename = "details")]
    pub details: MonitorType,
    /// The database store configuration.
    #[serde(rename = "store", default = "default_database_store_level")]
    pub store: DatabaseStoreLevel,
}

/**
 * Database store level.
 */
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DatabaseStoreLevel {
    /// Store nothing.
    None,
    /// Store all.
    All,
    /// Store only errors.
    Errors,
}

/**
 * Monitoring configuration.
 *
 * This struct represents the monitoring configuration.
 *
 * input: Input file.
 * monitors: List of monitors.
 *
 */
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// The server configuration. Example ip and port where web services are made available.
    #[serde(rename = "server", default = "default_server")]
    pub server: ServerConfig,
    /// The database configuration. If non is provided, then no storage is used.
    #[serde(rename = "database")]
    pub database: Option<DatabaseConfig>,
    /// The list of monitors.
    #[serde(rename = "monitors")]
    pub monitors: Vec<Monitor>,


}

impl MonitoringConfig {
    /**
     * Create a new monitoring configuration.
     * 
     * input: The input file.
     * 
     * result: The result of creating the monitoring configuration.
     */
    pub fn new(input: &str) -> Result<MonitoringConfig, ApplicationError> {
        let monitor_data: String = MonitoringConfig::get_monitor_data(input)?;
        MonitoringConfig::get_monitor_config(monitor_data.as_str())
    }

    /**
     * Get monitor data.
     * 
     * path: The path to the monitor data.
     * 
     * result: The result of getting the monitor data.
     */
    fn get_monitor_data(path: &str) -> Result<String, ApplicationError> {
        match fs::read_to_string(path) {
            Ok(data) => Ok(data),
            Err(err) => Err(ApplicationError::new(
                format!("Could not read config file {path}, error: {err}").as_str(),
            )),
        }
    }

    /**
     * Get monitor configuration.
     * 
     * data: The monitor data.
     * 
     * result: The result of getting the monitor configuration.
     */
    fn get_monitor_config(data: &str) -> Result<MonitoringConfig, ApplicationError> {
        match serde_json::from_str(data) {
            Ok(monitor_config) => Ok(monitor_config),
            Err(err) => Err(ApplicationError::new(
                format!("Could not parse config file: Line {}", err.line()).as_str(),
            )),
        }
    }
}

/**
 * Server configuration.
 */
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ServerConfig {
    /// The port of the server.
    #[serde(rename = "port", default = "default_server_port")]
    pub port: u16,
    /// The ip of the server.
    #[serde(rename = "ip", default = "default_server_ip")]
    pub ip: String,
    /// The name of the server.
    #[serde(rename = "name", default="default_server_name")]  
    pub name: String,
}

/**
 * Database type.
 */
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum DatabaseType {
    Postgres,
    Mysql,
    Maria
}

/**
 * Database configuration.
 */
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DatabaseConfig {
    /// The type of database.
    #[serde(rename = "type")]
    pub dbtype: DatabaseType,
    /// The host or ip of the database.
    #[serde(rename = "host", default = "default_server_ip")]
    pub host: String,
    /// The database name
    #[serde(rename = "database")]
    pub db_name: String,
    /// The user.
    #[serde(rename = "user")]
    pub user: String,
    /// The password.
    #[serde(rename = "password")]
    pub password: String,
    /// The port.
    #[serde(rename = "port")]
    pub port: u16,
    /// The minimum connections in pool.
    #[serde(rename = "minConnections")]
    pub min_connections: u32,
    /// The maximum connections in pool.
    #[serde(rename = "maxConnections")]
    pub max_connections: u32,    
}

/**
 * Default server configuration.
 * 
 * result: The default server configuration.
 */
fn default_server() -> ServerConfig {
    ServerConfig {
        port: default_server_port(),
        ip: default_server_ip(),
        name: default_server_name(),
    }
}

fn default_server_name() -> String {
    "Default".to_string()
}

/**
 * Default as false. Fix for issue with serde. Issue <https://github.com/serde-rs/serde/issues/368>
 */
fn default_as_false() -> bool {
    false
}

/**
 * Default as true. Fix for issue with serde. Issue <https://github.com/serde-rs/serde/issues/368>
 */
fn default_as_true() -> bool {
    true
}

/**
 * Default port.
 */
fn default_server_port() -> u16 {
    65000
}
/**
 * Default ip.
 */
fn default_server_ip() -> String {
    "127.0.0.1".to_string()
}
/**
 * Default database store level.
 */
fn default_database_store_level() -> DatabaseStoreLevel {
    DatabaseStoreLevel::Errors
}

#[cfg(test)]
mod tests {

    use super::*;

    /**
     * Test for a simple tcp monitor.
     */
    #[test]
    fn test_simple_tcp_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_simple_tcp.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                host: "192.168.1.1".to_string(),
                port: 8080
            }
        );
        assert_eq!(&8080, &monitoring.server.clone().port);
        assert_eq!(&"127.0.0.1", &monitoring.server.ip);
        Ok(())
    }

    /**
     * Test for a simple http monitor.
     */
    #[test]
    fn test_simple_http_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_simple_http.json")?;
        assert_eq!("1 2 3 4 5 6 7".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://post.com".to_string(),
                body: Some("body".to_string()),
                method: HttpMethod::Post,
                headers: Some(HashMap::new()),
                use_builtin_root_certs: true,
                accept_invalid_certs: false,
                tls_info: false,
                root_certificate: None,
                identity: None,
                identity_password: None
            }
        );
        assert_eq!(&65000, &monitoring.server.clone().port);
        assert_eq!(&"127.0.0.1", &monitoring.server.ip);
        Ok(())
    }

    /**
     * Test for multiple monitors in a single file.
     */
    #[test]
    fn test_multiple_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_multiple.json")?;
        assert_eq!("* * * * * * *".to_string(), monitoring.monitors[0].schedule);
        assert_eq!("* * * * * * *".to_string(), monitoring.monitors[1].schedule);
        assert_eq!(2, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                host: "127.0.0.1".to_string(),
                port: 80,
            }
        );
        let monitor = monitoring.monitors[1].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://post.com".to_string(),
                body: Some("body".to_string()),
                method: HttpMethod::Post,
                headers: Some(HashMap::new()),
                use_builtin_root_certs: true,
                accept_invalid_certs: false,
                tls_info: false,
                root_certificate: None,
                identity: None,
                identity_password: None
            }
        );
        Ok(())
    }

    /**
     * Test for a http monitor with tls fields set.
     */
    #[ignore = "No support for testing yet."]
    #[test]
    fn test_simple_tlsfields() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_simple_tlsfields.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://post.com".to_string(),
                body: Some("body".to_string()),
                method: HttpMethod::Post,
                headers: Some(HashMap::new()),
                use_builtin_root_certs: false,
                accept_invalid_certs: true,
                tls_info: true,
                root_certificate: Some("rootCertificate".to_string()),
                identity: Some("identity".to_string()),
                identity_password: Some("identityPassword".to_string())
            }
        );
        Ok(())
    }
}
