use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use log::info;
use log::{debug, error};
use reqwest::header::HeaderMap;
use reqwest::Certificate;
use reqwest::Identity;

use crate::common::configuration::DatabaseStoreLevel;
use crate::common::ApplicationError;
use crate::common::{MonitorStatus, Status};
use crate::common::HttpMethod;
use crate::services::monitors::Monitor;
use crate::services::MariaDbService;

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
    /// The HTTP client.
    client: reqwest::Client,
    /// The status of the monitor.
    pub status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
    /// The database service.
    database_service: Arc<Option<MariaDbService>>,
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
        use_builtin_root_certs: bool,
        accept_invalid_certs: bool,
        tls_info: bool,
        root_certificate: Option<String>,
        identity: Option<String>,
        identity_password: Option<String>,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        database_service: &Arc<Option<MariaDbService>>,
        database_store_level: DatabaseStoreLevel,
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
                lock.insert(name.to_string(), MonitorStatus::new(name.to_string(), Status::Unknown));
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
            status: status.clone(),
            client,
            database_service: database_service.clone(),
            database_store_level,
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
     * Check the monitor.
     *
     */
    pub async fn check(&mut self) -> Result<(), ApplicationError> {
        debug!("Checking monitor: {}", &self.name);
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
        let request_builder = request_builder.timeout(Duration::from_secs(5));
        /*
         * Send request.
         */
        let req_response = request_builder.send().await;
        /*
         * Check response and set status in the monitor.
         */
        self.check_response_and_set_status(req_response);
        debug!("Monitor checked: {}", &self.name);
        Ok(())
    }

    /**
     * Check the response and set the status of the monitor.
     *
     * `response`: The response from the request.
     *
     */
    fn check_response_and_set_status(
        &mut self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) {
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    self.set_status(&Status::Ok);
                } else {
                    info!("Monitor status error: {} - {:?}", &self.name, response);
                    self.set_status(&Status::Error {
                        message: format!(
                            "Error connecting to {} with status code: {}",
                            &self.url,
                            response.status()
                        ),
                    });
                }
            }
            Err(err) => {
                self.set_status(&Status::Error {
                    message: format!("Error connecting to {} with error: {err}", &self.url),
                });
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
    fn get_status(&self) -> Arc<Mutex<HashMap<String, MonitorStatus>>> {
        self.status.clone()
    }

    /**
     * Get the database service.
     *
     * Returns: The database service.
     */
    fn get_database_service(&self) -> Arc<Option<MariaDbService>> {
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

    use reqwest::header::HeaderValue;

    use crate::common::HttpMethod;
    use std::collections::HashMap;

    /**
     * Test the `check` method. Testing failure towards a non-existing URL.
     */
    #[tokio::test]
    async fn test_check() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = HttpMonitor::new(
            "http://localhost:65000",
            HttpMethod::Get,
            &None,
            &None,
            "localhost",
            true,
            true,
            false,
            None,
            None,
            None,
            &status,
            &Arc::new(None),
            DatabaseStoreLevel::None
        )
        .unwrap();
        monitor.check().await.unwrap();
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error connecting to http://localhost:65000 with error: error sending request for url (http://localhost:65000/)".to_string() });
    }

    /**
     * Test the `check` method with tls config. Testing failure towards a non-existing URL.
     */
    #[tokio::test]
    async fn test_check_with_tls() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = HttpMonitor::new(
            "http://localhost:65000",
            HttpMethod::Get,
            &None,
            &None,
            "localhost",
            true,
            true,
            false,
            Some("./resources/test/server_cert/server.cer".to_string()),
            Some("./resources/test/client_cert/client.p12".to_string()),
            Some("test".to_string()),
            &status,
            &Arc::new(None),
            DatabaseStoreLevel::None
        )
        .unwrap();
        monitor.check().await.unwrap();
        assert_eq!(status.lock().unwrap().get("localhost").unwrap().status, Status::Error { message: "Error connecting to http://localhost:65000 with error: error sending request for url (http://localhost:65000/)".to_string() });
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
    #[test]
    fn test_set_status() {
        let status: Arc<Mutex<HashMap<String, MonitorStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut monitor = HttpMonitor::new(
            "https://www.google.com",
            HttpMethod::Get,
            &None,
            &None,
            "Google",
            true,
            true,
            false,
            None,
            None,
            None,
            &status,
            &Arc::new(None),
            DatabaseStoreLevel::None
        )
        .unwrap();
        monitor.set_status(&Status::Ok);
        assert_eq!(
            status.lock().unwrap().get("Google").unwrap().status,
            Status::Ok
        );
    }
}
