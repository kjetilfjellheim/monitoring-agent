use std::{fs::File, io::{BufRead, BufReader}};
use std::str::FromStr;

use log::error;
use serde::{Deserialize, Serialize};

use crate::common::CommonLibError;
/**
 * Load average information from /cat/loadavg
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcsLoadavg {
    /// The load average for the last minute.    
    pub loadavg1min: Option<f32>,
    /// The load average for the last 5 minutes.
    pub loadavg5min:  Option<f32>,
    #[allow(clippy::similar_names)]
    /// The load average for the last 10 minutes.
    pub loadavg10min:  Option<f32>,
    /// The number of currently running processes.
    pub current_running_processes: Option<u32>,
    /// The total number of processes.
    pub total_number_of_processes: Option<u32>
}

impl ProcsLoadavg {

    /**
     * Create a new `ProcsLoadavg`.
     *
     * `loadavg1min`: The load average for the last minute.
     * `loadavg5min`: The load average for the last 5 minutes.
     * `loadavg10min`: The load average for the last 10 minutes.
     * `current_running_processes`: The number of currently running processes.
     * `total_number_of_processes`: The total number of processes.
     * 
     * Returns a new `ProcsLoadavg`.
     * 
     */
    #[allow(clippy::similar_names)]
    #[must_use] pub fn new( loadavg1min: Option<f32>,
                            loadavg5min:  Option<f32>,
                            loadavg10min:  Option<f32>,
                            current_running_processes: Option<u32>,
                            total_number_of_processes: Option<u32>
    ) -> Self {
        ProcsLoadavg {
            loadavg1min,
            loadavg5min,
            loadavg10min,
            current_running_processes,
            total_number_of_processes
        }
    }

    /**
     * Get the loadavg of the cpu.
     * 
     * # Errors
     *  - If there is an error reading the loadavg file.
     *  - If there is an error reading a line from the loadavg file.
     *  - If there is an error parsing the data from the loadavg file.
     */
    pub fn get_loadavg() -> Result<ProcsLoadavg, CommonLibError> {
        let loadavg_file = "/proc/loadavg";
        ProcsLoadavg::read_loadavg(loadavg_file)
    }    

    /**
     * Read the loadavg file.
     * 
     * `file`: The file to read.
     * 
     * Returns the loadavg data or an error.
     * 
     * # Errors
     *  - If there is an error reading the loadavg file.
     *  - If there is an error reading a line from the loadavg file.
     *  - If there is an error parsing the data from the loadavg file.
     */
    fn read_loadavg(file: &str) -> Result<ProcsLoadavg, CommonLibError> {
        let loadavg_file = File::open(file);
        match loadavg_file {
            Ok(file) => {
                handle_loadavg_file(file)
            },
            Err(err) => {
                error!("Error reading meminfo: {err:?}");
                return Err(CommonLibError::new(format!("Error reading meminfo: {err:?}").as_str()));
            }
        }
    }

 }

 /**
  * Handle the loadavg file.
  * 
  * `file`: The file to read.
  * 
  * Returns the loadavg data or an error.
  * 
  * # Errors
  *  - If there is an error reading the loadavg file.
  *  - If there is an error reading a line from the loadavg file.
  *  - If there is an error parsing the data from the loadavg file.
  */
fn handle_loadavg_file(file: File) -> Result<ProcsLoadavg, CommonLibError> {
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let data = reader.read_line(&mut buffer);
    match data {
        Ok(_) => {
            let main_cols = buffer.split_whitespace().collect::<Vec<&str>>();
            let process_parts = match main_cols.get(3) {
                Some(process_parts) => process_parts.split('/').collect::<Vec<&str>>(),
                None => return Err(CommonLibError::new("Error parsing loadavg")),
            };
            Ok(ProcsLoadavg::new(
                main_cols.first().and_then(|f| f32::from_str(f).ok()),
                main_cols.get(1).and_then(|f| f32::from_str(f).ok()),
                main_cols.get(2).and_then(|f| f32::from_str(f).ok()),
                process_parts.first().and_then(|f| u32::from_str(f).ok()),
                process_parts.get(1).and_then(|f| u32::from_str(f).ok()),
            ))
        },
        Err(err) => {
            error!("Error reading meminfo: {err:?}");
            return Err(CommonLibError::new(format!("Error reading meminfo: {err:?}").as_str()));
        }
    }
}

#[cfg(test)]
mod test {
    
        use super::*;
    
        #[test]
        fn test_current() {
            let binding = ProcsLoadavg::get_loadavg();
            assert!(binding.is_ok());
        }
    
        #[test]
        fn test_read_predefined_cpuinfo() {
            let binding = ProcsLoadavg::read_loadavg("resources/test/test_loadavg").unwrap();
            assert_eq!(&binding.loadavg1min.unwrap(), &0.59);
            assert_eq!(&binding.loadavg5min.clone().unwrap(), &0.63);
            assert_eq!(&binding.loadavg10min.clone().unwrap(), &0.32);
            assert_eq!(&binding.current_running_processes.clone().unwrap(), &1);
            assert_eq!(&binding.total_number_of_processes.clone().unwrap(), &1419);
        }
}