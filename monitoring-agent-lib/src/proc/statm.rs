use std::{fs::File, io::BufReader, io::BufRead};

use crate::common::CommonLibError;

/**
 * Process memory structure from /proc/{pid}/statm
 * 
 * TODO: Add more detailed text.
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ProcsStatm {
    /// Total program size (pages)
    pub size: Option<u32>,
    /// Size of memory portions (pages)
    pub resident: Option<u32>,
    /// Number of pages that are shared
    pub share: Option<u32>,
    /// Number of pages that are ‘code’
    pub trs: Option<u32>,
    /// Number of pages of data/stack
    pub drs: Option<u32>,
    /// Number of pages of library
    pub lrs: Option<u32>,
    /// Number of dirty pages
    pub dt: Option<u32>,
    /// Pagesize
    pub pagesize: Option<u32>
}

impl ProcsStatm {
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
     * `pagesize`: Pagesize
     * 
     * Returns a new `Statm`.
     */
    #[allow(clippy::too_many_arguments)]
    #[must_use] pub fn new(
        size: &Option<u32>,
        resident: &Option<u32>,
        share: &Option<u32>,
        trs: &Option<u32>,
        drs: &Option<u32>,
        lrs: &Option<u32>,
        dt: &Option<u32>,
        pagesize: &Option<u32>
    ) -> ProcsStatm {

        ProcsStatm {
            size: *size,
            resident: *resident,
            share: *share,
            trs: *trs,
            drs: *drs,
            lrs: *lrs,
            dt: *dt,
            pagesize: *pagesize
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
    pub fn get_statm(pid: u32) -> Result<ProcsStatm, CommonLibError> {

        let pagesize = ProcsStatm::get_pagesize()?;

        let statm_file = File::open("/proc/".to_string() + pid.to_string().as_str() + "/statm").map_err(|err| {
            CommonLibError::new(format!("Error reading statm file: {err:?}").as_str())
        })?;
        ProcsStatm::read_statm(statm_file, pagesize)        
    }

    /**
     * Get the pagesize.
     * 
     * Returns the pagesize or an error.
     * 
     * # Errors
     * - If there is an error getting the pagesize.
     */
    fn get_pagesize() -> Result<Option<u32>, CommonLibError> {
        let pagesize = std::process::Command::new("getconf")
        .arg("PAGESIZE")
        .output()
        .map_err(|err| {
            CommonLibError::new(format!("Error getting pagesize: {err:?}").as_str())
        })?.stdout;    
        let pagesize = String::from_utf8(pagesize)
            .map_err(|err| {
                CommonLibError::new(format!("Error parsing pagesize: {err:?}").as_str())
            })?.trim().parse::<u32>().ok();
        Ok(pagesize)
    }
    
    /**
     * Read the statm file. 
     * If any data field is not parseable, it will be set to None.
     * 
     * `file`: The file to read.
     * `pagesize`: The pagesize.
     * 
     * Returns the statm data or an error.
     * 
     * # Errors
     * - If there is an error reading the statm file.
     */
    fn read_statm(file: File, pagesize: Option<u32>) -> Result<ProcsStatm, CommonLibError> {
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        let _ =  reader.read_line(&mut buffer).map_err(|err| {
            CommonLibError::new(format!("Error reading statm: {err:?}").as_str())
        })?;
        Ok(ProcsStatm::handle_statm_file(buffer.as_str(), pagesize))        
    }

    /**
     * Handle the statm file.
     * 
     * `buffer`: The buffer to parse.
     * 
     * Returns the statm data or an error.
     */
    fn handle_statm_file(buffer: &str, pagesize: Option<u32>) -> ProcsStatm {
        let cols = buffer.split_whitespace().collect::<Vec<&str>>();
        let size = cols[0].parse::<u32>().ok();
        let resident = cols[1].parse::<u32>().ok();
        let share = cols[2].parse::<u32>().ok();
        let trs = cols[3].parse::<u32>().ok();
        let lrs: Option<u32> = cols[4].parse::<u32>().ok();
        let drs = cols[5].parse::<u32>().ok();
        let dt = cols[6].parse::<u32>().ok();
        ProcsStatm::new(&size, &resident, &share, &trs, &drs, &lrs, &dt, &pagesize)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_statm_from_buffer1() {
        let buffer = "5805 3442 2354 11 0 1082 0";
        let statm = ProcsStatm::handle_statm_file(buffer, Some(4096));
        assert_eq!(statm.size, Some(5805));
        assert_eq!(statm.resident, Some(3442));
        assert_eq!(statm.share, Some(2354));
        assert_eq!(statm.trs, Some(11));
        assert_eq!(statm.drs, Some(1082));
        assert_eq!(statm.lrs, Some(0));
        assert_eq!(statm.dt, Some(0));
    }
    
    #[test]
    fn test_handle_statm_buffer2() {
        let buffer = "494524 21506 2214 1471 0 59164 0";
        let statm = ProcsStatm::handle_statm_file(buffer, Some(4096));
        assert_eq!(statm.size, Some(494524));
        assert_eq!(statm.resident, Some(21506));
        assert_eq!(statm.share, Some(2214));
        assert_eq!(statm.trs, Some(1471));
        assert_eq!(statm.drs, Some(59164));
        assert_eq!(statm.lrs, Some(0));
        assert_eq!(statm.dt, Some(0));
    }    

    #[test]
    fn test_handle_statm_pid_1() {
        let statm = ProcsStatm::get_statm(1);
        assert!(statm.is_ok());
    }  

    #[test]
    fn test_handle_statm_pid_0() {
        let statm = ProcsStatm::get_statm(0);
        assert!(statm.is_err());
    }    
}