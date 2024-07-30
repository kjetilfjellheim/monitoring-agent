use std::{collections::HashMap, fs::{self, DirEntry, File, ReadDir}, io::{BufRead, BufReader}};
use std::str::FromStr;

use log::error;
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::common::CommonLibError;

/**
 * Process information from /proc/*/status */
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcsProcess {
    /// The process id.
    pub pid: Option<u32>,
    /// The parent process id.
    pub parent_pid: Option<u32>,
    /// The name of the process.
    pub name: Option<String>,
    /// The umask of the process.
    pub umask: Option<String>,
    /// The state of the process.
    pub state: Option<ProcessState>,
    /// The number of threads in the process.
    pub threads: Option<u32>,
    /// The groups the process belongs to.
    pub groups: Option<Vec<String>>,
}

impl ProcsProcess {
    
    /**
    * Create a new `ProcsProcess`.
    *
    * `pid`: The process id.
    * `parent_pid`: The parent process id.
    * `name`: The name of the process.
    * `umask`: The umask of the process.
    * `state`: The state of the process.
    * `threads`: The number of threads in the process.
    * `groups`: The groups the process belongs to.
    * 
    * Returns a new `ProcsProcess`.
    */
    #[must_use] pub fn new(
        pid: Option<u32>,
        parent_pid: Option<u32>,
        name: Option<String>,
        umask: Option<String>,
        state: Option<ProcessState>,
        threads: Option<u32>,
        groups: Option<Vec<String>>,
    ) -> ProcsProcess {
        ProcsProcess {
            pid,
            parent_pid,
            name,
            umask,
            state,
            threads,
            groups,
        }
    }

    /**
     * Get all processes.
     * 
     * Returns the processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading a process.
     * - If there is an error reading a line from the process file.
     *
     */
    pub fn get_all_processes() -> Result<Vec<ProcsProcess>, CommonLibError> {
        let paths = fs::read_dir("/proc");
        match paths {
            Ok(paths) => {
                ProcsProcess::read_processes(paths)
            },
            Err(err) => {
                error!("Error reading /proc: {err:?}");
                Err(CommonLibError::new(&format!("Error reading /proc, err: {err:?}")))
            }
        }
    }

    /**
     * Read all processes.
     * 
     * `read_dir`: The directory to read.
     * 
     * Returns the processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading a process.
     * - If there is an error reading a line from the process file.
     *
     */
    fn read_processes(read_dir: ReadDir) -> Result<Vec<ProcsProcess>, CommonLibError> {        
        let starts_with_number_regexp = Regex::new(r"^[0-9]+$").map_err(|err|CommonLibError::new(format!("Error creating regexp: err: {err:?}").as_str()))?;
        let mut processes: Vec<ProcsProcess> = Vec::new();
        for path in read_dir {            
            match &path {
                Ok(path) => {
                    if ProcsProcess::is_process_directory(&starts_with_number_regexp, path) {                          
                        let path_buffer = path.path();
                        let use_dir = path_buffer.to_str().ok_or(CommonLibError::new("Error reading path"))?;
                        let process = ProcsProcess::get_process_status_with_dir(use_dir)?;
                        processes.push(process);
                    }           
                },
                Err(err) => {
                    error!("Error reading /proc: {err:?}");
                    return Err(CommonLibError::new(&format!("Error reading /proc, err: {err:?}")));
                }            
            }
        }
        Ok(processes)
    }

    /**
     * Get process status.
     * 
     * `dir`: The directory to get the process status from.
     * 
     * Returns the process status or an error.
     * 
     * # Errors
     * - If there is an error reading the status file.
     * - If there is an error reading a line from the process file.                  
     * 
     */
    pub fn get_process(pid: u32) -> Result<ProcsProcess, CommonLibError> {
        let path = "/proc".to_string() + "/" + &pid.to_string();
        ProcsProcess::get_process_status_with_dir(&path)
    }

    /**
     * Get child processes.
     * 
     * `pid`: The process id to get the child processes from.
     * 
     * Returns the child processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading a process.
     * - If there is an error reading a line from the process file.
     * 
     */
    pub fn get_process_threads(pid: u32) -> Result<Vec<ProcsProcess>, CommonLibError> {
        ProcsProcess::read_process_threads(pid, "/proc")
    }

    /**
     * Read child processes.
     * 
     * `pid`: The process id to get the child processes from.
     * `path`: The path to read the child processes from.
     * 
     * Returns the child processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading a process.
     * - If there is an error reading a line from the process file.
     * 
     */
    fn read_process_threads(pid: u32, path: &str) -> Result<Vec<ProcsProcess>, CommonLibError> {
        let task_path = path.to_string() + "/" + &pid.to_string() + "/task";
        let task_paths = fs::read_dir(task_path);
        match task_paths {
            Ok(paths) => {
                let mut processes: Vec<ProcsProcess> = Vec::new();
                ProcsProcess::loop_child_paths(paths, &mut processes)?;
                Ok(processes)
            },
            Err(err) => {
                error!("Error reading /proc: {err:?}");
                Err(CommonLibError::new(&format!("Error reading /proc, err: {err:?}")))
            }
        }        
    }

    /**
     * Loop child paths.
     * 
     * `paths`: The paths to loop.
     * `processes`: The processes to add the child processes to.
     * 
     * Returns the child processes or an error.
     * 
     * # Errors
     * - If there is an error reading the path.
     * 
     */
    fn loop_child_paths(paths: ReadDir, processes: &mut Vec<ProcsProcess>) -> Result<(), CommonLibError> {
        for path in paths {
            let starts_with_number_regexp = Regex::new(r"^[0-9]+$").map_err(|err|CommonLibError::new(format!("Error creating regexp: err: {err:?}").as_str()))?;
            match &path {
                Ok(path) => {
                    ProcsProcess::add_child_process(&starts_with_number_regexp, path, processes)?;
                },
                Err(err) => {
                    error!("Error reading /proc: {err:?}");
                    return Err(CommonLibError::new(&format!("Error reading /proc, err: {err:?}")));
                }
            }
        };
        Ok(())            
    }

    /**
     * Add a child process.
     * 
     * `starts_with_number_regexp`: The regexp to check if the directory starts with a number.
     * `path`: The directory to check.
     * `processes`: The processes to add the child process to.
     * 
     * Returns the child process or an error.
     * 
     * # Errors
     * - If there is an error reading the path.
     * 
     */
    fn add_child_process(starts_with_number_regexp: &Regex, path: &DirEntry, processes: &mut Vec<ProcsProcess>) -> Result<(), CommonLibError> {
        if ProcsProcess::is_process_directory(starts_with_number_regexp, path) {
            let path_buffer = path.path();
            let use_dir = path_buffer.to_str().ok_or(CommonLibError::new("Error reading path"))?;
            let process = ProcsProcess::get_process_status_with_dir(use_dir)?;
            processes.push(process);
        }
        Ok(())
    }

    /**
     * Get process status.
     * 
     * `proc_dir`: The directory to get the process status from.
     * 
     * Returns the process status or an error.
     * 
     * # Errors
     * - If there is an error reading the status file.
     * - If there is an error reading a line from the process file.                  
     * 
     */
    fn get_process_status_with_dir(proc_dir: &str) -> Result<ProcsProcess, CommonLibError> {
        let path = proc_dir.to_string() + "/status";
        let file = File::open(path);
        match file {
            Ok(file) => {
                ProcsProcess::get_process_status_from_file(file)
            },
            Err(err) => {
                error!("Error reading status: {err:?}");
                Err(CommonLibError::new(&format!("Error reading status, err: {err:?}")))
            }
        
        }

    }

    /**
     * Get process status from file.
     * 
     * `file`: The file to read.
     * 
     * Returns the process status or an error.
     * 
     * # Errors
     * - If there is an error reading a line from the process file.
     * 
     */
    fn get_process_status_from_file(file: File) -> Result<ProcsProcess, CommonLibError> {
        let reader = BufReader::new(file);
        let mut parts: HashMap<String, String> = HashMap::new();
        for line in reader.lines() {
            let line = ProcsProcess::get_line(line)?;
            let parts_data: Vec<&str> = line.split(':').collect();
            if parts_data.len() == 2 {
                parts.insert(parts_data[0].trim().to_string(), parts_data[1].trim().to_string());
            } 
        }
        Ok(ProcsProcess::new(
            parts.get("Pid").and_then(|f| u32::from_str(f).ok()),
            parts.get("PPid").and_then(|f| u32::from_str(f).ok()),
            parts.get("Name").cloned(),
            parts.get("Umask").cloned(),
            ProcsProcess::get_state(parts.get("State")),
            parts.get("Threads").and_then(|f| u32::from_str(f).ok()),
            ProcsProcess::get_groups(parts.get("Groups")),
        ))
    }

    /**
     * Get the groups.
     * 
     * `groups`: The groups to get.
     * 
     * Returns the groups.
     * 
     */
    fn get_groups(groups: Option<&String>) -> Option<Vec<String>> {
        match groups {
            Some(groups) => {                
                let groups: Vec<String> = groups.split_whitespace().map(std::string::ToString::to_string).collect();
                Some(groups)
            },
            None => {
                None
            }
        }
    }

    /**
     * Get the process state. 
     * 
     * `state`: The state to get.
     * 
     * Returns the process state.
     */
    fn get_state(state: Option<&String>) -> Option<ProcessState> {
        match state {
            Some(state) => {
                match state.as_str().chars().nth(0) {
                    Some('R') => Some(ProcessState::Running),
                    Some('D') => Some(ProcessState::DiskSleep),
                    Some('S') => Some(ProcessState::InterruptableSleep),
                    Some('T') => Some(ProcessState::Stopped),
                    Some('t') => Some(ProcessState::TracingStop),
                    Some('Z') => Some(ProcessState::Zombie),
                    Some('I') => Some(ProcessState::Idle),
                    Some('X') => Some(ProcessState::Dead),
                    None => None,
                    _ => Some(ProcessState::Unknown)
                }
            },
            None => {
                None
            }
        }   
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
     * Get a line from the process file.
     * 
     * `line`: The line to get.
     * 
     * Returns the line or an error.
     * 
     * # Errors
     * - If there is an error reading a line from the process file.
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

}

