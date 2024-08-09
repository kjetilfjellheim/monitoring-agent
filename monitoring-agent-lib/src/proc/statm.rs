use std::{fs::File, io::BufReader, io::BufRead};

use crate::common::CommonLibError;
use log::error;

/**
 * Process memory structure from /proc/{pid}/statm
 * 
 * TODO: Add more detailed text.
 */
#[derive(Debug, Clone)]
pub struct Statm {
    /// Total program size (pages)
    size: Option<u32>,
    /// Size of memory portions (pages)
    resident: Option<u32>,
    /// Number of pages that are shared
    share: Option<u32>,
    /// Number of pages that are ‘code’
    trs: Option<u32>,
    /// Number of pages of data/stack
    drs: Option<u32>,
    /// Number of pages of library
    lrs: Option<u32>,
    /// Number of dirty pages
    dt: Option<u32>
}

impl Statm {
    /**
     * Create a new `Statm`.
     * 
     * `size`: Total program size (pages)
     * `resident`: Size of memory portions (pages)
     * `share`: Number of pages that are shared
     * `trs`: Number of pages that are ‘code’
     * `drs`: Number of pages of data/stack
     * `lrs`: Number of pages of library
     * `dt`: Number of dirty pages
     * 
     * Returns a new `Statm`.
     */
    #[must_use] pub fn new(
        size: &Option<u32>,
        resident: &Option<u32>,
        share: &Option<u32>,
        trs: &Option<u32>,
        drs: &Option<u32>,
        lrs: &Option<u32>,
        dt: &Option<u32>
    ) -> Statm {
        Statm {
            size: size.clone(),
            resident: resident.clone(),
            share: share.clone(),
            trs: trs.clone(),
            drs: drs.clone(),
            lrs: lrs.clone(),
            dt: dt.clone()
        }
    }   

    /**
     * Get the memory use of the process.
     * 
     * ```
     * use monitoring_agent_lib::proc::statm::Statm;
     * Statm::get_meminfo();
     * ```
     * 
     * Returns the statm data or an error.
     * 
     * # Errors
     *  - If there is an error reading the meminfo file.
     *  - If there is an error reading a line from the meminfo file.
     *  - If there is an error parsing the data from the meminfo file.
     */
    #[tracing::instrument(level = "debug")]
    pub fn get_statm(pid: u32) -> Result<Statm, CommonLibError> {
        let statm_file = File::open("/proc/".to_string() + pid.to_string().as_str() + "/statm").map_err(|err| {
            CommonLibError::new(format!("Error reading statm file: {err:?}").as_str())
        })?;
        Statm::read_statm(statm_file)
    }
    
    /**
     * Read the statm file. 
     * If any data field is not parseable, it will be set to None.
     * 
     * `file`: The file to read.
     * 
     * Returns the statm data or an error.
     * 
     * # Errors
     * - If there is an error reading the statm file.
     */
    fn read_statm(file: File) -> Result<Statm, CommonLibError> {
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        let data = reader.read_line(&mut buffer);   
        match data {
            Ok(_) => {
                Statm::handle_statm_file(buffer)
            },
            Err(err) => {
                return Err(CommonLibError::new(format!("Error reading statm: {err:?}").as_str()));
            }
        }     
    }

    /**
     * Handle the statm file.
     * 
     * `buffer`: The buffer to parse.
     * 
     * Returns the statm data or an error.
     */
    fn handle_statm_file(buffer: String) -> Result<Statm, CommonLibError> {
        let cols = buffer.split_whitespace().collect::<Vec<&str>>();
        let size = cols[0].parse::<u32>().ok();
        let resident = cols[1].parse::<u32>().ok();
        let share = cols[2].parse::<u32>().ok();
        let trs = cols[3].parse::<u32>().ok();
        let lrs: Option<u32> = cols[4].parse::<u32>().ok();
        let drs = cols[5].parse::<u32>().ok();
        let dt = cols[6].parse::<u32>().ok();
        Ok(Statm::new(&size, &resident, &share, &trs, &drs, &lrs, &dt))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_statm_from_buffer1() {
        let buffer = "5805 3442 2354 11 0 1082 0";
        let statm = Statm::handle_statm_file(buffer.to_string());
        assert_eq!(statm.as_ref().unwrap().size, Some(5805));
        assert_eq!(statm.as_ref().unwrap().resident, Some(3442));
        assert_eq!(statm.as_ref().unwrap().share, Some(2354));
        assert_eq!(statm.as_ref().unwrap().trs, Some(11));
        assert_eq!(statm.as_ref().unwrap().drs, Some(1082));
        assert_eq!(statm.as_ref().unwrap().lrs, Some(0));
        assert_eq!(statm.as_ref().unwrap().dt, Some(0));
    }
    
    #[test]
    fn test_handle_statm_buffer2() {
        let buffer = "494524 21506 2214 1471 0 59164 0";
        let statm = Statm::handle_statm_file(buffer.to_string());
        assert_eq!(statm.as_ref().unwrap().size, Some(494524));
        assert_eq!(statm.as_ref().unwrap().resident, Some(21506));
        assert_eq!(statm.as_ref().unwrap().share, Some(2214));
        assert_eq!(statm.as_ref().unwrap().trs, Some(1471));
        assert_eq!(statm.as_ref().unwrap().drs, Some(59164));
        assert_eq!(statm.as_ref().unwrap().lrs, Some(0));
        assert_eq!(statm.as_ref().unwrap().dt, Some(0));
    }    

    #[test]
    fn test_handle_statm_pid_1() {
        let statm = Statm::get_statm(1);
        assert!(statm.is_ok());
    }  

    #[test]
    fn test_handle_statm_pid_0() {
        let statm = Statm::get_statm(0);
        assert!(statm.is_err());
    }    
}