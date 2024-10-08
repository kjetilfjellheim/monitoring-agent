use log::debug;
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
        #[serde(skip_serializing_if = "Option::is_none", rename = "retry", default = "default_none")]
        retry: Option<u16>
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
        #[serde(skip_serializing, rename = "identityPassword")]
        identity_password: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "retry", default = "default_none")]
        retry: Option<u16>
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
        #[serde(skip_serializing_if = "Option::is_none", rename = "threshold15min")]
        threshold_15min: Option<f32>,
        #[serde(rename = "threshold1minLevel", default = "default_threshold_level")]
        threshold_1min_level: ThresholdLevel,
        #[serde(rename = "threshold5minLevel", default = "default_threshold_level")]
        threshold_5min_level: ThresholdLevel,
        #[serde(rename = "threshold15minLevel", default = "default_threshold_level")]
        threshold_15min_level: ThresholdLevel,        
        #[serde(rename = "storeValues", default = "default_as_false")]
        store_values: bool,    
    },  
    Mem {
        #[serde(skip_serializing_if = "Option::is_none", rename = "errorPercentageMemUsed")]
        error_percentage_used_mem: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "errorPercentageSwapUsed")]
        error_percentage_used_swap: Option<f64>,   
        #[serde(skip_serializing_if = "Option::is_none", rename = "warnPercentageMemUsed")]
        warn_percentage_used_mem: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "warnPercentageSwapUsed")]
        warn_percentage_used_swap: Option<f64>,                       
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
    Process {
        /// Aplication names to monitor.
        #[serde(rename = "applicationNames")]
        application_names: Option<Vec<String>>,
        /// Pids to monitor.
        #[serde(rename = "pids")] 
        pids: Option<Vec<u32>>,
        /// Regexp on name.
        #[serde(rename = "regexp")] 
        regexp: Option<String>,     
        /// The maximum memory before warn.
        #[serde(skip_serializing_if = "Option::is_none", rename = "thresholdMemWarn")]
        threshold_mem_warn: Option<u64>,                 
        /// The maximum memory before error.
        #[serde(skip_serializing_if = "Option::is_none", rename = "thresholdMemError")]
        threshold_mem_error: Option<u64>,        
        /// Store vales in database        
        #[serde(rename = "storeValues", default = "default_as_false")]
        store_values: bool,         
    },
    Certificate {
        /// The certificate to monitor.
        #[serde(rename = "certificates")]
        certificates: Vec<String>,
        #[serde(rename = "thresholdDaysWarn", default = "default_threshold_days_warn")]
        threshold_days_warn: u32,
        #[serde(rename = "thresholdDaysError", default = "default_threshold_days_error")]
        threshold_days_error: u32,
    }
}

/**
 * Threshold level.
 */
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Copy)]
pub enum ThresholdLevel {
    Warn,
    Error,
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
    /// The description of the monitor.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
    /// Number of tokio threads.
    #[serde(rename = "tokioThreads", default = "default_tokio_threads")]
    pub tokio_threads: usize,
    /// The tokio stack size in kb.
    #[serde(rename = "tokioStackSize", default = "default_tokio_stack_size")]
    pub tokio_stack_size: usize,
    /// The server configuration. Example ip and port where web services are made available.
    #[serde(rename = "server", default = "default_server")]
    pub server: ServerConfig,
    /// The database configuration. If non is provided, then no storage is used.
    #[serde(rename = "database")]
    pub database: Option<DatabaseConfig>,
    /// The list of monitors.
    #[serde(rename = "monitors")]
    pub monitors: Vec<Monitor>,
    /// Cleanup configuration.
    #[serde(rename = "cleanupConfig")]
    pub cleanup_config: Option<CleanupConfig>,
    /// Notification configuration.
    #[serde(rename = "notificationConfig")]    
    pub notification_config: Option<NotificationConfig>,

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
 * Cleanup configuration.
 */
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct CleanupConfig {
    /// Hours to keep information in the database.
    #[serde(rename = "removeFromDbAfter", default = "default_none")]
    pub max_time_stored_db: Option<u32>,
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
    // The cors headers.
    #[serde(skip_serializing_if = "Option::is_none", rename = "accessControlAllowOrigin")]
    pub access_control_allow_origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "accessControlAllowHeaders")]
    pub access_control_allow_headers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "accessControlAllowMethods")]
    pub access_control_allow_methods: Option<String>,
    #[serde(rename = "accessControlAllowCredentials",skip_serializing_if = "Option::is_none", default = "default_none")]
    pub access_control_allow_credentials: Option<bool>,
    #[serde(rename = "accessControlMaxAge", skip_serializing_if = "Option::is_none", default = "default_none")]
    pub access_control_max_age: Option<u32>,
    #[serde(rename = "tlsConfig", skip_serializing_if = "Option::is_none", default = "default_none")]
    pub tls_config: Option<TlsConfig>,
    #[serde(rename = "workers", default = "default_server_workers")]
    pub workers: usize
}

