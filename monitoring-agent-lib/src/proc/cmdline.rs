use std::{fs::{self, DirEntry, File, ReadDir}, io::Read};
use log::debug;
use regex::Regex;

use crate::common::CommonLibError;

/**
 * The `CmdLine` struct represents the command line of a process.
 */
#[derive(Debug, Clone)]
pub struct ProcsCmdLine {
    /// The full path of the command.
    pub fullpath: String,
    /// The process id.
    pub pid: u32,
}

impl ProcsCmdLine {

    /**
     * Create a new `CmdLine`.
     * 
     * `fullpath`: The full path of the command.
     * `application`: The application name.
     * `pid`: The process id.
     * 
     * Returns a new `CmdLine`.
     */
    #[must_use] pub fn new(fullpath: &str, pid: u32) -> ProcsCmdLine {
        ProcsCmdLine {
            fullpath: fullpath.to_string(),
            pid
        }
    }

    /**
     * Get all processes.
     * 
     * Returns a list of processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading the process directory.
     * - If there is an error reading the cmdline file.
     */
    #[tracing::instrument(level = "debug")]
    pub fn get_all_processes() -> Result<Vec<ProcsCmdLine>, CommonLibError> {
        let paths = fs::read_dir("/proc");
        match paths {
            Ok(paths) => {
                ProcsCmdLine::read_processes(paths)
            },
            Err(err) => {
                Err(CommonLibError::new(&format!("Error reading /proc, err: {err:?}")))
            }
        }
    }

    /**
     * Read all processes.
     * 
     * `read_dir`: The directory to read.
     * 
     * Returns a list of processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading the process directory.
     * - If there is an error reading the cmdline file.
     */
    fn read_processes(read_dir: ReadDir) -> Result<Vec<ProcsCmdLine>, CommonLibError> {        
        let starts_with_number_regexp = Regex::new(r"^[0-9]+$").map_err(|err|CommonLibError::new(format!("Error creating regexp: err: {err:?}").as_str()))?;
        let mut processes: Vec<ProcsCmdLine> = Vec::new();
        for path in read_dir {            
            match &path {
                Ok(path) => {
                    if ProcsCmdLine::is_process_directory(&starts_with_number_regexp, path) {                          
                        let path_buffer = path.path();
                        let use_dir = path_buffer.to_str().ok_or(CommonLibError::new("Error reading path"))?;
                        let pid: u32 = path.file_name().to_str().unwrap_or("").parse::<u32>().map_err(|err| CommonLibError::new(&format!("Error reading pid: {err:?}")))?;
                        let process = ProcsCmdLine::get_cmdline_with_dir(use_dir, pid)?;
                        processes.push(process);
                    }           
                },
                Err(err) => {
                    return Err(CommonLibError::new(&format!("Error reading /proc, err: {err:?}")));
                }            
            }
        }
        Ok(processes)
    }    

    /**
     * Get the cmdline of a process.
     * 
     * `proc_dir`: The process directory.
     * `pid`: The process id.
     * 
     * Returns the cmdline or an error.
     * 
     * # Errors
     * - If there is an error reading the cmdline file.
     */
    fn get_cmdline_with_dir(proc_dir: &str, pid: u32) -> Result<ProcsCmdLine, CommonLibError> {
        let path = proc_dir.to_string() + "/cmdline";
        let file = File::open(path);
        match file {
            Ok(file) => {
                ProcsCmdLine::get_cmdline_from_file(file, pid)
            },
            Err(err) => {
                Err(CommonLibError::new(&format!("Error reading status, err: {err:?}")))
            }
        
        }
    }  

    /**
     * Get the cmdline of a process.
     * 
     * `pid`: The process id.
     * 
     * Returns the cmdline or an error.
     * 
     * # Errors
     * - If there is an error reading the cmdline file.
     */
    pub fn read_cmdline(pid: u32) -> Result<ProcsCmdLine, CommonLibError> {
        let path = "/proc/".to_string() + pid.to_string().as_str() + "/cmdline";
        let file = File::open(path);
        match file {
            Ok(file) => {
                ProcsCmdLine::get_cmdline_from_file(file, pid)
            },
            Err(err) => {
                Err(CommonLibError::new(&format!("Error reading status, err: {err:?}")))
            }
        
        }
    }         

    /**
     * Get the cmdline from a file.
     * 
     * `file`: The file to read.
     * `pid`: The process id.
     * 
     * Returns the cmdline or an error.
     * 
     * # Errors
     * - If there is an error reading the cmdline file.
     */
    fn get_cmdline_from_file(mut file: File, pid: u32) -> Result<ProcsCmdLine, CommonLibError> {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(|err| {
            CommonLibError::new(&format!("Error reading cmdline, err: {err:?}"))
        })?;
        Ok(ProcsCmdLine::new(
            buffer.as_str(),
            pid
        ))
    }    

    /**
     * Check if the directory is a process directory. These directories start with a number.
     * 
     * `starts_with_number_regexp`: The regexp to check if the directory starts with a number.
     * `dir`: The directory to check.
     * 
     * Returns true if the directory is a process directory. Otherwise false.
     */
    fn is_process_directory(starts_with_number_regexp: &Regex, dir: &DirEntry) -> bool {
        let file_name = dir.file_name();
        let file_name = file_name.to_str();
        if !dir.path().is_dir() {
            return false;
        }
        match file_name {
            Some(file_name) => {
                starts_with_number_regexp.is_match_at(file_name, 0)
            },
            None => {
                false
            }
        }
    }    

    /**
     * Get all processes by application name.
     * 
     * `application`: The application name.
     * 
     * Returns a list of processes or an error.
     */
    #[tracing::instrument(level = "debug")]
     pub fn read_by_application(application: &str) -> Result<Vec<ProcsCmdLine>, CommonLibError> {
        fs::read_dir("/proc").map_err(|err| {
            CommonLibError::new(&format!("Error reading /proc, err: {err:?}"))
        }).and_then(|read_dir| {
            ProcsCmdLine::get_by_application(read_dir, application)
        })
    }    
    /**
     * Get all processes by application name.
     * 
     * `application`: The application name.
     * 
     * Returns a list of processes or an error.
     */
    fn get_by_application(file_path: ReadDir, application: &str) -> Result<Vec<ProcsCmdLine>, CommonLibError> {
        let processes = ProcsCmdLine::read_processes(file_path)?;
        let processes = processes.into_iter().filter(|process| {
            debug!("Checking process: {process:?}");
            process.fullpath.contains(application)
        }).collect();
        Ok(processes)
    }

}

