use openssl::{asn1::Asn1Time, x509::X509};
use tokio_cron_scheduler::Job;
use tracing::{debug, error};

use crate::{
    common::{configuration::DatabaseStoreLevel, ApplicationError, DatabaseServiceType, MonitorStatus, MonitorStatusType, Status},
    services::monitors::Monitor,
};

/**
 * The certificate monitor.
 *
 * Fields:
 * `name`: The name.
 * `status`: The status.
 * `certificates`: Paths to the certificates.
 * `threshold_days_warn`: The threshold days for a warning.
 * `threshold_days_error`: The threshold days for an error.
 * `database_service`: The database service.
 * `database_store_level`: The database store level.
 */
#[derive(Debug, Clone)]
pub struct CertificateMonitor {
    /// The name of the monitor.
    name: String,
    /// The status.
    status: MonitorStatusType,
    /// Paths to certificates.
    certificates: Vec<String>,
    /// The threshold days for a warning.
    threshold_days_warn: u32,
    /// The threshold days for an error.
    threshold_days_error: u32,
    /// The database service.
    database_service: DatabaseServiceType,
    /// The database store level.
    database_store_level: DatabaseStoreLevel,
}

impl CertificateMonitor {
    /**
     * Create a new certificate monitor.
     *
     * `name`: The name.
     * `description`: The description.
     * `status`: The status.
     * `certificates`: The certificates.
     * `threshold_days_warn`: The threshold days for a warning.
     * `threshold_days_error`: The threshold days for an error.
     * `database_service`: The database service.
     * `database_store_level`: The database store level.
     *
     * Returns a new `CertificateMonitor`.
     */
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        description: &Option<String>,
        status: &MonitorStatusType,
        certificates: Vec<String>,
        threshold_days_warn: u32,
        threshold_days_error: u32,
        database_service: &DatabaseServiceType,
        database_store_level: &DatabaseStoreLevel,
    ) -> CertificateMonitor {
        let status_lock = status.lock();
        match status_lock {
            Ok(mut lock) => {
                lock.insert(
                    name.to_string(),
                    MonitorStatus::new(name, description, Status::Unknown),
                );
            }
            Err(err) => {
                error!("Error creating command monitor: {:?}", err);
            }
        }
        CertificateMonitor {
            name: name.to_string(),
            status: status.clone(),
            certificates,
            threshold_days_warn,
            threshold_days_error,
            database_service: database_service.clone(),
            database_store_level: database_store_level.clone(),
        }
    }

    pub fn get_certificate_job(certificate_monitor: CertificateMonitor, schedule: &str) -> Result<Job, ApplicationError> {
        let job_result = Job::new_async(schedule, move |_uuid, _locked| {
            Box::pin({
                let mut certificate_monitor = certificate_monitor.clone();
                async move {
                    let _ = certificate_monitor.check().await.map_err(|err| {
                        error!("Error checking monitor: {:?}", err);
                    });
                }
            })
        });
        job_result.map_err(|err| {
            ApplicationError::new(&format!("Error creating certificate job: {err:?}"))
        })
    }

    pub async fn check(&mut self) -> Result<(), ApplicationError> {
        debug!("Checking certificates for monitor: {:?}", self.name);
        /*
         * Store errors and warnings in these vectors.
         */
        let mut vec_errors = Vec::new();
        let mut vec_warns = Vec::new();
        /*
         * Get the error and warn times. If the certificates not after is less than the error time, add to errors or warnings.
         */
        let error_time = Asn1Time::days_from_now(self.threshold_days_error)
            .map_err(|err| ApplicationError::new(&format!("Error getting error time: {err:?}")))?;
        let warn_time = Asn1Time::days_from_now(self.threshold_days_warn)
            .map_err(|err| ApplicationError::new(&format!("Error getting warn time: {err:?}")))?;
        /*
         * Loop through the certificates and check.
         */
        for certificate in &self.certificates {
            /*
             * Read the certificate.
             */
            let cert = Self::read_certificate(certificate)?;
            /*
             * Check if the not after is less than the error time.
             */
            let not_after_error = Self::check_not_after(&cert, &error_time)?;
            /*
             * If the not after is less than the error time, add to errors.
             */
            if not_after_error {
                vec_errors.push(format!(
                    "Certificate: {:?} will expire in less than {:?} days",
                    certificate, self.threshold_days_error
                ));
                continue;
            };
            /*
             * Check if the not after is less than the warn time.
             */
            let not_after_warn = Self::check_not_after(&cert, &warn_time)?;
            /*
             * If the not after is less than the warn time, add to warnings.
             */
            if not_after_warn {
                vec_warns.push(format!(
                    "Certificate: {:?} will expire in less than {:?} days",
                    certificate, self.threshold_days_warn
                ));
                continue;
            };
        }
        /*
         * Set the status.
         */
        self.set_status_info(vec_errors, vec_warns).await;
        Ok(())
    }

    /**
     * Set the status.
     *
     * `vec_errors`: Errors found.
     * `vec_warns`: Warnings found.
     */
    async fn set_status_info(&mut self, vec_errors: Vec<String>, vec_warns: Vec<String>) {
        if !vec_errors.is_empty() {
            self.set_status(&Status::Error {
                message: vec_errors.join("\n") + "\n" + vec_warns.join("\n").as_str(),
            })
            .await;
        } else if !vec_warns.is_empty() {
            self.set_status(&Status::Warn {
                message: vec_warns.join("\n"),
            })
            .await;
        } else {
            self.set_status(&Status::Ok).await;
        }
    }

    /**
     * Read the certificate.
     *
     * `certificate`: The certificate.
     *
     * Returns: The certificate.
     *
     * Errors:
     * - Error reading certificate.
     * - Error parsing certificate.
     */
    fn read_certificate(certificate: &String) -> Result<X509, ApplicationError> {
        let cert_vec = std::fs::read(certificate)
            .map_err(|err| ApplicationError::new(&format!("Error reading certificate: {err:?}")))?;
        let cert = X509::from_pem(&cert_vec)
            .map_err(|err| ApplicationError::new(&format!("Error parsing certificate: {err:?}")))?;
        Ok(cert)
    }

    /**
     * Check if the certificate is older than the error time.
     *
     * `cert`: The certificate.
     * `check_time`: The error time.
     *
     * Returns: True if the certificate not after is less than the error time.
     *
     * Errors:
     * - Error comparing certificate not after to error threshold.
     */
    fn check_not_after(cert: &X509, check_time: &Asn1Time) -> Result<bool, ApplicationError> {
        let not_after_error = cert.not_after().compare(check_time).map_err(|err| {
            ApplicationError::new(&format!(
                "Error comparing certificate not after to error threshold: {err:?}"
            ))
        })?;
        Ok(not_after_error.is_le())
    }
}

