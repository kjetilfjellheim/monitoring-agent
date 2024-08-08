use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}};
use std::str::FromStr;

use log::error;

use crate::common::CommonLibError;

/**
 * Memory information from /cat/meminfo
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ProcsMeminfo {
    /// The total memory.
    pub memtotal: Option<u64>,
    /// The free memory.
    pub memfree: Option<u64>,
    /// The available memory.
    pub memavailable: Option<u64>,
    /// The total swap.
    pub swaptotal: Option<u64>,
    /// The free swap.
    pub swapfree: Option<u64>,
}

impl ProcsMeminfo {

    /**
     * Create a new `Meminfo`.
     *
     * ```
     * use monitoring_agent_lib::proc::meminfo::ProcsMeminfo;
     * ProcsMeminfo::new(None, None, None, None, None);
     * ```
     * ```
     * use monitoring_agent_lib::proc::meminfo::ProcsMeminfo;
     * ProcsMeminfo::new(Some(15_538_476), Some(1_286_156), Some(4_491_376), Some(1_998_844), Some(13_952));
     * ```
     * 
     * `memtotal`: The total memory.
     * `memfree`: The free memory.
     * `memavailable`: The available memory.
     * `swaptotal`: The total swap.
     * `swapfree`: The free swap.
     * 
     * Returns a new `ProcsMeminfo`.
     */
    #[must_use] pub fn new(
        memtotal: Option<u64>,
        memfree: Option<u64>,
        memavailable: Option<u64>,
        swaptotal: Option<u64>,
        swapfree: Option<u64>,
    ) -> ProcsMeminfo {
        ProcsMeminfo {
            memtotal,
            memfree,
            memavailable,
            swaptotal,
            swapfree,
        }
    }

    /**
     * Get the memory of the system.
     * 
     * ```
     * use monitoring_agent_lib::proc::meminfo::ProcsMeminfo;
     * ProcsMeminfo::get_meminfo();
     * ```
     * 
     * 
     * Returns the cpuinfo data or an error.
     * 
     * # Errors
     *  - If there is an error reading the meminfo file.
     *  - If there is an error reading a line from the meminfo file.
     *  - If there is an error parsing the data from the meminfo file.
     */
    #[tracing::instrument(level = "debug")]
    pub fn get_meminfo() -> Result<ProcsMeminfo, CommonLibError> {
        let meminfo_file = "/proc/meminfo";
        ProcsMeminfo::read_meminfo(meminfo_file)
    }

    /**
     * Read the meminfo file.
     * 
     * `file`: The file to read.
     * 
     * Returns the meminfo data or an error.
     * 
     * # Errors
     *  - If there is an error reading the meminfo file.
     *  - If there is an error reading a line from the meminfo file.
     *  - If there is an error parsing the data from the meminfo file.
     */
    fn read_meminfo(file: &str) -> Result<ProcsMeminfo, CommonLibError> {
        let meminfo_file = File::open(file);
        let mut parts: HashMap<String, String> = HashMap::new();                
        match meminfo_file {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = ProcsMeminfo::get_line(line)?;                    
                    let parts_data: Vec<&str> = line.split(':').collect();
                    if parts_data.len() == 2 {
                        parts.insert(parts_data[0].trim().to_string(), parts_data[1].replace("kB", "").trim().to_string());
                    }                                                                                    
                }
            },
            Err(err) => {
                error!("Error reading meminfo: {err:?}");
                return Err(CommonLibError::new(format!("Error reading meminfo: {err:?}").as_str()));
            }
        }
        Ok(ProcsMeminfo::new(   
            parts.get("MemTotal").and_then(|f| u64::from_str(f).ok()),
            parts.get("MemFree").and_then(|f| u64::from_str(f).ok()),
            parts.get("MemAvailable").and_then(|f| u64::from_str(f).ok()),
            parts.get("SwapTotal").and_then(|f| u64::from_str(f).ok()),
            parts.get("SwapFree").and_then(|f| u64::from_str(f).ok()),
        ))


    }

    /**
     * Get a line from the meminfo file.
     * 
     * `line`: The line to get.
     * 
     * Returns the line or an error.
     * 
     * # Errors
     * - If there is an error reading a line from the meminfo file.
     * 
     */
    fn get_line(line: Result<String, std::io::Error>) -> Result<String, CommonLibError> {
        match line {
            Ok(line) => {
                Ok(line)
            },
            Err(err) => {
                Err(CommonLibError::new(format!("Error reading line: {err:?}").as_str()))
            }
        }
    }


    /**
     * Calculate the percentage.
     * 
     * `free`: Free memory 
     * `total`: Total memory 
     * 
     * Returns: Percentage or none
     */
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn get_percent_used(free: Option<u64>, total: Option<u64>) -> Option<f64> {
        let free = free?;
        let total = total?;   
        Some(100f64 - ((free as f64 / total as f64) * 100f64))      
    }    
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_current() {
        let binding = ProcsMeminfo::get_meminfo();
        assert!(binding.is_ok());
    }

    #[test]
    fn test_read_predefined_meminfo() {
        let binding = ProcsMeminfo::read_meminfo("resources/test/test_meminfo").unwrap();
        assert_eq!(binding.memtotal, Some(15_538_476));
        assert_eq!(binding.memfree, Some(1_286_156));
        assert_eq!(binding.memavailable, Some(4_491_376));
        assert_eq!(binding.swaptotal, Some(1_998_844));
        assert_eq!(binding.swapfree, Some(13_952));
    }

    #[test]
    fn test_get_percent_used() {
        let result = ProcsMeminfo::get_percent_used(None, None);
        assert_eq!(result, None);
        let result = ProcsMeminfo::get_percent_used(Some(32000), None);
        assert_eq!(result, None);  
        let result = ProcsMeminfo::get_percent_used(None, Some(32000));
        assert_eq!(result, None); 
        let result = ProcsMeminfo::get_percent_used(Some(25000), Some(100000));
        assert_eq!(result, Some(75f64));                         
    }

}