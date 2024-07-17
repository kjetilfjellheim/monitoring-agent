use clap::Parser;

/**
 * Application arguments. 
 * 
 * This struct is used to parse command line arguments. 
 *
 * config: Configuration file. Not required.
 * 
 */
#[derive(Parser, Debug)]
#[command(version, about="Monitoring tool", long_about = None)]
pub struct ApplicationArguments {
    /// Configuration file. Not required.
    #[arg(short, long, default_value = "/etc/monitoring/config.json")]
    pub config: String,

}
