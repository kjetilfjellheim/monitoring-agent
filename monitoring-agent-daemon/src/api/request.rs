use chrono::{DateTime, Utc};
use serde::Deserialize;

/**
 * The historical parameters. Used to represent the historical parameters.
 * 
 * `from_datetime`: The from date time.
 * `to_datetime`: The to date time.
 * `split`: The split. One is every minute 2 is every two minutes.
 */
#[derive(Debug, Deserialize)]
pub struct HistoricalParams {
    /// The from date time.
    #[serde(rename = "fromDateTime", default = "default_from_datetime")]
    pub from_datetime: DateTime<Utc>,
    /// The to date time.
    #[serde(rename = "toDateTime", default = "Utc::now")]
    pub to_datetime: DateTime<Utc>,
    /// The split. One is every minute 2 is every two minutes.
    #[serde(rename = "split", default = "default_split")]
    pub split: u16,
    /// The server.
    #[serde(rename = "server", default = "default_server")]
    pub server: String,    
}

/**
 * The default from date time.
 */
fn default_from_datetime() -> DateTime<Utc> {
    Utc::now() - chrono::Duration::days(1)
}

/**
 * The default split.
 */
fn default_split() -> u16 {
    1
}

/**
 * The default server.
 */
fn default_server() -> String {
    "localhost".to_string()
}