/**
 * Implement the `Monitor` trait for `CertificateMonitor`.
 */
impl super::Monitor for CertificateMonitor {
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

    use std::{collections::HashMap, sync::{Arc, Mutex}};

    use super::*;

    #[test]
    fn test_read_certificate() {
        let certificate = "resources/test/client_cert/client.cer".to_string();
        let result = CertificateMonitor::read_certificate(&certificate);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_not_after() {
        let mut cert = X509::builder().unwrap();
        cert.set_not_after(Asn1Time::days_from_now(1).unwrap().as_ref())
            .unwrap();
        let cert = cert.build();
        let check_time_before = Asn1Time::days_from_now(0).unwrap();
        let result_before = CertificateMonitor::check_not_after(&cert, &check_time_before).unwrap();
        assert!(!result_before);
        let check_time_after = Asn1Time::days_from_now(2).unwrap();
        let result_after = CertificateMonitor::check_not_after(&cert, &check_time_after).unwrap();
        assert!(result_after);
    }

    #[tokio::test]
    async fn test_set_status_info_both_err_and_warn() {
        let mut certificate_monitor = CertificateMonitor::new(
            "test",
            &None,
            &Arc::new(Mutex::new(HashMap::new())),
            vec!["resources/test/client_cert/client.cer".to_string()],
            1,
            2,
            &Arc::new(None),
            &DatabaseStoreLevel::None,
        );
        let vec_errors = vec!["Error".to_string()];
        let vec_warns = vec!["Warning".to_string()];
        certificate_monitor
            .set_status_info(vec_errors, vec_warns)
            .await;
        let status = certificate_monitor.get_status();
        let status_lock = status.lock().unwrap();
        let monitor_status = status_lock.get("test").unwrap();
        assert_eq!(
            monitor_status.status,
            Status::Error {
                message: "Error\nWarning".to_string()
            }
        );
    }

    #[tokio::test]
    async fn test_set_status_info_err() {
        let mut certificate_monitor = CertificateMonitor::new(
            "test",
            &None,
            &Arc::new(Mutex::new(HashMap::new())),
            vec!["resources/test/client_cert/client.cer".to_string()],
            1,
            2,
            &Arc::new(None),
            &DatabaseStoreLevel::None,
        );
        let vec_errors = vec!["Error".to_string()];
        let vec_warns = vec![];
        certificate_monitor
            .set_status_info(vec_errors, vec_warns)
            .await;
        let status = certificate_monitor.get_status();
        let status_lock = status.lock().unwrap();
        let monitor_status = status_lock.get("test").unwrap();
        assert_eq!(
            monitor_status.status,
            Status::Error {
                message: "Error\n".to_string()
            }
        );
    }

    #[tokio::test]
    async fn test_set_status_info_warn() {
        let mut certificate_monitor = CertificateMonitor::new(
            "test",
            &None,
            &Arc::new(Mutex::new(HashMap::new())),
            vec!["resources/test/client_cert/client.cer".to_string()],
            1,
            2,
            &Arc::new(None),
            &DatabaseStoreLevel::None,
        );
        let vec_errors = vec![];
        let vec_warns = vec!["Warn".to_string()];
        certificate_monitor
            .set_status_info(vec_errors, vec_warns)
            .await;
        let status = certificate_monitor.get_status();
        let status_lock = status.lock().unwrap();
        let monitor_status = status_lock.get("test").unwrap();
        assert_eq!(
            monitor_status.status,
            Status::Warn {
                message: "Warn".to_string()
            }
        );
    }

    #[tokio::test]
    async fn test_set_status_info_none() {
        let mut certificate_monitor = CertificateMonitor::new(
            "test",
            &None,
            &Arc::new(Mutex::new(HashMap::new())),
            vec!["resources/test/client_cert/client.cer".to_string()],
            1,
            2,
            &Arc::new(None),
            &DatabaseStoreLevel::None,
        );
        let vec_errors = vec![];
        let vec_warns = vec![];
        certificate_monitor
            .set_status_info(vec_errors, vec_warns)
            .await;
        let status = certificate_monitor.get_status();
        let status_lock = status.lock().unwrap();
        let monitor_status = status_lock.get("test").unwrap();
        assert_eq!(monitor_status.status, Status::Ok);
    }
}
