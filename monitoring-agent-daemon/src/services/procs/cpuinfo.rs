use std::sync::{Arc, Mutex};

use log::error;
use monitoring_agent_lib::proc::ProcsCpuinfo;

use crate::common::ApplicationError;

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
    pub fn get_and_set_cpuinfo(self) -> Result<(), ApplicationError> {
        let lock = self.procsdata.lock();
        match lock {
            Ok(mut procsdata) => {
                let data = ProcsCpuinfo::get_cpuinfo();
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

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_current_and_set_values() {
        let cpuinfo = Cpuinfo::new();
        let binding = cpuinfo.get_and_set_cpuinfo();
        assert!(binding.is_ok());
    }

}