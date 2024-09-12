use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use log::info;
use log::{debug, error};
use reqwest::header::HeaderMap;
use reqwest::{Certificate, RequestBuilder};
use reqwest::Identity;
use tokio_cron_scheduler::Job;

use crate::common::configuration::DatabaseStoreLevel;
use crate::common::ApplicationError;
use crate::common::DatabaseServiceType;
use crate::common::HttpMethod;
use crate::common::MonitorStatusType;
use crate::common::{MonitorStatus, Status};
use crate::services::monitors::Monitor;

/**
 * HTTP Monitor.
 *
 * This struct represents an HTTP monitor.
 *
 * name: The name of the monitor.
 * description: The description of the monitor.
 * url: The URL to monitor.
 * method: The HTTP method to use.
 * body: The body of the request.
 * headers: The headers of the request.
 * status: The status of the monitor.
 */
#[derive(Debug, Clone)]
pub struct HttpMonitor {
    /// The name of the monitor.
    pub name: String,
    /// The URL to monitor.
    pub url: String,
    /// The HTTP method to use.
    pub method: HttpMethod,
    /// The body of the request.
    pub body: Option<String>,
    /// The headers of the request.
    pub headers: Option<HashMap<String, String>>,
    /// Number of retries if error occurs.
    pub retry: Option<u16>,
    /// The HTTP client.
    client: reqwest::Client,
    /// The status of the monitor.
    pub status: MonitorStatusType,
    /// The database service.
    database_service: DatabaseServiceType,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
}

