use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, sync::{Arc, Mutex}};
use std::str::FromStr;

use log::error;

use crate::common::{ApplicationError, ProcsCpuinfo};

/**
 * Cpuinfo.
 * 
 * This struct represents the cpuinfo data.
 * 
 */
#[derive(Debug, Clone)]
pub struct Cpuinfo {
    pub procsdata: Arc<Mutex<Vec<ProcsCpuinfo>>>,
}

impl Cpuinfo {
    /**
     * Create a new `Cpuinfo`.
     * 
     * Returns a new `Cpuinfo`.
     */
    pub fn new() -> Cpuinfo {
        Cpuinfo { 
            procsdata: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /**
     * Get and set the cpuinfo data.
     * 
     * Returns error if there is an error getting the lock or reading the data.
     */
    pub async fn get_and_set_cpuinfo(self) -> Result<(), ApplicationError> {
        let lock = self.procsdata.lock();
        match lock {
            Ok(mut procsdata) => {
                let data = Cpuinfo::read_cpuinfo("/proc/cpuinfo");
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
     * Read the cpuinfo file.
     * 
     * `file`: The file to read.
     * 
     * Returns the cpuinfo data or an error.
     */
    pub fn read_cpuinfo(file: &str) -> Result<Vec<ProcsCpuinfo>, ApplicationError> {
        let mut cpuinfo_data: Vec<ProcsCpuinfo> = Vec::new();
        let cpuinfo_file = File::open(file);
        match cpuinfo_file {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut parts: HashMap<String, String> = HashMap::new();                
                for line in reader.lines() {
                    let line = Cpuinfo::get_line(line)?;                    
                    if line.is_empty() {
                        let cpuinfo = ProcsCpuinfo::new(
                            parts.get("apicid").and_then(|f| u8::from_str(f).ok()),
                            parts.get("vendor_id").cloned(),
                            parts.get("cpu family").cloned(),
                            parts.get("model").cloned(),
                            parts.get("model name").cloned(),
                            parts.get("cpu cores").and_then(|f| u8::from_str(f).ok()),
                            parts.get("cpu MHz").and_then(|f| f32::from_str(f).ok()),
                        );
                        cpuinfo_data.push(cpuinfo);
                        parts.clear();
                    } else {
                        let parts_data: Vec<&str> = line.split(':').collect();
                        if parts_data.len() == 2 {
                            parts.insert(parts_data[0].trim().to_string(), parts_data[1].trim().to_string());
                        } 
                    }                                                                
                }
            },
            Err(err) => {
                error!("Error reading cpuinfo: {:?}", err);
                return Err(ApplicationError::new("Error reading cpuinfo"));
            }
        }
        Ok(cpuinfo_data)
    }

    /**
     * Get a line from the cpuinfo file.
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
                Err(ApplicationError::new(format!("Error reading line: {:?}", err).as_str()))
            }
        }
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_current_and_set_values() {
        let cpuinfo = Cpuinfo::new();
        let binding = cpuinfo.get_and_set_cpuinfo();
        assert!(binding.await.is_ok());
    }

    #[test]
    fn test_current() {
        let binding = Cpuinfo::read_cpuinfo("/proc/cpuinfo");
        assert!(binding.is_ok());
    }

    #[test]
    fn test_read_cpuinfo() {
        let binding = Cpuinfo::read_cpuinfo("resources/test/test_cpuinfo").unwrap();
        let first = binding.first().unwrap();
        assert_eq!(&first.apicid.unwrap(), &0);
        assert_eq!(&first.vendor_id.clone().unwrap(), "AuthenticAMD");
        assert_eq!(&first.cpu_family.clone().unwrap(), "25");
        assert_eq!(&first.model.clone().unwrap(), "116");
        assert_eq!(&first.model_name.clone().unwrap(), "AMD Ryzen 7 7840HS w/ Radeon 780M Graphics");
        assert_eq!(&first.cpu_cores.unwrap(), &8);
        assert_eq!(&first.cpu_mhz.unwrap(), &3404.518);
    }

}