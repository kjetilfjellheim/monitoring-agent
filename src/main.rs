mod common;
mod config;
mod monitoring;

use std::fs::OpenOptions;

use clap::Parser;
use daemonize::Daemonize;

use crate::config::ApplicationArguments;
use crate::monitoring::MonitoringService;

fn main() {
    /*
     * Parse command line arguments.
     */
    let args = ApplicationArguments::parse();

    /*
     * stdout out and stderr file descriptors.
     */
    let stdout = match OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open("/var/log/monitoring_agent.out")
    {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening stdout file: {:?}", err);
            return;
        }
    };
    let stderr = match OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open("/var/log/monitoring_agent.err")
    {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening stderr file: {:?}", err);
            return;
        }
    };
    /*
     * Daemonize the application.
     */
    let daemonize = Daemonize::new()
        .pid_file("/tmp/monitoring_agent.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .umask(0770)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(move || {
            let mut monitoring_service = MonitoringService::new();
            match monitoring_service.start(&args) {
                Ok(_) => {
                    println!("Monitoring service started!");
                }
                Err(err) => {
                    eprintln!("Error starting monitoring service: {:?}", err.message);
                }
            }
        });

    /*
     * Start the daemon.
     */
    match daemonize.start() {
        Ok(_) => {
            println!("Started daemon!");
        }
        Err(err) => {
            eprintln!("Error starting daemon: {:?}", err);
        }
    }
}