impl HttpMonitor {
    /**
     * Create a new HTTP monitor.
     *
     * `url`: The URL to monitor.
     * `method`: The HTTP method to use.
     * `body`: The body of the request.
     * `headers`: The headers of the request.
     * `name`: The name of the monitor.
     * `use_builtin_root_certs`: Use the built-in root certificates.
     * `accept_invalid_certs`: Accept invalid certificates.
     * `tls_info`: Use TLS info.
     * `root_certificate`: The root certificate.
     * `identity`: The identity.
     * `identity_password`: The password for the identity.
     * `status`: The status of the monitor.
     * `database_service`: The database service.
     *
     * Returns: A new HTTP monitor.
     *
     */
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        url: &str,
        method: HttpMethod,
        body: &Option<String>,
        headers: &Option<HashMap<String, String>>,
        name: &str,
        description: &Option<String>,
        use_builtin_root_certs: bool,
        accept_invalid_certs: bool,
        tls_info: bool,
        root_certificate: Option<String>,
        identity: Option<String>,
        identity_password: Option<String>,
        retry: Option<u16>,
        status: &MonitorStatusType,
        database_service: &DatabaseServiceType,
        database_store_level: &DatabaseStoreLevel,
    ) -> Result<HttpMonitor, ApplicationError> {
        debug!("Creating HTTP monitor: {}", &name);
        /*
         *  Start create http client.
         */
        let client = reqwest::Client::builder()
            .tls_built_in_root_certs(use_builtin_root_certs)
            .danger_accept_invalid_certs(accept_invalid_certs)
            .use_native_tls()
            .tls_info(tls_info);

        /*
         * Add root certificate if included.
         */
        let client = match root_certificate {
            Some(root_certificate) => {
                let root_certificate =
                    HttpMonitor::get_root_certificate(root_certificate.as_str())?;
                client.add_root_certificate(root_certificate)
            }
            None => client,
        };

        /*
         * Set identity if included.
         */
        let client = match identity {
            Some(identity) => {
                let identity = HttpMonitor::get_identity(identity, identity_password)?;
                client.identity(identity)
            }
            None => client,
        };
        /*
         * Get client
         */
        let client = match client.build() {
            Ok(client) => client,
            Err(err) => {
                return Err(ApplicationError::new(&format!(
                    "Error creating HTTP client: {err}"
                )));
            }
        };

        /*
         * Set monitor status.
         */
        let monitor_lock = status.lock();
        match monitor_lock {
            Ok(mut lock) => {
                lock.insert(
                    name.to_string(),
                    MonitorStatus::new(name, description, Status::Unknown),
                );
            }
            Err(err) => {
                error!("Error creating HTTP monitor: {:?}", err);
            }
        };
        /*
         * Return HTTP monitor.
         */
        debug!("HTTP monitor created: {}", &name);
        Ok(HttpMonitor {
            url: url.to_string(),
            name: name.to_string(),
            method,
            body: body.clone(),
            headers: headers.clone(),
            retry,
            status: status.clone(),
            client,
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
        })
    }

    /**
     * This method converts a `HashMap` to a `HeaderMap`.
     *
     * `headers`: The headers.
     *
     * Returns a `HeaderMap`.
     * Returns a `HeaderMap`.
     *
     */
    fn get_header_map(
        headers: &HashMap<String, String>,
    ) -> Result<reqwest::header::HeaderMap, ApplicationError> {
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in headers {
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
     * `key`: The key.
     *
     * Returns a `HeaderName`.
     *
     */
    fn get_header_name(key: &String) -> Result<reqwest::header::HeaderName, ApplicationError> {
        match reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
            Ok(header_name) => Ok(header_name),
            Err(err) => Err(ApplicationError::new(&format!(
                "Error creating header name: {err}"
            ))),
        }
    }

    /**
     * Get header value.
     *
     * `value`: The value.
     *
     * Returns a `HeaderValue`.
     *
     */
    fn get_header_value(value: &str) -> Result<reqwest::header::HeaderValue, ApplicationError> {
        match reqwest::header::HeaderValue::from_str(value) {
            Ok(header_value) => Ok(header_value),
            Err(err) => Err(ApplicationError::new(&format!(
                "Error creating header value: {err}"
            ))),
        }
    }

    /**
     * Get headers.
     * If `headers` is None, an empty `HeaderMap` is returned.
     *
     * `headers`: The headers.
     *
     * Returns a `HeaderMap`.
     *
     */
    fn get_headers(
        headers: &Option<HashMap<String, String>>,
    ) -> Result<reqwest::header::HeaderMap, ApplicationError> {
        match headers {
            Some(headers) => HttpMonitor::get_header_map(headers),
            None => Ok(HeaderMap::new()),
        }
    }

    /**
     * Check the response and set the status of the monitor.
     *
     * `response`: The response from the request.
     * 
     * Returns: The result of checking the response and setting the status.
     * 
     */
    fn check_response_and_set_status(&mut self, response: Result<reqwest::Response, reqwest::Error> ) -> Result<(), ApplicationError> {
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {                    
                    Err(ApplicationError::new(&format!(
                        "Error checking monitor: {} ", response.status()
                    )))                   
                }
            }
            Err(err) => {
                Err(ApplicationError::new(&format!("Error connecting to {} with error: {err}", &self.url)))
            }
        }
    }
    
    /**
     * Get identity.
     *
     * `identity`: The identity file path
     * `identity_password`: The password for the identity file.
     *
     * Returns an Identity.
     *
     */
    fn get_identity(
        identity: String,
        identity_password: Option<String>,
    ) -> Result<Identity, ApplicationError> {
        /*
         * Read identity file.
         */
        let data = match fs::read(identity) {
            Ok(data) => data,
            Err(err) => {
                return Err(ApplicationError::new(&format!(
                    "Error reading identity: {err}"
                )));
            }
        };
        /*
         * Get identity password. If missing then fail.
         */
        let Some(identity_password) = identity_password else {
            return Err(ApplicationError::new("Identity password is required"));
        };
        /*
         * Create identity.
         */
        let identity = match reqwest::Identity::from_pkcs12_der(&data, &identity_password) {
            Ok(identity) => identity,
            Err(err) => {
                return Err(ApplicationError::new(&format!(
                    "Error creating identity: {err}"
                )));
            }
        };
        Ok(identity)
    }

    /**
     * Get root certificate.
     *
     * `root_certificate`: The root certificate file path
     *
     * Returns a certificate.
     *
     */
    fn get_root_certificate(root_certificate: &str) -> Result<Certificate, ApplicationError> {
        /*
         * Read root certificate.
         */
        let data = match fs::read(root_certificate) {
            Ok(data) => data,
            Err(err) => {
                return Err(ApplicationError::new(&format!(
                    "Error reading root_certificate: {err}"
                )));
            }
        };
        /*
         * Create root certificate.
         */
        let identity = match reqwest::Certificate::from_pem(&data) {
            Ok(identity) => identity,
            Err(err) => {
                return Err(ApplicationError::new(&format!(
                    "Error creating identity: {err}"
                )));
            }
        };
        Ok(identity)
    }

    /**
     * Get an HTTP monitor job.
     *
     * `schedule`: The schedule.
     * `name`: The name of the monitor.
     * `url`: The URL to monitor.
     * `method`: The HTTP method.
     * `body`: The body.
     * `headers`: The headers.
     *
     * result: The result of getting the HTTP monitor job.
     *
     * throws: `ApplicationError`: If the job fails to be created.
     */
    pub fn get_http_monitor_job(
        http_monitor: Self,
        schedule: &str,
    ) -> Result<Job, ApplicationError> {
        info!("Creating http monitor: {}", &http_monitor.name);
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {
            Box::pin({
                let mut http_monitor = http_monitor.clone();
                async move {
                    let _ = http_monitor.check().await.map_err(|err| {
                        error!("Error checking monitor: {:?}", err);
                    });
                }
            })
        });
        match job_result {
            Ok(job) => Ok(job),
            Err(err) => Err(ApplicationError::new(
                format!("Could not create job: {err}").as_str(),
            )),
        }
    }

    /**
     * Check the monitor.
     */
    async fn check(&mut self) -> Result<(), ApplicationError> {
        debug!("Checking monitor: {}", &self.name);
        let status = self.connect().await?;
        self.set_status(&status).await;    
        Ok(())   
    }  

    /**
     * Check the monitor.
     */
    async fn connect(&mut self) -> Result<Status, ApplicationError> {
        debug!("Checking monitor: {}", &self.name);
        /*
         * Build the request.
         */
        let request = self.build_request()?;
        /*
         * Send request.
         */
        let req_response = request.send().await;

        let mut current_err = match self.check_response_and_set_status(req_response) {
            Ok(()) => {
                return Ok(Status::Ok);
            },
            Err(err) => {
                err.message 
            },
        };    

        if let Some(retry) = self.retry {
            for index in 1..=retry {
                /*
                * Build the request.
                */
                let request = self.build_request()?;
                let req_response = request.send().await;
                match self.check_response_and_set_status(req_response) {
                    Ok(()) => {
                        return Ok(Status::Warn { message: format!("Success after retries {index}. Previous err: {current_err:?}") });
                    },
                    Err(err) => {
                        current_err = format!("Error after {index} retries. Error: {err:?}");
                    },
                };
            }            
        }                    
        Ok(Status::Error { message: current_err })
    }

    /**
     * Build the request.
     * 
     * Returns: The request builder.
     */
    fn build_request(&self) -> Result<RequestBuilder, ApplicationError> {
        /*
         * Set http method.
         */
        let request_builder = match &self.method {
            HttpMethod::Get => self.client.get(&self.url),
            HttpMethod::Post => self.client.post(&self.url),
            HttpMethod::Put => self.client.put(&self.url),
            HttpMethod::Delete => self.client.delete(&self.url),
            HttpMethod::Option => self.client.request(reqwest::Method::OPTIONS, &self.url),
            HttpMethod::Head => self.client.head(&self.url),
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
        Ok(request_builder.timeout(Duration::from_secs(5)))
    }

}

