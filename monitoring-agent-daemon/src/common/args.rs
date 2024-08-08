use clap::Parser;

/**
 * Application arguments.
 *
 * This struct is used to parse command line arguments.
 *
 * config: Configuration file. Not required.
 *
 */
#[derive(Parser, Debug, Clone)]
#[command(version, about="Monitoring agent", long_about = None)]
pub struct ApplicationArguments {
    /// Configuration file.
    #[arg(short = 'c', long, default_value = "/etc/monitoring-agent-daemon/config.json")]
    pub config: String,

    /// log4rs logfile.
    #[arg(short = 'l', long, default_value = "/var/log/monitoring-agent-daemon/monitoring-agent.log")]
    pub logfile: String,

    /// Daemonize the application. Will not daemonize by default.
    #[arg(short = 'd', long, default_value = "false")]
    pub daemon: bool,

    /// Error level for stdout. Valid values are TRACE, DEBUG, INFO, WARN, ERROR.
    #[arg(long = "stderrlevel", default_value = "ERROR")]
    pub stdout_errorlevel: String,

    /// Error level for file logger. Valid values are TRACE, DEBUG, INFO, WARN, ERROR.
    #[arg(long = "fileerrlevel", default_value = "ERROR")]
    pub file_errorlevel: String,

    /// Test configuration. Will not test by default.
    #[arg(short = 't', long, default_value = "false")]
    pub test: bool,

    /// pid file. Only used when daemonizing the application.
    #[arg(short = 'p', long, default_value = "/var/run/monitoring-agent-daemon.pid")]
    pub pidfile: String,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_application_arguments() {
        let args = ApplicationArguments::try_parse_from(&["monitoring-agent-daemon", "-c", "/etc/monitoring-agent-daemon/config.json", "-d", "-t", "-p", "/var/run/monitoring-agent-daemon.pid"]).unwrap();
        assert_eq!(args.config, "/etc/monitoring-agent-daemon/config.json");
        assert_eq!(args.logfile, "/var/log/monitoring-agent-daemon/monitoring-agent.log");
        assert_eq!(args.daemon, true);
        assert_eq!(args.test, true);
        assert_eq!(args.pidfile, "/var/run/monitoring-agent-daemon.pid");
    }


    #[test]
    fn test_application_default_arguments() {
        let args = ApplicationArguments::parse_from(&["monitoring-agent-daemon"]);
        assert_eq!(args.config, "/etc/monitoring-agent-daemon/config.json");
        assert_eq!(args.logfile, "/var/log/monitoring-agent-daemon/monitoring-agent.log");
        assert_eq!(args.daemon, false);
        assert_eq!(args.test, false);
        assert_eq!(args.pidfile, "/var/run/monitoring-agent-daemon.pid");
    }

}