/**
 * Processstates.
 * 
 * This enum represents the different states a process can be in.
 * 
 * `Running`: The process is running.
 * `UninterruptibleSleep`: The process is in an uninterruptible sleep.
 * `InterruptableSleep`: The process is in an interruptable sleep.
 * `Stopped`: The process is stopped.
 * `Zombie`: The process is a zombie.
 * `Idle`: The process is idle.
 *
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessState {
    /// The process is running.
    Running,
    /// The process is in an uninterruptible sleep.
    InterruptableSleep,
    /// The process is stopped.
    Stopped,
    /// The process is a zombie.
    Zombie,
    /// The process is idle.
    Idle,
    /// The process is in disk sleep.
    DiskSleep,
    /// The process is dead.
    Dead,
    /// The process is in tracing stop.
    TracingStop,
    /// The process state is unknown. This probably occurs if the state is not found in the enum.
    Unknown
}

#[cfg(test)]
mod test {

    use std::vec;

    use super::*;

    #[test]
    fn test_read_all_processes() {
        let processes = ProcsProcess::get_all_processes();
        println!("{:?}", processes);
        assert!(processes.is_ok());
    }   

    #[test]
    fn test_read_2914() {
        let processes = ProcsProcess::read_processes( fs::read_dir("resources/test/processes").unwrap());
        println!("{:?}", processes);
        assert!(&processes.is_ok());
        let processes = processes.unwrap().clone();
        assert_eq!(&processes.get(0).unwrap().pid, &Some(2914));
        assert_eq!(&processes.get(0).unwrap().parent_pid, &Some(2656));
        assert_eq!(&processes.get(0).unwrap().name, &Some("code".to_string()));
        assert_eq!(&processes.get(0).unwrap().umask, &Some("0002".to_string()));
        assert_eq!(&processes.get(0).unwrap().state, &Some(ProcessState::InterruptableSleep));
        assert_eq!(&processes.get(0).unwrap().threads, &Some(1));
        assert_eq!(&processes.get(0).unwrap().groups, &Some(vec!["4".to_string(), "24".to_string(), "27".to_string(), "30".to_string(), "46".to_string(), "100".to_string(), "119".to_string(), "129".to_string(), "1000".to_string()]));
    }   

    #[test]
    fn test_read_single_2914() {
        let process = ProcsProcess::get_process_status_with_dir("resources/test/processes/2914").unwrap();
        assert_eq!(&process.pid, &Some(2914));
        assert_eq!(&process.parent_pid, &Some(2656));
        assert_eq!(&process.name, &Some("code".to_string()));
        assert_eq!(&process.umask, &Some("0002".to_string()));
        assert_eq!(&process.state, &Some(ProcessState::InterruptableSleep));
        assert_eq!(&process.threads, &Some(1));
        assert_eq!(&process.groups, &Some(vec!["4".to_string(), "24".to_string(), "27".to_string(), "30".to_string(), "46".to_string(), "100".to_string(), "119".to_string(), "129".to_string(), "1000".to_string()]));
    }    

    #[test]
    fn test_read_children() {
        let processes = ProcsProcess::read_process_threads(2914, "resources/test/processes");
        println!("{:?}", processes);
        assert!(&processes.is_ok());
        let processes = processes.unwrap().clone();
        assert_eq!(&processes.get(0).unwrap().pid, &Some(54112));
        assert_eq!(&processes.get(0).unwrap().parent_pid, &Some(2));
        assert_eq!(&processes.get(0).unwrap().umask, &Some("0000".to_string()));
        assert_eq!(&processes.get(0).unwrap().state, &Some(ProcessState::Idle));
        assert_eq!(&processes.get(0).unwrap().threads, &Some(1));
        assert_eq!(&processes.get(0).unwrap().groups, &Some(vec![]));

    }         

}