/**
 * Implement the `Monitor` trait for `HttpMonitor`.
 */
impl super::Monitor for HttpMonitor {
    /**
     * Get the name of the monitor.
     *
     * Returns: The name of the monitor.
     */
    fn get_name(&self) -> &str {
        &self.name
    }

    /**
     * Get the status of the monitor.
     *
     * Returns: The status of the monitor.
     */
    fn get_status(&self) -> MonitorStatusType {
        self.status.clone()
    }

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> DatabaseServiceType {
        self.database_service.clone()
    }

    /**
     * Get the database store level.
     *
     * Returns: The database store level.
     */
    fn get_database_store_level(&self) -> DatabaseStoreLevel {
        self.database_store_level.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::services::monitors::Monitor;

    use reqwest::header::HeaderValue;

    use crate::common::HttpMethod;
    use std::collections::HashMap;

    /**
     * Test the `check` method with tls config. Testing failure towards a non-existing URL.
     */
    #[tokio::test]
    async fn test_check_with_tls() {
        let status: MonitorStatusType =
        std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut monitor = HttpMonitor::new(
            "http://localhost:65000",
            HttpMethod::Get,
            &None,
            &None,
            "localhost",
            &None,
            true,
            true,
            false,
            Some("./resources/test/server_cert/server.cer".to_string()),
            Some("./resources/test/client_cert/client.p12".to_string()),
            Some("test".to_string()),
            Some(1),
            &status,
            &std::sync::Arc::new(None),
            &DatabaseStoreLevel::None,
        )
        .unwrap();
        monitor.check().await.unwrap();
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error after 1 retries. Error: ApplicationError { message: \"Error connecting to http://localhost:65000 with error: error sending request for url (http://localhost:65000/)\" }".to_string() });
    }

    /**
     * Test the `get_headers` method.
     */
    #[test]
    fn test_get_headers() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let header_map = HttpMonitor::get_headers(&Some(headers));
        let header_map = header_map.unwrap();
        assert_eq!(header_map.len(), 1);
        assert_eq!(
            header_map.get("Content-Type"),
            Some(&HeaderValue::from_str("application/json").unwrap())
        );
    }

    /**
     * Test the `set_status` method.
     */
    #[tokio::test]
    async fn test_set_status() {
        let status: MonitorStatusType =
        std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let mut monitor = HttpMonitor::new(
            "https://www.google.com",
            HttpMethod::Get,
            &None,
            &None,
            "Google",
            &None,
            true,
            true,
            false,
            None,
            None,
            None,
            None,
            &status,
            &std::sync::Arc::new(None),
            &DatabaseStoreLevel::None,
        )
        .unwrap();
        monitor.set_status(&Status::Ok).await;
        assert_eq!(
            status.lock().unwrap().get("Google").unwrap().status,
            Status::Ok
        );
    }

    #[test]
    fn test_get_http_monitor_job() {
        let status: MonitorStatusType =
        std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
        let monitor = HttpMonitor::new(
            "https://www.google.com",
            HttpMethod::Get,
            &None,
            &None,
            "Google",
            &None,
            true,
            true,
            false,
            None,
            None,
            None,
            None,
            &status,
            &std::sync::Arc::new(None),
            &DatabaseStoreLevel::None,
        )
        .unwrap();
        let job = HttpMonitor::get_http_monitor_job(monitor, "0 0 * * * *");
        assert!(job.is_ok());
    }
}
