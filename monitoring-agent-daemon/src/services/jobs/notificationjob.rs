use chrono::{DateTime, Utc};
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};
use std::collections::HashMap;
use tokio_cron_scheduler::Job;
use tracing::error;

use crate::common::{ApplicationError, MonitorStatus, MonitorStatusType, Status};

/**
 * The notification job.
 */
#[derive(Debug, Clone)]
pub struct NotificationJob {
    /// Monitor status
    status: MonitorStatusType,
    /// Map of all the already notified errors.
    already_notified: HashMap<String, DateTime<Utc>>,
    /// The email recipients.
    recipients: Vec<String>,
    /// The email transport.
    transport: SmtpTransport,
    /// The from email.
    from: Mailbox,
    /// The reply to email.
    reply_to: Mailbox,
    /// After how many minutes to resend the notification when an error occurs.
    resend_after: i64,
    /// The notify schedule.
    notify_schedule: String,
}

impl NotificationJob {
    /**
     * Create a new notification job.
     *
     * `status`: The status.
     * `url`: The url.
     * `recipients`: The recipients.
     * `from`: The from.
     * `reply_to`: The reply to.
     * `resend_after`: The resend after.
     * `notify_schedule`: The notify schedule.
     *
     * Returns a new `NotificationJob`.
     */
    pub fn new(
        status: &MonitorStatusType,
        url: &str,
        recipients: Vec<String>,
        from: &str,
        reply_to: &str,
        resend_after: i64,
        notify_schedule: &str,
    ) -> Result<NotificationJob, ApplicationError> {
        let transport = SmtpTransport::from_url(url)
            .map_err(|err| {
                ApplicationError::new(&format!("Error creating smtp transport: {err:?}"))
            })?
            .build();

        let from: Mailbox = from
            .parse()
            .map_err(|err| ApplicationError::new(&format!("Error parsing from email: {err:?}")))?;

        let reply_to: Mailbox = reply_to.parse().map_err(|err| {
            ApplicationError::new(&format!("Error parsing reply to email: {err:?}"))
        })?;

        Ok(NotificationJob {
            status: status.clone(),
            already_notified: HashMap::new(),
            recipients,
            transport,
            from,
            reply_to,
            resend_after,
            notify_schedule: notify_schedule.to_string(),
        })
    }

    /**
     * Get the notification job.
     *
     * # Returns
     * The notification job.
     */
    pub fn get_notification_job(&mut self) -> Result<Job, ApplicationError> {
        let mut notification_job = self.clone();
        let job_result = Job::new(self.notify_schedule.as_str(), move |_uuid, _locked| {
            let _ = notification_job.check().map_err(|err| {
                error!("Error checking notification job: {:?}", err);
            });
        });
        job_result.map_err(|err| {
            ApplicationError::new(&format!("Error creating db cleanup job: {err:?}"))
        })
    }

    /**
     * Check the notification job.
     *
     * # Returns
     * The result of the notification job.
     */
    fn check(&mut self) -> Result<(), ApplicationError> {
        self.remove_old_notifications();
        let new_errors = self.get_new_errors()?;
        if !new_errors.is_empty() {
            self.notify(new_errors);
        }
        Ok(())
    }

    fn get_new_errors(&mut self) -> Result<Vec<MonitorStatus>, ApplicationError> {
        let mut new_errors: Vec<MonitorStatus> = vec![];
        let status = self.status.lock();
        let statuses = status
            .map_err(|err| ApplicationError::new(&format!("Error getting status: {err:?}")))?;
        for (_name, monitor_status) in statuses.iter() {
            let current_monitor_status = monitor_status.status.clone();
            if let Status::Error { message: _ } = current_monitor_status {
                let name = monitor_status.name.clone();
                self.already_notified.entry(name).or_insert_with(|| {
                    new_errors.push(monitor_status.clone());
                    Utc::now()
                });
            }
        }
        Ok(new_errors)
    }

    /**
     * Remove old notifications
     *
     * If a notification is older than 2 hours, remove it.
     */
    fn remove_old_notifications(&mut self) {
        let older_than = Utc::now() - chrono::Duration::minutes(self.resend_after);
        let mut keys_to_remove = vec![];
        for (key, value) in &self.already_notified {
            if value < &older_than {
                keys_to_remove.push(key.to_string());
            }
        }
        for key in keys_to_remove {
            self.already_notified.remove(&key);
        }
    }

    /**
     * Notify the user of the new errors.
     *
     * `new_errors`: The new errors.
     */
    fn notify(&self, new_errors: Vec<MonitorStatus>) {
        let mut notification = "New errors: \n".to_string();
        for monitor_status in new_errors {
            notification.push_str(Self::get_notification_message(&monitor_status).as_str());
        }
        for recipient in &self.recipients {
            let email = Message::builder()
                .from(self.from.clone())
                .reply_to(self.reply_to.clone())
                .to(recipient.parse().unwrap())
                .subject("Monitoring agent daemon notification")
                .body(notification.clone())
                .unwrap();

            self.transport.send(&email).unwrap();
        }
    }

    /**
     * Get the notification message.
     *
     * `monitor_status`: The monitor status.
     *
     * Returns: The notification message.
     */
    fn get_notification_message(monitor_status: &MonitorStatus) -> String {
        format!(
            "Monitor: {:?} is in error. \nMessage: {:?} \n",
            monitor_status.name, monitor_status.status
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn test_get_new_errors() {
        let mut notification_job = NotificationJob {
            status: Arc::new(Mutex::new(HashMap::new())),
            already_notified: HashMap::new(),
            recipients: vec![],
            transport: SmtpTransport::unencrypted_localhost(),
            from: "test@test.com".parse().unwrap(),
            reply_to: "test@test.com".parse().unwrap(),
            resend_after: 120,
            notify_schedule: "".to_string(),
        };
        notification_job.status.lock().unwrap().insert(
            "test".to_string(),
            MonitorStatus {
                name: "test".to_string(),
                status: Status::Error {
                    message: "test".to_string(),
                },
                description: Some("test".to_string()),
                last_successful_time: None,
                last_error: None,
                last_error_time: None,
            },
        );
        let new_errors = notification_job.get_new_errors().unwrap();
        assert_eq!(new_errors.len(), 1);
        let new_errors = notification_job.get_new_errors().unwrap();
        assert_eq!(new_errors.len(), 0);
    }

    #[test]
    fn test_remove_old_notifications() {
        let mut notification_job = NotificationJob {
            status: Arc::new(Mutex::new(HashMap::new())),
            already_notified: HashMap::new(),
            recipients: vec![],
            transport: SmtpTransport::unencrypted_localhost(),
            from: "test@test.com".parse().unwrap(),
            reply_to: "test@test.com".parse().unwrap(),
            resend_after: 120,
            notify_schedule: "".to_string(),
        };
        notification_job
            .already_notified
            .insert("test".to_string(), Utc::now());
        notification_job.remove_old_notifications();
        assert_eq!(notification_job.already_notified.len(), 1);
        notification_job.already_notified.insert(
            "test".to_string(),
            Utc::now() - chrono::Duration::minutes(121),
        );
        notification_job.remove_old_notifications();
        assert_eq!(notification_job.already_notified.len(), 0);
        notification_job.already_notified.insert(
            "test".to_string(),
            Utc::now() - chrono::Duration::minutes(118),
        );
        notification_job.remove_old_notifications();
        assert_eq!(notification_job.already_notified.len(), 1);
    }
}
