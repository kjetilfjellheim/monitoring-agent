use chrono::{DateTime, Utc};

/**
 * The load average element. Used to represent a load average element.
 * 
 * `timestamp`: The timestamp.
 * `loadavg1min`: The 1 minute load average.
 * `loadavg5min`: The 5 minute load average.
 * `loadavg15min`: The 15 minute load average.
 */
#[allow(clippy::similar_names)]
#[derive(Debug, Clone)]
pub struct LoadavgElement {
    /// The timestamp.    
    pub timestamp: DateTime<Utc>,
    /// The 1 minute load average.
    pub loadavg1min: f64,
    /// The 5 minute load average.
    pub loadavg5min: f64,
    /// The 15 minute load average.
    pub loadavg15min: f64,
}

impl LoadavgElement {
    /**
     * Create a new load average element.
     * 
     * `timestamp`: The timestamp.
     * `loadavg1min`: The 1 minute load average.
     * `loadavg5min`: The 5 minute load average.
     * `loadavg15min`: The 15 minute load average.
     * 
     * Returns the load average element.
     */
    #[allow(clippy::similar_names)]
    pub fn new(timestamp: DateTime<Utc>, loadavg1min: f64, loadavg5min: f64, loadavg15min: f64) -> LoadavgElement {
        LoadavgElement {
            timestamp,
            loadavg1min,
            loadavg5min,
            loadavg15min,
        }
    }
}