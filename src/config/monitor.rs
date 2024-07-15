use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::common::ApplicationError;

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
    },
    Sql {
        #[serde(skip_serializing_if = "Option::is_none")]
        query: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        database: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    Command {
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Option,
    Head,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub name: String,
    pub schedule: String,
    pub monitor: MonitorType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MonitoringConfig {
    input: String,
    pub monitors: Vec<Monitor>,
}

impl MonitoringConfig {
    pub fn new(input: &str) -> Result<MonitoringConfig, ApplicationError> {
        let monitor_data: String = MonitoringConfig::get_monitor_data(input)?;
        let monitors: Vec<Monitor> = MonitoringConfig::get_monitor_config(monitor_data.as_str())?;
        Ok(MonitoringConfig {
            input: input.to_string(),
            monitors
        })        
    }

    fn get_monitor_data(path: &str) -> Result<String, ApplicationError> {
        match fs::read_to_string(path) {
            Ok(data) => Ok(data),
            Err(err) => Err(ApplicationError::new(format!("Could not read config file {}, error: {}", path, err).as_str())),
        }
    }

    fn get_monitor_config(data: &str) -> Result<Vec<Monitor>, ApplicationError> {
        match serde_json::from_str(data) {
            Ok(monitors) => Ok(monitors),
            Err(err) => Err(ApplicationError::new(format!("Could not parse config file: Line {}", err.line().to_string()).as_str())),        
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_tcp_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_simple_tcp.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                host: "192.168.1.1".to_string(),
                port: 8080
            }
        );
        Ok(())
    }

    #[test]
    fn test_simple_http_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_simple_http.json")?;
        assert_eq!("1 2 3 4 5 6 7".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://post.com".to_string(),
                body: Some("body".to_string()),
                method: HttpMethod::Post,
                headers: Some(HashMap::new())
            }
        );
        Ok(())
    }

    #[test]
    fn test_multiple_file() -> Result<(), ApplicationError> {
        let monitoring: MonitoringConfig =
            MonitoringConfig::new("resources/test/test_multiple.json")?;
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!("0 0 0 0 0 0 1".to_string(), monitoring.monitors[1].schedule);
        assert_eq!(2, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                host: "192.168.1.1".to_string(),
                port: 8080,
            }
        );
        let monitor = monitoring.monitors[1].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://test.com".to_string(),
                body: None,
                method: HttpMethod::Get,
                headers: None
            }
        );
        Ok(())
    }
}