#[cfg(test)]
mod test {
    use std::fs;

    use super::ProcsCmdLine;


    #[test]
    fn test_all() {
        let processes = super::ProcsCmdLine::get_all_processes();
        assert!(processes.is_ok());
    }

    #[test]
    fn test_testdir() {
        let processes = ProcsCmdLine::read_processes( fs::read_dir("resources/test/processes").unwrap());
        assert!(processes.is_ok());
        let processes = processes.unwrap();
        let cmdline = processes.first().unwrap();
        assert_eq!(cmdline.fullpath, "/usr/sbin/apache2\0-k\0start\0");
        assert_eq!(cmdline.pid, 2914);
    }

    #[test]
    fn test_testdir_by_application() {
        let processes = ProcsCmdLine::get_by_application( fs::read_dir("resources/test/processes").unwrap(), "apache2\0-k\0start\0");
        assert!(processes.is_ok());
        let processes = processes.unwrap();
        let cmdline = processes.first().unwrap();
        assert_eq!(cmdline.fullpath, "/usr/sbin/apache2\0-k\0start\0");
        assert_eq!(cmdline.pid, 2914);
    }    

    #[test]
    fn test_testdir_by_application_not_found() {
        let processes = ProcsCmdLine::get_by_application( fs::read_dir("resources/test/processes").unwrap(), "chrome");
        assert!(processes.is_ok());
        let processes = processes.unwrap();
        assert_eq!(processes.len(), 0);
    }       

    #[test]
    fn test_testdir_by_application_not_found2() {
        let processes = ProcsCmdLine::read_by_application("xyz");
        assert!(processes.is_ok());
        let processes = processes.unwrap();
        assert_eq!(processes.len(), 0);
    } 

    #[test]
    fn test_read_systemd() {
        let processes = ProcsCmdLine::read_cmdline(1);
        assert!(processes.is_ok());
    }     

}
