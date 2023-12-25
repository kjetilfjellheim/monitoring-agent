use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tokio_cron_scheduler::{Job, JobScheduler};
use std::error::Error;


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MonitorType {
    Tcp {
        ip: String,
        port: u32,
        #[serde(skip_serializing_if = "Option::is_none", rename = "serverCertificate")]
        server_certificate: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "privateKey")]
        private_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "privateKeyPassword")]
        private_key_password: Option<String>,
    },
    Http {
        url: String,
        method: HttpMethod,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "httpsCertificate")]
        https_certificate: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "serverCertificate")]
        server_certificate: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "privateKey")]
        private_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "privateKeyPassword")]
        private_key_password: Option<String>,
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    schedule: String,
    monitor: MonitorType,
}

#[derive(Debug, PartialEq)]
pub struct MonitoringConfig {
    input: String,
    monitors: Vec<Monitor>,
}

impl MonitoringConfig {
    pub fn new(input: &str) -> MonitoringConfig {
        let monitor_data: String = MonitoringConfig::get_monitor_data(input);
        let monitors: Vec<Monitor> = MonitoringConfig::get_monitor_config(monitor_data.as_str()).unwrap();
        MonitoringConfig {
            input: input.to_string(),
            monitors,
        }
    }

    fn get_monitor_data(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn get_monitor_config(data: &str) -> Result<Vec<Monitor>, serde_json::Error> {
        serde_json::from_str(data)
    }
}

fn main() {
    MonitoringConfig::new("resources/test/test_simple.json");
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_tcp_file() {
        let monitoring: MonitoringConfig = MonitoringConfig::new("resources/test/test_simple_tcp.json");
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                ip: "192.168.1.1".to_string(),
                port: 8080,
                server_certificate: Some("server_cert1".to_string()),
                private_key: Some("privatekey1".to_string()),
                private_key_password: Some("privatekeypasswd1".to_string())
            }
        );
    }

    #[test]
    fn test_simple_http_file() {
        let monitoring: MonitoringConfig = MonitoringConfig::new("resources/test/test_simple_http.json");
        assert_eq!("1 2 3 4 5 6 7".to_string(), monitoring.monitors[0].schedule);
        assert_eq!(1, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://post.com".to_string(),
                body: Some("body".to_string()),
                method: HttpMethod::Post,
                headers: Some(HashMap::new()),
                https_certificate: Some("httpscert2".to_string()),
                server_certificate: Some("server_cert2".to_string()),
                private_key: Some("privatekey2".to_string()),
                private_key_password: Some("privatekeypasswd2".to_string())
            }
        );
    }

    #[test]
    fn test_multiple_file() {
        let monitoring: MonitoringConfig = MonitoringConfig::new("resources/test/test_multiple.json");
        assert_eq!("0 0 0 0 0 0 0".to_string(), monitoring.monitors[0].schedule);
        assert_eq!("0 0 0 0 0 0 1".to_string(), monitoring.monitors[1].schedule);
        assert_eq!(2, monitoring.monitors.len());
        let monitor = monitoring.monitors[0].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Tcp {
                ip: "192.168.1.1".to_string(),
                port: 8080,
                server_certificate: None,
                private_key: None,
                private_key_password: None
            }
        );
        let monitor = monitoring.monitors[1].monitor.clone();
        assert_eq!(
            monitor,
            MonitorType::Http {
                url: "https://test.com".to_string(),
                body: None,
                method: HttpMethod::Get,
                headers: None,
                https_certificate: None,
                server_certificate: None,
                private_key: None,
                private_key_password: None
            }
        );
    }
}
