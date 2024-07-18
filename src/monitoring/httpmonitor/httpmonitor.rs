use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use reqwest::header::HeaderMap;

use crate::common::ApplicationError;
use crate::config::HttpMethod;
use crate::common::MonitorStatus;

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
     * This method converts a HashMap to a HeaderMap.
     * 
     * headers: The headers.
     * 
     * Returns a HeaderMap.
     * 
     */
    fn get_header_map(headers: &HashMap<String, String>) -> Result<reqwest::header::HeaderMap, ApplicationError> {
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in headers.iter() {
            header_map.insert(
                HttpMonitor::get_header_name(key)?,
                HttpMonitor::get_header_value(value)?,
            );
        }
        Ok(header_map)
    }

    /**
     * Get header name.
     * 
     * key: The key.
     * 
     * Returns a HeaderName.
     * 
     */
    fn get_header_name(key: &String) -> Result<reqwest::header::HeaderName, ApplicationError> {
        match reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
            Ok(header_name) => Ok(header_name),
            Err(err) => Err(ApplicationError::new(&format!("Error creating header name: {}", err)))
        }
    }

    /**
     * Get header value.
     * 
     * value: The value.
     * 
     * Returns a HeaderValue.
     * 
     */
    fn get_header_value(value: &String) -> Result<reqwest::header::HeaderValue, ApplicationError> {
        match reqwest::header::HeaderValue::from_str(value) {
            Ok(header_value) => Ok(header_value),
            Err(err) => Err(ApplicationError::new(&format!("Error creating header value: {}", err)))
        }
    }

    /**
     * Get headers. 
     * If headers is None, an empty HeaderMap is returned.
     * 
     * headers: The headers.
     * 
     * Returns a HeaderMap.
     * 
     */
    fn get_headers(headers: &Option<HashMap<String, String>>) -> Result<reqwest::header::HeaderMap, ApplicationError> {
        match headers {
            Some(headers) => {
                return HttpMonitor::get_header_map(headers);
            }
            None => { 
                return Ok(HeaderMap::new()); 
            }
        }
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
        /*
         *  Create http client.
         */
        let client = reqwest::Client::default();
        /*
         * Set http method.
         */
        let request_builder = match &self.method {
            HttpMethod::Get => client.get(&self.url),
            HttpMethod::Post => client.post(&self.url),
            HttpMethod::Put => client.put(&self.url),
            HttpMethod::Delete => client.delete(&self.url),
            HttpMethod::Option => client
                .request(reqwest::Method::OPTIONS, &self.url),
            HttpMethod::Head => client.head(&self.url),
        };
        /*
         * Set headers.
         */
        let request_builder = request_builder.headers(HttpMonitor::get_headers(&self.headers)?);
        /*
         * Set body.
         */
        let request_builder = match &self.body {
            Some(body) => request_builder.body(body.clone()),
            None => request_builder,
        };
        /*
         * Set timeout.
         */
        let request_builder = request_builder.timeout(Duration::from_secs(5));
        /*
         * Send request. 
         */
        let req_response = request_builder.send().await;
        /*
         * Check response and set status in the monitor.
         */
        self.check_response_and_set_status(req_response);
        Ok(())
    }

    /**
     * Check the response and set the status of the monitor.
     * 
     * response: The response from the request.
     * 
     */
    fn check_response_and_set_status(&mut self, response: Result<reqwest::Response, reqwest::Error>) {
        match response {
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
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use reqwest::header::HeaderValue;

    use crate::config::HttpMethod;
    use std::collections::HashMap;

    /**
     * Test the check method. Testing failure towards a non-existing URL.
     */
    #[tokio::test]
    async fn test_check() {
        let mut monitor = HttpMonitor::new(
            "http://localhost:65000",
            &HttpMethod::Get,
            &None,
            &None,
            "localhost",
        );
        monitor.check().await.unwrap();
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Error { message: "Error connecting to http://localhost:65000 with error: error sending request for url (http://localhost:65000/)".to_string() });
    }

    /**
     * Test the get_headers method.
     */
    #[test]
    fn test_get_headers() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let header_map = HttpMonitor::get_headers(&Some(headers));
        let header_map = header_map.unwrap();
        assert_eq!(header_map.len(), 1);
        assert_eq!(header_map.get("Content-Type"), Some(&HeaderValue::from_str("application/json").unwrap()));
    }

    /**
     * Test the set_status method.
     */
    #[test]
    fn test_set_status() {
        let mut monitor = HttpMonitor::new(
            "https://www.google.com",
            &HttpMethod::Get,
            &None,
            &None,
            "Google",
        );
        monitor.set_status(MonitorStatus::Ok);
        assert_eq!(*monitor.status.lock().unwrap(), MonitorStatus::Ok);
    }
}