/**
 * Tls configuration.
 */
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TlsConfig {
    #[serde(rename = "certificate")]
    pub certificate: String,
    #[serde(rename = "identity")]
    pub identity: String,
    #[serde(skip_serializing, rename = "password")]
    pub identity_password: Option<String>
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
    #[serde(skip_serializing, rename = "password")]
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
    /// The maximum lifetime in seconds.
    #[serde(rename = "maxLifetime", default = "default_max_lifetime")]
    pub max_lifetime: u32,        
}

/**
 * Notification configuration.
 */
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NotificationConfig {
    /// The url of the smtp server.
    #[serde(rename = "url")]
    pub url: String,
    /// The recipients.
    #[serde(rename = "recipients")]
    pub recipients: Vec<String>,
    /// From.
    #[serde(rename = "from")]
    pub from: String,
    /// Reply to email.
    #[serde(rename = "replyTo")]
    pub reply_to: String,  
    /// Notification check interval.  
    #[serde(rename = "schedule", default = "default_notify_schedule")]
    pub schedule: String,
    /// Resend errors after minutes
    #[serde(rename = "resendAfter", default = "default_resend_after")]
    pub resend_after: i64,

}

/**
 * Default server configuration.
 * 
 * result: The default server configuration.
 */
fn default_server() -> ServerConfig {
    debug!("Using default server configuration");
    ServerConfig {
        port: default_server_port(),
        ip: default_server_ip(),
        name: default_server_name(),
        access_control_allow_origin: None,
        access_control_allow_headers: None,
        access_control_allow_methods: None,
        access_control_allow_credentials: default_none(),
        access_control_max_age: default_none(),
        tls_config: default_none(),
        workers: default_server_workers()
    }
}

fn default_threshold_level() -> ThresholdLevel {
    ThresholdLevel::Error
}

/**
 * Default server workers.
 * 
 * result: The default server workers.
 */
fn default_server_workers() -> usize {
    debug!("Using default server workers");
    4
}

/**
 * Default none.
 */
fn default_none<T>() -> Option<T> {
    debug!("Using default none");
    None
}

/**
 * Default server name.
 */
fn default_server_name() -> String {
    debug!("Using default server name");
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
    debug!("Using default server port");
    65000
}
/**
 * Default ip.
 */
fn default_server_ip() -> String {
    debug!("Using default server ip");
    "127.0.0.1".to_string()
}
/**
 * Default database store level.
 */
fn default_database_store_level() -> DatabaseStoreLevel {
    debug!("Using default database store level");
    DatabaseStoreLevel::Errors
}
/**
 * Default max lifetime for database connections.
 */
fn default_max_lifetime() -> u32 {
    debug!("Using default max lifetime");
    300
}

/** 
 * Default tokio stack size.
 */
fn default_tokio_stack_size() -> usize {
    debug!("Using default tokio stack size");
    2 * 1024
}

/** 
 * Default tokio threads.
 */
fn default_tokio_threads() -> usize {
    debug!("Using default tokio threads");
    4
}
/**
 * Send notifications evry.
 */
fn default_notify_schedule() -> String {
    debug!("Using default notify schedule");
    "0 */5 * * * *".to_string()
}

/**
 * Default resend error after set hours.
 */
fn default_resend_after() -> i64 {
    debug!("Using default resend after");
    120
}

/**
 * Default threshold days warn.
 */
fn default_threshold_days_warn() -> u32 {
    debug!("Using default threshold days warn");
    30
}

/**
 * Default threshold days error.
 */
