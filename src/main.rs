mod common;
mod services;

use clap::Parser;
use daemonize::Daemonize;
use log::{error, info};
use log4rs::config::Deserializers;
use std::fs::OpenOptions;

use crate::common::ApplicationArguments;
use crate::services::MonitoringService;

fn main() {
    /*
     * Parse command line arguments.
     */
    let args: ApplicationArguments = ApplicationArguments::parse();

    match log4rs::init_file(&args.loggingfile, Deserializers::default()) {
        Ok(()) => {
            info!("Logging initialized!");
        }
        Err(err) => {
            error!("Error initializing logging: {:?}", err);
        }
    }
    /*
     * Start appliction in daemon or non daemon mode.
     */
    if args.daemon {
        daemonize_application(args);
    } else {
        normal_application(&args);
    }
}
/**
 * Start the application in non daemon mode.
 *
 * @param args Application arguments.
 *
 */
fn normal_application(args: &ApplicationArguments) {
    let mut monitoring_service = MonitoringService::new();
    match monitoring_service.start(&args.config, args.test) {
        Ok(()) => {
            info!("Monitoring service started!");
        }
        Err(err) => {
            error!("Error starting monitoring service: {:?}", err.message);
        }
    }
}
/**
 * Daemonize the application.
 *
 * @param args Application arguments.
 */
fn daemonize_application(args: ApplicationArguments) {
    /*
     * Open stdout for logging daemon output.
     */
    let stdout = match OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&args.stdout)
    {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening stdout file: {:?}", err);
            return;
        }
    };
    /*
     * Open stderr for logging daemon errors.
     */
    let stderr = match OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&args.stderr)
    {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening stderr file: {:?}", err);
            return;
        }
    };
    /*
     * Create daemonize object.
     */
    let daemonize = Daemonize::new()
        .pid_file(&args.pidfile)
        .chown_pid_file(true)
        .umask(770)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(move || {
            let mut monitoring_service = MonitoringService::new();
            match monitoring_service.start(&args.config, args.test) {
                Ok(()) => {
                    info!("Monitoring service started!");
                }
                Err(err) => {
                    error!("Error starting monitoring service: {:?}", err.message);
                }
            }
        });

    /*
     * Start the daemon.
     */
    match daemonize.start() {
        Ok(()) => {
            info!("Started daemon!");
        }
        Err(err) => {
            error!("Error starting daemon: {:?}", err);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_normal_application() {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_integration_test.json".to_string(),
            daemon: false,
            test: true,
            stdout: String::new(),
            stderr: String::new(),
            pidfile: String::new(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        super::normal_application(&args);
    }

    #[test]
    fn test_daemonize_application() {
        let args = ApplicationArguments {
            config: "./resources/test/test_full_integration_test.json".to_string(),
            daemon: true,
            test: true,
            stdout: "/tmp/monitoring-agent.out".to_string(),
            stderr: "/tmp/monitoring-agent.err".to_string(),
            pidfile: "/tmp/monitoring-agent.pid".to_string(),
            loggingfile: "./resources/test/logging.yml".to_string(),
        };
        super::daemonize_application(args);
    }
}
