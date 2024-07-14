use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about="Monitoring tool", long_about = None)]
pub struct ApplicationArguments {
    /// Configuration file. Not required.
    #[arg(short, long, default_value = "/etc/monitoring/config.json")]
    pub config: String,

    /// Port to listen on. Not required.
    #[arg(short, long, default_value = "64000")]
    pub port: u16,

}