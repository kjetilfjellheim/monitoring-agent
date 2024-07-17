use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::common::ApplicationError;
use crate::config::HttpMethod;
use crate::monitoring::monitoring::MonitorStatus;

#[derive(Debug, Clone)]
pub struct HttpMonitor {
    pub name: String,
    pub url: String,
    pub method: HttpMethod,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub status: Arc<Mutex<MonitorStatus>>
}

impl HttpMonitor {
    pub fn new(url: &str, method: &HttpMethod, body: &Option<String>, headers: &Option<HashMap<String, String>>, name: &str) -> HttpMonitor {
        HttpMonitor {
            url: url.to_string(),
            name: name.to_string(),
            method: method.clone(),
            body: body.clone(),
            headers: headers.clone(),
            status: Arc::new(Mutex::new(MonitorStatus::Unknown))
        }
    }
    
    fn get_headers(headers: &Option<HashMap<String, String>>) -> reqwest::header::HeaderMap {
        let mut header_map = reqwest::header::HeaderMap::new();
        match headers {
            Some(headers) => {
                for (key, value) in headers.iter() {
                    header_map.insert(reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(), reqwest::header::HeaderValue::from_str(value).unwrap());
                }
            },
            None => { }
        }
        return header_map;
    }

    fn set_status(&mut self, status: MonitorStatus) {
        match self.status.lock() {
            Ok(mut monitor_status) => {
                *monitor_status = status;
            },
            Err(err) => {
                eprintln!("Error updating monitor status: {:?}", err);
            }
        }
    }

    pub async fn check(&mut self, http_method: &HttpMethod, url: &str, headers: &Option<HashMap<String, String>>, body: &Option<String>) -> Result<(), ApplicationError> {
        let client = reqwest::Client::default();
        let headers = HttpMonitor::get_headers(headers);
        let request_builder = match http_method {
            HttpMethod::Get => {
                client.get(url).headers(headers)
            },
            HttpMethod::Post => {
                client.post(url).headers(headers)
            },
            HttpMethod::Put => {
                client.put(url).headers(headers)
            },
            HttpMethod::Delete => {
                client.delete(url).headers(headers)
            },
            HttpMethod::Option => {
                client.request(reqwest::Method::OPTIONS, url).headers(headers)
            },
            HttpMethod::Head => {
                client.head(url).headers(headers)
            }
        };
        let request_builder = match body {
            Some(body) => {
                request_builder.body(body.clone())
            },
            None => {
                request_builder
            }
        };
        let request_builder = request_builder.timeout(Duration::from_secs(5));
        let req_response = request_builder.send().await;        
        match req_response {
            Ok(response) => {
                if response.status().is_success() {
                    self.set_status(MonitorStatus::Ok);
                } else {
                    self.set_status(MonitorStatus::Error { message: format!("Error connecting to {} with status code: {}", url, response.status()) });
                }                
            },
            Err(err) => {
                self.set_status(MonitorStatus::Error { message: format!("Error connecting to {} with error: {}", url, err) });
            }
        }
        Ok(())
    }

}

