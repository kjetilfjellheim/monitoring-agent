use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, sync::{Arc, Mutex}};
use std::str::FromStr;

use log::error;

use crate::common::{ApplicationError, ProcsMeminfo};

/**
 * Meminfo.
 * 
 * This struct represents the meminfo data.
 * 
 */
#[derive(Debug, Clone)]
pub struct Meminfo {
    pub procsdata: Arc<Mutex<ProcsMeminfo>>,
}

impl Meminfo {
    /**
     * Create a new `Meminfo`.
     * 
     * Returns a new `Meminfo`.
     */
    pub fn new() -> Meminfo {
        Meminfo { 
            procsdata: Arc::new(Mutex::new(ProcsMeminfo::new(None, None, None, None, None))),
        }
    }

    /**
     * Get and set the meminfo data.
     * 
     * Returns error if there is an error getting the lock or reading the data.
     */
    pub fn get_and_set_meminfo(self) -> Result<(), ApplicationError> {
        let lock = self.procsdata.lock();
        match lock {
            Ok(mut procsdata) => {
                let data = Meminfo::read_meminfo("/proc/meminfo");
                match data {
                    Ok(data) => {
                        *procsdata = data;
                    },
                    Err(err) => {
                        error!("Error reading cpuinfo: {err:?}");
                        return Err(ApplicationError::new("Error getting procsdata lock"));
                    }
                }
            },
            Err(err) => {
                error!("Error getting procsdata lock: {err:?}");
                return Err(ApplicationError::new("Error getting procsdata lock"));
            }
        };        
        Ok(())
    }       

    /**
     * Read the meminfo file.
     * 
     * `file`: The file to read.
     * 
     * Returns the meminfo data or an error.
     */
    pub fn read_meminfo(file: &str) -> Result<ProcsMeminfo, ApplicationError> {
        let meminfo_file = File::open(file);
        let mut parts: HashMap<String, String> = HashMap::new();                
        match meminfo_file {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = Meminfo::get_line(line)?;                    
                    let parts_data: Vec<&str> = line.split(':').collect();
                    if parts_data.len() == 2 {
                        parts.insert(parts_data[0].trim().to_string(), parts_data[1].replace("kB", "").trim().to_string());
                    }                                                                                    
                }
            },
            Err(err) => {
                error!("Error reading meminfo: {:?}", err);
                return Err(ApplicationError::new("Error reading meminfo"));
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
     */
    fn get_line(line: Result<String, std::io::Error>) -> Result<String, ApplicationError> {
        match line {
            Ok(line) => {
                Ok(line)
            },
            Err(err) => {
                Err(ApplicationError::new(format!("Error reading line: {err:?}").as_str()))
            }
        }
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_current_and_set_values() {
        let meminfo = Meminfo::new();
        let binding = meminfo.get_and_set_meminfo();
        assert!(binding.is_ok());
    }

    #[test]
    fn test_current() {
        let binding = Meminfo::read_meminfo("/proc/meminfo");
        assert!(binding.is_ok());
    }

    #[test]
    fn test_read_cpuinfo() {
        let binding = Meminfo::read_meminfo("resources/test/test_meminfo").unwrap();
        assert_eq!(binding.memtotal, Some(15538476));
        assert_eq!(binding.memfree, Some(1286156));
        assert_eq!(binding.memavailable, Some(4491376));
        assert_eq!(binding.swaptotal, Some(1998844));
        assert_eq!(binding.swapfree, Some(13952));
    }

}