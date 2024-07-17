use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::common::ApplicationError;
use crate::config::HttpMethod;
use crate::monitoring::monitoring::MonitorStatus;

/**
 * HTTP Monitor.
 * 
 * This struct represents an HTTP monitor.
 * 
 * name: The name of the monitor.
 * url: The URL to monitor.
 * method: The HTTP method to use.
 * body: The body of the request.
 * headers: The headers of the request.
 * status: The status of the monitor.
 */
#[derive(Debug, Clone)]
pub struct HttpMonitor {
    pub name: String,
    pub url: String,
    pub method: HttpMethod,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub status: Arc<Mutex<MonitorStatus>>,
}

impl HttpMonitor {

    /**
     * Create a new HTTP monitor.
     * 
     * url: The URL to monitor.
     * method: The HTTP method to use.
     * body: The body of the request.
     * headers: The headers of the request.
     * name: The name of the monitor.
     * 
     */
    pub fn new(
        url: &str,
        method: &HttpMethod,
        body: &Option<String>,
        headers: &Option<HashMap<String, String>>,
        name: &str,
    ) -> HttpMonitor {
        HttpMonitor {
            url: url.to_string(),
            name: name.to_string(),
            method: method.clone(),
            body: body.clone(),
            headers: headers.clone(),
            status: Arc::new(Mutex::new(MonitorStatus::Unknown)),
        }
    }

    /**
     * Get headers.
     * 
     * headers: The headers.
     * 
     */
    fn get_headers(headers: &Option<HashMap<String, String>>) -> reqwest::header::HeaderMap {
        let mut header_map = reqwest::header::HeaderMap::new();
        match headers {
            Some(headers) => {
                for (key, value) in headers.iter() {
                    header_map.insert(
                        reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                        reqwest::header::HeaderValue::from_str(value).unwrap(),
                    );
                }
            }
            None => {}
        }
        return header_map;
    }

    /**
     * Set the status of the monitor.
     * 
     * status: The new status.
     * 
     */
    fn set_status(&mut self, status: MonitorStatus) {
        match self.status.lock() {
            Ok(mut monitor_status) => {
                *monitor_status = status;
            }
            Err(err) => {
                eprintln!("Error updating monitor status: {:?}", err);
            }
        }
    }

    /**
     * Check the monitor.
     * 
     */
    pub async fn check(
        &mut self
    ) -> Result<(), ApplicationError> {
        let client = reqwest::Client::default();
        let headers = HttpMonitor::get_headers(&self.headers);
        let request_builder = match &self.method {
            HttpMethod::Get => client.get(&self.url).headers(headers),
            HttpMethod::Post => client.post(&self.url).headers(headers),
            HttpMethod::Put => client.put(&self.url).headers(headers),
            HttpMethod::Delete => client.delete(&self.url).headers(headers),
            HttpMethod::Option => client
                .request(reqwest::Method::OPTIONS, &self.url)
                .headers(headers),
            HttpMethod::Head => client.head(&self.url).headers(headers),
        };
        let request_builder = match &self.body {
            Some(body) => request_builder.body(body.clone()),
            None => request_builder,
        };
        let request_builder = request_builder.timeout(Duration::from_secs(5));
        let req_response = request_builder.send().await;
        match req_response {
            Ok(response) => {
                if response.status().is_success() {
                    self.set_status(MonitorStatus::Ok);
                } else {
                    self.set_status(MonitorStatus::Error {
                        message: format!(
                            "Error connecting to {} with status code: {}",
                            &self.url,
                            response.status()
                        ),
                    });
                }
            }
            Err(err) => {
                self.set_status(MonitorStatus::Error {
                    message: format!("Error connecting to {} with error: {}", &self.url, err),
                });
            }
        }
        Ok(())
    }
}