fn default_threshold_days_error() -> u32 {
    debug!("Using default threshold days error");
    14
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
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_tcp.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                host: "192.168.1.1".to_string(),
                port: 8080,
                retry: None
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
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_http.json")?;
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
                identity_password: None,
                retry: None
            }
        );
        assert_eq!(&65000, &monitoring.server.clone().port);
        assert_eq!(&"127.0.0.1", &monitoring.server.ip);
        Ok(())
    }

    /**
     * Test for a simple command monitor.
     */
    #[test]
    fn test_simple_command_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_command.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Command {
                command: "ls".to_string(),
                args: Some(vec!["-l".to_string()]),
                expected: Some("expected".to_string())
            }
        );
        Ok(())
    }

    /**
     * Test for a simple mariadb monitor.
     */
    #[test]
    fn test_simple_db_mariadb_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_db_mariadb.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Database { 
                database_config: Some(DatabaseConfig {
                    dbtype: DatabaseType::Maria,
                    host: "localhost".to_string(),
                    db_name: "test".to_string(),
                    user: "root".to_string(),
                    password: "root".to_string(),
                    port: 3306,
                    min_connections: 1,
                    max_connections: 10,
                    max_lifetime: 300,
                }),
                max_query_time: Some(100),
            }
        );
        Ok(())
    }

    /**
     * Test for a simple postgres monitor.
     */
    #[test]
    fn test_simple_db_postgres_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_db_postgres.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Database { 
                database_config: Some(DatabaseConfig {
                    dbtype: DatabaseType::Postgres,
                    host: "localhost".to_string(),
                    db_name: "test".to_string(),
                    user: "root".to_string(),
                    password: "root".to_string(),
                    port: 5432,
                    min_connections: 1,
                    max_connections: 10,
                    max_lifetime: 300,
                }),
                max_query_time: Some(100),
            }
        );
        Ok(())
    }

    /**
     * Test for a simple loadavg monitor.
     */
    #[test]
    fn test_simple_loadavg_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_loadavg.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::LoadAvg {
                threshold_1min: Some(1.0),
                threshold_5min: Some(2.0),
                threshold_15min: Some(3.0),
                threshold_1min_level: ThresholdLevel::Error,
                threshold_5min_level: ThresholdLevel::Error,
                threshold_15min_level: ThresholdLevel::Error,
                store_values: true,               
            }
        );
        Ok(())
    }


    /**
     * Test for a simple memory monitor.
     */
    #[test]
    fn test_simple_meminfo_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_meminfo.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Mem {
                error_percentage_used_mem: Some(80.0),
                error_percentage_used_swap: Some(70.0),
                warn_percentage_used_mem: Some(60.0),
                warn_percentage_used_swap: Some(50.0),
                store_values: true,                            
            }
        );
        Ok(())
    }    


    /**
     * Test for a simple systemctl monitor.
     */
    #[test]
    fn test_simple_systemctl_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_systemctl.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Systemctl { 
                active: vec!["service1".to_string(), "service2".to_string()]                        
            }
        );
        Ok(())
    }   

    /**
     * Test for a simple process monitor.
     */
    #[test]
    fn test_simple_process_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_process.json")?;
        assert_eq!("0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].details.clone();
        assert_eq!(
            monitor,
            MonitorType::Process { 
                application_names: Some(vec!["app1".to_string(), "app2".to_string()]),
                pids: None,
                regexp: None,
                threshold_mem_error: Some(100),
                threshold_mem_warn: Some(100),                
                store_values: true,
                
            }
        );
        Ok(())
    }    

    /**
     * Test for a simple server.
     */
    #[test]
    fn test_simple_server_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/configuration_import_test/test_simple_server.json")?;
        assert_eq!("64999".to_string(), monitoring.server.clone().port.to_string());
        assert_eq!("dev".to_string(), monitoring.server.clone().name.to_string());
        assert_eq!("127.0.0.1".to_string(), monitoring.server.clone().ip.to_string());
        assert_eq!("10".to_string(), monitoring.server.clone().access_control_max_age.unwrap().to_string());
        assert_eq!("*".to_string(), monitoring.server.clone().access_control_allow_origin.unwrap());
        assert_eq!("GET, POST, PUT, DELETE, OPTIONS".to_string(), monitoring.server.clone().access_control_allow_methods.unwrap());
        assert_eq!("Content-Type, Authorization, Content-Length, X-Requested-With".to_string(), monitoring.server.clone().access_control_allow_headers.unwrap());
        Ok(())
    }                   

}
