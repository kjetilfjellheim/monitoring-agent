use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use log::error;
use monitoring_agent_lib::proc::{ProcStat, ProcsCpuinfo, ProcsLoadavg, ProcsMeminfo, ProcsProcess, ProcsStatm};

use crate::common::{ApplicationError, MonitorStatus};

/**
 * Monitoring Service.
 *
 * This struct represents the monitoring service.
 *
 * `scheduler`: The job scheduler.
 * `tcp_monitors`: The TCP monitors.
 * `http_monitors`: The HTTP monitors.
 * `command_monitors`: The command monitors.
 *  
 */
#[derive(Clone)]
pub struct MonitoringService {
    /// The status of the monitors.
    status: Arc<Mutex<HashMap<String, MonitorStatus>>>
}

impl MonitoringService {
    /**
     * Create a new monitoring service.
     *
     * result: The result of creating the monitoring service.
     */
    pub fn new() -> MonitoringService {
        MonitoringService {
            status: Arc::new(Mutex::new(HashMap::new())),            
        }
    }

    /**
     * Get the current memory information.
     *
     * result: The result of getting the current memory information.
     */
    #[allow(clippy::unused_self)]
    pub fn get_current_meminfo(&self) -> Result<ProcsMeminfo, ApplicationError> {
        let meminfo = ProcsMeminfo::get_meminfo();
        match meminfo {
            Ok(meminfo) => Ok(meminfo),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting meminfo"))                
            }
        }
    }

    /**
     * Get the current cpu information.
     *
     * result: The result of getting the current cpu information.
     */
    #[allow(clippy::unused_self)]
    pub fn get_current_cpuinfo(&self) -> Result<Vec<ProcsCpuinfo>, ApplicationError> {
        let cpuinfo = ProcsCpuinfo::get_cpuinfo();
        match cpuinfo {
            Ok(cpuinfo) => Ok(cpuinfo),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting cpuinfo"))                
            }
        }
    }    

    /**
     * Get the current load average.
     *
     * result: The result of getting the load average information.
     */
    #[allow(clippy::unused_self)]
    pub fn get_current_loadavg(&self) -> Result<ProcsLoadavg, ApplicationError> {
        let loadavg = ProcsLoadavg::get_loadavg();
        match loadavg {
            Ok(loadavg) => Ok(loadavg),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting loadavg"))                
            }
        }
    }

    /**
     * Get the current processes.
     * 
     * result: The result of getting the current processes.
     * 
     * # Errors
     * - If there is an error getting the processes.
     */
    #[allow(clippy::unused_self)]
    pub fn get_processes(&self) -> Result<Vec<ProcsProcess>, ApplicationError> {
        let processes = ProcsProcess::get_all_processes();
        match processes {
            Ok(processes) => Ok(processes),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting processes"))                
            }
        }
    }

    /**
     * Get the current process.
     * 
     * `pid`: The process id.
     * 
     * result: The result of getting the current process.
     * 
     * # Errors
     * - If there is an error getting the process.
     */
    #[allow(clippy::unused_self)]
    pub fn get_process(&self, pid: u32) -> Result<ProcsProcess, ApplicationError> {
        let process = ProcsProcess::get_process(pid);
        match process {
            Ok(process) => Ok(process),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting process"))                
            }
        }
    }

    /**
     * Get the process threads.
     * 
     * `pid`: The process id.
     * 
     * result: The result of getting the current process.
     * 
     * # Errors
     * - If there is an error getting the process.
     */
    #[allow(clippy::unused_self)]
    pub fn get_process_threads(&self, pid: u32) -> Result<Vec<ProcsProcess>, ApplicationError> {
        let threads = ProcsProcess::get_process_threads(pid);
        match threads {
            Ok(threads) => Ok(threads),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting threads"))                
            }
        }
    } 

    /**
     * Get the status.
     * 
     * result: The result of getting the status.
     */
    pub fn get_status(&self) -> Arc<Mutex<HashMap<String, MonitorStatus>>> {
        self.status.clone()
    }

    /**
     * Get all monitor statuses.
     * 
     * result: The result of getting all monitor statuses.
     */
    pub fn get_all_monitorstatuses(&self) -> Vec<MonitorStatus> {
        let status_lock = self.status.lock();
        match status_lock {
            Ok(lock) => lock.values().cloned().collect(),
            Err(err) => {
                error!("Error getting monitor statuses: {:?}", err);
                Vec::new()
            }
        }
    }

    /**
     * Get the current statm.
     * 
     * `pid`: The process id.
     * 
     * result: The result of getting the current statm.
     * 
     * # Errors
     * - If there is an error getting the statm.
     */
    #[allow(clippy::unused_self)]
    pub fn get_current_statm(&self, pid: u32) -> Result<ProcsStatm, ApplicationError> {
        let statm = ProcsStatm::get_statm(pid);
        match statm {
            Ok(statm) => Ok(statm),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting statm"))                
            }
        }
    }

    /**
     * Get the current stat.
     * 
     * result: The result of getting the current stat.
     * 
     * # Errors
     * - If there is an error getting the stat.
     */
    #[allow(clippy::unused_self)]
    pub fn get_stat(&self) -> Result<ProcStat, ApplicationError> {
        let stat = ProcStat::get_stat();
        match stat {
            Ok(stat) => Ok(stat),
            Err(err) => {
                error!("Error: {}", err.message);
                Err(ApplicationError::new("Error getting stat"))                
            }
        }
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_all_monitorstatuses() {
        let monitoring_service = MonitoringService::new();
        let monitor_statuses = monitoring_service.get_all_monitorstatuses();
        assert_eq!(monitor_statuses.len(), 0);
    }

    #[test]
    fn test_get_status() {
        let monitoring_service = MonitoringService::new();
        let status = monitoring_service.get_status();
        assert_eq!(status.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_get_current_meminfo() {
        let monitoring_service = MonitoringService::new();
        let meminfo = monitoring_service.get_current_meminfo();
        assert!(meminfo.is_ok());
    }

    #[test]
    fn test_get_current_cpuinfo() {
        let monitoring_service = MonitoringService::new();
        let cpuinfo = monitoring_service.get_current_cpuinfo();
        assert!(cpuinfo.is_ok());
    } 

    #[test]
    fn test_get_current_loadavg() {
        let monitoring_service = MonitoringService::new();
        let cpuinfo = monitoring_service.get_current_loadavg();
        assert!(cpuinfo.is_ok());
    } 

    #[test]
    fn test_get_processes() {
        let monitoring_service = MonitoringService::new();
        let cpuinfo = monitoring_service.get_processes();
        assert!(cpuinfo.is_ok());
    }     

    #[test]
    fn test_get_process() {
        let monitoring_service = MonitoringService::new();
        let cpuinfo = monitoring_service.get_process(1);
        assert!(cpuinfo.is_ok());
    }    

    #[test]
    fn test_get_threads() {
        let monitoring_service = MonitoringService::new();
        let cpuinfo = monitoring_service.get_process_threads(1);
        assert!(cpuinfo.is_ok());
    }    

     

}