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


/**
 * The meminfo element. Used to represent a memory use element.
 * 
 * `timestamp`: The timestamp.
 * `total`: The total memory.
 * `freemem`: The free memory.
 */
#[allow(clippy::similar_names)]
#[derive(Debug, Clone)]
pub struct MeminfoElement {
    /// The timestamp.    
    pub timestamp: DateTime<Utc>,
    /// The free memory.
    pub freemem: u64,
}

impl MeminfoElement {
    /**
     * Create a new meminfo element.
     * 
     * `timestamp`: The timestamp.
     * `total`: The total memory.
     * `freemem`: The free memory.
     * 
     * Returns the meminfo element.
     */
    #[allow(clippy::similar_names)]
    pub fn new(timestamp: DateTime<Utc>, freemem: u64) -> MeminfoElement {
        MeminfoElement {
            timestamp,
            freemem,
        }
    }
}

/**
 * The used process memory element.
 * 
 * `timestamp`: The timestamp.
 * `resident`: Size of memory portions (pages)
 * `share`: Number of pages that are shared
 * `trs`: Number of pages that are ‘code’
 * `drs`: Number of pages of data/stack
 * `lrs`: Number of pages of library
 * `dt`: Number of dirty pages
 */
#[allow(clippy::similar_names)]
#[derive(Debug, Clone)]
pub struct ProcessMemoryElement {
    /// The timestamp.    
    pub timestamp: DateTime<Utc>,
    /// Size of memory portions (pages)
    pub resident: Option<u64>,
    /// Number of pages that are shared
    pub share: Option<u64>,
    /// Number of pages that are ‘code’
    pub trs: Option<u64>,
    /// Number of pages of data/stack
    pub drs: Option<u64>,
    /// Number of pages of library
    pub lrs: Option<u64>,
    /// Number of dirty pages
    pub dt: Option<u64>,
}

impl ProcessMemoryElement {
    /**
     * Create a new process memory element.
     * 
     * `timestamp`: The timestamp.
     * `resident`: Size of memory portions (pages)
     * `share`: Number of pages that are shared
     * `trs`: Number of pages that are ‘code’
     * `drs`: Number of pages of data/stack
     * `lrs`: Number of pages of library
     * `dt`: Number of dirty pages
     * 
     * Returns the process memory element.
     */
    #[allow(clippy::similar_names)]
    pub fn new(timestamp: DateTime<Utc>, resident: Option<u64>, share: Option<u64>, trs: Option<u64>, drs: Option<u64>, lrs: Option<u64>, dt: Option<u64>) -> ProcessMemoryElement {
        ProcessMemoryElement {
            timestamp,
            resident,
            share,
            trs,
            drs,
            lrs,
            dt,
        }
    }
}