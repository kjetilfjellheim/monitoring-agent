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
    /// The group names the process belongs to.
    pub groups: Option<Vec<String>>,
    /// user of the process. Real, Effective, Saved Set, File System.
    pub uid: Option<Vec<String>>,
    /// group of the process. Real, Effective, Saved Set, File System.
    pub gid: Option<Vec<String>>,
    
}

impl ProcsProcess {
    
    /**
    * Create a new `ProcsProcess`.
    *
    * ```
    * use monitoring_agent_lib::proc::process::ProcsProcess;
    * ProcsProcess::new(None, None, None, None, None, None, None, None, None);
    * ```
    * ```
    * use monitoring_agent_lib::proc::process::ProcsProcess;
    * use monitoring_agent_lib::proc::process::ProcessState;
    * ProcsProcess::new(Some(2914), Some(2656), Some("code".to_string()), Some("0002".to_string()), Some(ProcessState::InterruptableSleep), Some(1), Some(vec!["4".to_string(), "24".to_string(), "27".to_string(), "30".to_string(), "46".to_string(), "100".to_string(), "119".to_string(), "129".to_string(), "1000".to_string()]), None, None);
    * ```
    * 
    * `pid`: The process id.
    * `parent_pid`: The parent process id.
    * `name`: The name of the process.
    * `umask`: The umask of the process.
    * `state`: The state of the process.
    * `threads`: The number of threads in the process.
    * `groups`: The groups the process belongs to.
    * `uid`: The uid of the process. Real, Effective, Saved Set, File System.
    * `gid`: The gid of the process. Real, Effective, Saved Set, File System.
    * 
    * Returns a new `ProcsProcess`.
    */
    #[allow(clippy::too_many_arguments)]
    #[must_use] pub fn new(
        pid: Option<u32>,
        parent_pid: Option<u32>,
        name: Option<String>,
        umask: Option<String>,
        state: Option<ProcessState>,
        threads: Option<u32>,
        groups: Option<Vec<String>>,
        uid: Option<Vec<String>>,
        gid: Option<Vec<String>>,
    ) -> ProcsProcess {
        ProcsProcess {
            pid,
            parent_pid,
            name,
            umask,
            state,
            threads,
            groups,
            uid,
            gid
        }
    }

    /**
     * Get all processes.
     * 
     * ```
     * use monitoring_agent_lib::proc::process::ProcsProcess;
     * ProcsProcess::get_all_processes();
     * ```
     * 
     * Returns the processes or an error.
     * 
     * # Errors
     * - If there is an error reading the directory.
     * - If there is an error reading a process.
     * - If there is an error reading a line from the process file.
     *
     */
    #[tracing::instrument(level = "debug")]
    pub fn get_all_processes() -> Result<Vec<ProcsProcess>, CommonLibError> {
        let paths = fs::read_dir("/proc");

        let group_names: HashMap<u32, String> = crate::proc::group::Group::get_groups_map()?;
        let user_names: HashMap<u32, String> = crate::proc::user::User::get_users_map()?;

        match paths {
            Ok(paths) => {
                ProcsProcess::read_processes(paths, &group_names, &user_names)
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
    fn read_processes(read_dir: ReadDir, group_names: &HashMap<u32, String>, user_names: &HashMap<u32, String>) -> Result<Vec<ProcsProcess>, CommonLibError> {        
        let starts_with_number_regexp = Regex::new(r"^[0-9]+$").map_err(|err|CommonLibError::new(format!("Error creating regexp: err: {err:?}").as_str()))?;
        let mut processes: Vec<ProcsProcess> = Vec::new();
        for path in read_dir {            
            match &path {
                Ok(path) => {
                    if ProcsProcess::is_process_directory(&starts_with_number_regexp, path) {                          
                        let path_buffer = path.path();
                        let use_dir = path_buffer.to_str().ok_or(CommonLibError::new("Error reading path"))?;
                        let process = ProcsProcess::get_process_status_with_dir(use_dir, group_names, user_names)?;
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
     * ```
     * use monitoring_agent_lib::proc::process::ProcsProcess;
     * ProcsProcess::get_process(1);
     * ```
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
    #[tracing::instrument(level = "debug")]
    pub fn get_process(pid: u32) -> Result<ProcsProcess, CommonLibError> {
        let group_names: HashMap<u32, String> = crate::proc::group::Group::get_groups_map()?;
        let user_names: HashMap<u32, String> = crate::proc::user::User::get_users_map()?;        

        let path = "/proc".to_string() + "/" + &pid.to_string();
        ProcsProcess::get_process_status_with_dir(&path, &group_names, &user_names)
    }

    /**
     * Get child processes.
     * 
     * ```
     * use monitoring_agent_lib::proc::process::ProcsProcess;
     * ProcsProcess::get_process_threads(1);
     * ```
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
    #[tracing::instrument(level = "debug")]
    pub fn get_process_threads(pid: u32) -> Result<Vec<ProcsProcess>, CommonLibError> {
        let group_names: HashMap<u32, String> = crate::proc::group::Group::get_groups_map()?;
        let user_names: HashMap<u32, String> = crate::proc::user::User::get_users_map()?;        
        ProcsProcess::read_process_threads(pid, "/proc", &group_names, &user_names)
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
    fn read_process_threads(pid: u32, path: &str, group_names: &HashMap<u32, String>, user_names: &HashMap<u32, String>) -> Result<Vec<ProcsProcess>, CommonLibError> {
        let task_path = path.to_string() + "/" + &pid.to_string() + "/task";
        let task_paths = fs::read_dir(task_path);
        match task_paths {
            Ok(paths) => {
                let mut processes: Vec<ProcsProcess> = Vec::new();
                ProcsProcess::loop_child_paths(paths, &mut processes, group_names, user_names)?;
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
    fn loop_child_paths(paths: ReadDir, processes: &mut Vec<ProcsProcess>, group_names: &HashMap<u32, String>, user_names: &HashMap<u32, String>) -> Result<(), CommonLibError> {
        for path in paths {
            let starts_with_number_regexp = Regex::new(r"^[0-9]+$").map_err(|err|CommonLibError::new(format!("Error creating regexp: err: {err:?}").as_str()))?;
            match &path {
                Ok(path) => {
                    ProcsProcess::add_child_process(&starts_with_number_regexp, path, processes, group_names, user_names)?;
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
    fn add_child_process(starts_with_number_regexp: &Regex, path: &DirEntry, processes: &mut Vec<ProcsProcess>, group_names: &HashMap<u32, String>, user_names: &HashMap<u32, String>) -> Result<(), CommonLibError> {
        if ProcsProcess::is_process_directory(starts_with_number_regexp, path) {
            let path_buffer = path.path();
            let use_dir = path_buffer.to_str().ok_or(CommonLibError::new("Error reading path"))?;
            let process = ProcsProcess::get_process_status_with_dir(use_dir, group_names, user_names)?;
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
    fn get_process_status_with_dir(proc_dir: &str, group_names: &HashMap<u32, String>, user_names: &HashMap<u32, String>) -> Result<ProcsProcess, CommonLibError> {
        let path = proc_dir.to_string() + "/status";
        let file = File::open(path);
        match file {
            Ok(file) => {
                ProcsProcess::get_process_status_from_file(file, group_names, user_names)
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
    fn get_process_status_from_file(file: File, group_names: &HashMap<u32, String>, user_names: &HashMap<u32, String>) -> Result<ProcsProcess, CommonLibError> {
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
            ProcsProcess::get_names_by_map(parts.get("Groups"), group_names)?,
            ProcsProcess::get_names_by_map(parts.get("Uid"), user_names)?,
            ProcsProcess::get_names_by_map(parts.get("Gid"), group_names)?
        ))
    }

    /**
     * Get the groups.
     * 
     * `groups`: The groups to get.
     * `groupNames`: The group names.
     * 
     * Returns the groups.
     * 
     * # Errors
     * - If there is an error parsing the group id.
     * - If there is an error getting the group.
     * 
     */
    fn get_names_by_map(data: Option<&String>, names: &HashMap<u32, String>) -> Result<Option<Vec<String>>, CommonLibError> {
        match data {
            Some(data) => {
                let mut groups: Vec<String> = Vec::new();
                for str in data.split_whitespace() {
                    let id = u32::from_str(str).map_err(|err| CommonLibError::new(format!("Error parsing name id: {err:?}").as_str()))?;
                    let name = {
                        let default_name = "Unknown".to_string();
                        names.get(&id).unwrap_or(&default_name).to_string()
                    };
                    groups.push(name.clone());
                }                            
                Ok(Some(groups))
            },
            None => {
                Ok(None)
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

    use crate::proc::{Group, User};

    use super::*;

    #[test]
    fn test_read_all_processes() {
        let processes = ProcsProcess::get_all_processes();
        println!("{:?}", processes);
        assert!(processes.is_ok());
    }   

    #[test]
    fn test_read_2914() {
        let groups = Group::get_groups_map().unwrap();
        let users = User::get_users_map().unwrap();
        
        let processes = ProcsProcess::read_processes( fs::read_dir("resources/test/processes").unwrap(), &groups, &users);
        println!("{:?}", processes);
        assert!(&processes.is_ok());
        let processes = processes.unwrap().clone();
        assert_eq!(&processes.get(0).unwrap().pid, &Some(2914));
        assert_eq!(&processes.get(0).unwrap().parent_pid, &Some(2656));
        assert_eq!(&processes.get(0).unwrap().name, &Some("code".to_string()));
        assert_eq!(&processes.get(0).unwrap().umask, &Some("0002".to_string()));
        assert_eq!(&processes.get(0).unwrap().state, &Some(ProcessState::InterruptableSleep));
        assert_eq!(&processes.get(0).unwrap().threads, &Some(1));
    }   

    #[test]
    fn test_read_single_2914() {
        let groups = Group::get_groups_map().unwrap();
        let users = User::get_users_map().unwrap();

        let process = ProcsProcess::get_process_status_with_dir("resources/test/processes/2914", &groups, &users).unwrap();
        assert_eq!(&process.pid, &Some(2914));
        assert_eq!(&process.parent_pid, &Some(2656));
        assert_eq!(&process.name, &Some("code".to_string()));
        assert_eq!(&process.umask, &Some("0002".to_string()));
        assert_eq!(&process.state, &Some(ProcessState::InterruptableSleep));
        assert_eq!(&process.threads, &Some(1));
    }    

    #[test]
    fn test_read_children() {
        let groups = Group::get_groups_map().unwrap();
        let users = User::get_users_map().unwrap();

        let processes = ProcsProcess::read_process_threads(2914, "resources/test/processes", &groups, &users);
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

    #[test]
    fn test_get_process_threads() {
        let processes = ProcsProcess::get_process_threads(1);
        assert!(processes.is_ok());
    }

    #[test]
    fn test_get_process() {
        let process = ProcsProcess::get_process(1);
        assert!(process.is_ok());
    }

    #[test]
    fn test_get_state() {
        let state = ProcsProcess::get_state(Some(&"R".to_string()));
        assert_eq!(state, Some(ProcessState::Running));
        let state = ProcsProcess::get_state(Some(&"D".to_string()));
        assert_eq!(state, Some(ProcessState::DiskSleep));
        let state = ProcsProcess::get_state(Some(&"S".to_string()));
        assert_eq!(state, Some(ProcessState::InterruptableSleep));
        let state = ProcsProcess::get_state(Some(&"t".to_string()));
        assert_eq!(state, Some(ProcessState::TracingStop));
        let state = ProcsProcess::get_state(Some(&"Z".to_string()));
        assert_eq!(state, Some(ProcessState::Zombie));
        let state = ProcsProcess::get_state(Some(&"I".to_string()));
        assert_eq!(state, Some(ProcessState::Idle));
        let state = ProcsProcess::get_state(Some(&"X".to_string()));
        assert_eq!(state, Some(ProcessState::Dead));
        let state = ProcsProcess::get_state(Some(&"U".to_string()));
        assert_eq!(state, Some(ProcessState::Unknown));
        let state = ProcsProcess::get_state(Some(&"T".to_string()));
        assert_eq!(state, Some(ProcessState::Stopped));           
        let state = ProcsProcess::get_state(None);
        assert_eq!(state, None);                   
    }

    #[test]
    fn test_get_process_status_with_dir_error() {
        let groups = Group::get_groups_map().unwrap();
        let users = User::get_users_map().unwrap();

        let process = ProcsProcess::get_process_status_with_dir("resources/test/677676", &groups, &users);
        assert!(process.is_err());
    }

}
