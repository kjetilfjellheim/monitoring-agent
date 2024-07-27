use log::error;
use monitoring_agent_lib::proc::{ProcsCpuinfo, ProcsLoadavg, ProcsMeminfo, ProcsProcess};

use crate::common::ApplicationError;
use crate::common::configuration::MonitoringConfig;

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
    /// The monitoring configuration.
    monitoring_config: MonitoringConfig,
}

impl MonitoringService {
    /**
     * Create a new monitoring service.
     *
     * result: The result of creating the monitoring service.
     */
    pub fn new(monitoring_config: &MonitoringConfig) -> MonitoringService {
        MonitoringService {
            monitoring_config: monitoring_config.clone(),
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

}
