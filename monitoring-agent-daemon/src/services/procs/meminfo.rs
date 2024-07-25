use std::sync::{Arc, Mutex};

use log::error;
use monitoring_agent_lib::proc::ProcsMeminfo;

use crate::common::ApplicationError;

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
                let data = ProcsMeminfo::get_meminfo();
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
        let meminfo = Meminfo::new();
        let binding = meminfo.get_and_set_meminfo();
        assert!(binding.is_ok());
    }

}