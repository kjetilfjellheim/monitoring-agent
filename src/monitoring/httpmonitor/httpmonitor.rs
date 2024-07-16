use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use futures::executor::LocalPool;

use tokio_cron_scheduler::Job;

use crate::common::ApplicationError;
use crate::config::HttpMethod;
use crate::monitoring::monitoring::MonitorTrait;
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
    pub fn new(url: &str, method: HttpMethod, body: Option<String>, headers: Option<HashMap<String, String>>, name: &str) -> HttpMonitor {
        HttpMonitor {
            url: url.to_string(),
            name: name.to_string(),
            method: method,
            body: body,
            headers: headers,
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

    fn check(http_method: &HttpMethod, url: &str, headers: &Option<HashMap<String, String>>, body: &Option<String>) -> MonitorStatus {
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
        println!("Before http async");
        let req_response = tokio::runtime::Builder::.unwrap().block_on(request_builder.send());
        println!("After http async");
        match req_response {
            Ok(response) => {
                if response.status().is_success() {
                    MonitorStatus::Ok
                } else {
                    MonitorStatus::Error { message: format!("Error connecting to {} with status code: {}", url, response.status()) }
                }
            },
            Err(err) => {
                MonitorStatus::Error { message: format!("Error connecting to {} with error: {}", url, err) }
            }
        }
    }

}

impl MonitorTrait for HttpMonitor {

    fn get_job(&mut self, schedule: &str) -> Result<Job, ApplicationError> {
        println!("Creating Http monitor {:?}:{} job...", &self.method, &self.url);
        let status = self.status.clone();
        let http_method = self.method.clone();
        let url = self.url.clone();
        let headers = self.headers.clone();
        let body = self.body.clone();
        let name = self.name.clone();
        match Job::new(schedule, move |_uuid,_locked| {  
            println!("Running http monitor job {}", &name);                
            let new_status = HttpMonitor::check(&http_method, &url, &headers, &body);
            match status.lock() {
                Ok(mut monitor_status) => {
                    *monitor_status = new_status;
                },
                Err(err) => {
                    eprintln!("Error updating monitor status: {:?}", err);
                }
            }
        }) {
            Ok(job) => {
                return Ok(job);
            },
            Err(err) => {
                return Err(ApplicationError::new(format!("Could not create job: {}", err).as_str()));
            }
        };
    }    
    
    fn get_status(&self) -> MonitorStatus {
        match self.status.lock() {
            Ok(status) => {
                return status.clone();
            },
            Err(err) => {
                eprintln!("Error getting monitor status: {:?}", err);
                return MonitorStatus::Unknown;
            }
        }
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }
}
