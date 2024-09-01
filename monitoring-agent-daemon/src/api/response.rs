use chrono::{DateTime, TimeZone, Utc };
use monitoring_agent_lib::proc::{process::ProcessState, ProcStat, ProcsCpuinfo, ProcsLoadavg, ProcsMeminfo, ProcsProcess, ProcsStatm};
use serde::{Deserialize, Serialize};

use crate::common::{historical::MeminfoElement, LoadavgElement, MonitorStatus, ProcessMemoryElement, Status};

/**
 * The `MeminfoResponse` struct represents the response of the meminfo endpoint.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct MeminfoResponse {
    /// The total memory.
    #[serde(skip_serializing_if = "Option::is_none", rename = "totalMem")]
    pub total_mem: Option<u64>,
    /// The free memory.
    #[serde(skip_serializing_if = "Option::is_none", rename = "freeMem")]
    pub free_mem: Option<u64>,
    /// The available memory.
    #[serde(skip_serializing_if = "Option::is_none", rename = "availableMem")]
    pub available_mem: Option<u64>,
    /// The total swap.
    #[serde(skip_serializing_if = "Option::is_none", rename = "swapTotal")]
    pub swap_total: Option<u64>,
    /// The free swap.
    #[serde(skip_serializing_if = "Option::is_none", rename = "swapFree")]
    pub swap_free: Option<u64>,
}

impl MeminfoResponse {
    /**
     * Create a new `MeminfoResponse`.
     *
     * `total_em`: The total memory.
     * `free_mem`: The free memory.
     * `available_mem`: The available memory.
     * `swap_total`: The total swap.
     * `swap_free`: The free swap.
     * 
     * Returns a new `MeminfoResponse`.
     */
    pub fn new(
        total_mem: Option<u64>,
        free_mem: Option<u64>,
        available_mem: Option<u64>,
        swap_total: Option<u64>,
        swap_free: Option<u64>,
    ) -> MeminfoResponse {
        MeminfoResponse {
            total_mem,
            free_mem,
            available_mem,
            swap_total,
            swap_free,
        }
    }

    /**
     * Create a new `MeminfoResponse` from a `Meminfo`.
     *
     * `procs_mem_info`: The `ProcsMeminfo` object.
     * 
     * Returns a new `MeminfoResponse`.
     */
    pub fn from_meminfo(procs_mem_info: &ProcsMeminfo) -> MeminfoResponse {
        MeminfoResponse::new(procs_mem_info.memtotal, procs_mem_info.memfree, procs_mem_info.memavailable, procs_mem_info.swaptotal, procs_mem_info.swapfree)
    }
}

/**
 * The `CpuinfoResponse` struct represents the response of the cpu endpoint.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct CpuinfoResponse {
    /// Onboard apicid of the cpu.
    #[serde(skip_serializing_if = "Option::is_none", rename = "apicid")]    
    pub apicid: Option<u8>,
    #[allow(clippy::doc_markdown)]
    /// Vendor id of the cpu e.g. AuthenticAMD.
    #[serde(skip_serializing_if = "Option::is_none", rename = "vendorId")]        
    pub vendor_id:  Option<String>,
    /// Authoritatively identifies the type of processor in the system.
    #[serde(skip_serializing_if = "Option::is_none", rename = "cpuFamily")]        
    pub cpu_family:  Option<String>,
    /// Model identifier of the cpu.
    #[serde(skip_serializing_if = "Option::is_none", rename = "model")]    
    pub model:  Option<String>,
    /// Displays the common name of the processor, including its project name.
    #[serde(skip_serializing_if = "Option::is_none", rename = "modelName")]    
    pub model_name:  Option<String>,
    /// Number of cores in the cpu.
    #[serde(skip_serializing_if = "Option::is_none", rename = "cpuCores")]        
    pub cpu_cores:  Option<u8>,
    /// Speed of the cpu in Mhz.
    #[serde(skip_serializing_if = "Option::is_none", rename = "cpuMhz")]    
    pub cpu_mhz:  Option<f32>,
}

impl CpuinfoResponse {
    /**
     * Create a new `CpuinfoResponse`.
     *
     * `apicid`: The apicid of the cpu.
     * `vendor_id`: The vendor id of the cpu.
     * `cpu_family`: The cpu family.
     * `model`: The model of the cpu.
     * `model_name`: The model name of the cpu.
     * `cpu_cores`: The number of cores in the cpu.
     * `cpu_mhz`: The speed of the cpu in mhz.
     * 
     * Returns a new `CpuinfoResponse`.
     */
    pub fn new(
        apicid: Option<u8>,
        vendor_id: Option<String>,
        cpu_family: Option<String>,
        model: Option<String>,
        model_name: Option<String>,
        cpu_cores: Option<u8>,
        cpu_mhz: Option<f32>,
    ) -> CpuinfoResponse {
        CpuinfoResponse {
            apicid,
            vendor_id,
            cpu_family,
            model,
            model_name,
            cpu_cores,
            cpu_mhz,
        }
    }

    /**
     * Create a new `CpuinfoResponse` from a `ProcsCpuinfo`.
     *
     * `procs_cpu_info`: The `ProcsCpuinfo` object.
     * 
     * Returns a new `CpuinfoResponse`.
     */
    pub fn from_cpuinfo(procs_cpu_info: &[ProcsCpuinfo]) -> Vec<CpuinfoResponse> {
        procs_cpu_info.iter().map(|cpu_info| CpuinfoResponse::new(cpu_info.apicid, cpu_info.vendor_id.clone(), cpu_info.cpu_family.clone(), cpu_info.model.clone(), cpu_info.model_name.clone(), cpu_info.cpu_cores, cpu_info.cpu_mhz)).collect()
    }    

}

/**
 * The `LoadavgResponse` struct represents the response of the loadavg endpoint.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct LoadavgResponse {
    /// Load average last 1 minute.
    #[serde(skip_serializing_if = "Option::is_none", rename = "loadAvg1Min")]    
    pub loadavg1min: Option<f32>,
    /// Load average last 5 minutes.
    #[serde(skip_serializing_if = "Option::is_none", rename = "loadAvg5Min")]        
    pub loadavg5min:  Option<f32>,
    /// Load average last 10 minutes.
    #[serde(skip_serializing_if = "Option::is_none", rename = "loadAvg15Min")]        
    pub loadavg15min:  Option<f32>,
    /// The number of currently running processes.
    pub current_running_processes: Option<u32>,
    /// The total number of processes.
    pub total_number_of_processes: Option<u32> 
}

impl LoadavgResponse {
    /**
     * Create a new `LoadavgResponse`.
     *
     * `loadavg1min`: The 1 minute load average.
     * `loadavg5min`: The 5 minute load average.
     * `loadavg15min`: The 10 minute load average.
     * 
     * Returns a new `CpuinfoResponse`.
     */
    #[allow(clippy::similar_names)]
    pub fn new(        
        loadavg1min: Option<f32>,
        loadavg5min: Option<f32>,
        loadavg15min: Option<f32>,
        current_running_processes: Option<u32>,
        total_number_of_processes: Option<u32>,
    ) -> LoadavgResponse {
        LoadavgResponse {
            loadavg1min,
            loadavg5min,
            loadavg15min,
            current_running_processes,
            total_number_of_processes,
        }
    }

    /**
     * Create a new `LoadavgResponse` from a `Loadavg`.
     *
     * `procs_loadavg`: The `Loadavg` object.
     * 
     * Returns a new `LoadavgResponse`.
     */
    pub fn from_loadavg(procs_loadavg: &ProcsLoadavg) -> LoadavgResponse {
        LoadavgResponse::new(
            procs_loadavg.loadavg1min, 
            procs_loadavg.loadavg5min, 
            procs_loadavg.loadavg15min, 
            procs_loadavg.current_running_processes, 
            procs_loadavg.total_number_of_processes)
    }    

}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ProcessResponse {
    /// The process id.
    #[serde(skip_serializing_if = "Option::is_none", rename = "pid")]   
    pub pid: Option<u32>,
    /// The parent process id.
    #[serde(skip_serializing_if = "Option::is_none", rename = "parentPid")]  
    pub parent_pid: Option<u32>,
    /// Command run by this process. Strings longer than 16 characters (including the terminating null byte) are silently truncated.
    #[serde(skip_serializing_if = "Option::is_none", rename = "name")]      
    pub name: Option<String>,
    /// Process umask, expressed in octal with a leading zero.
    #[serde(skip_serializing_if = "Option::is_none", rename = "umask")]          
    pub umask: Option<String>,
    /// The state of the process.
    #[serde(skip_serializing_if = "Option::is_none", rename = "processState")]              
    pub state: Option<ProcessStateResponse>,
    /// Number of threads in process containing this thread.
    #[serde(skip_serializing_if = "Option::is_none", rename = "numThreads")]          
    pub threads: Option<u32>,
    /// The groups the process belongs to.
    #[serde(skip_serializing_if = "Option::is_none", rename = "groups")]          
    pub groups: Option<Vec<String>>,
    /// The uids of the process changed to usernames.
    #[serde(skip_serializing_if = "Option::is_none", rename = "uid")]          
    pub uids: Option<Vec<String>>,
    /// The gids of the process changed to group names.
    #[serde(skip_serializing_if = "Option::is_none", rename = "gid")]          
    pub gids: Option<Vec<String>>,
    /// Whether the process is monitored.
    #[serde(rename = "monitored")]          
    pub monitored: bool
}

impl ProcessResponse {

    /**
     * Create a new `ProcessResponse`.
     * 
     * `pid`: The process id.
     * `parent_pid`: The parent process id.
     * `name`: The name of the process.
     * `umask`: The umask of the process.
     * `state`: The state of the process.
     * `threads`: The number of threads in the process.
     * `groups`: The groups the process belongs to.
     * 
     * Returns a new `ProcessResponse`.
     * 
     */
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pid: Option<u32>,
        parent_pid: Option<u32>,
        name: Option<String>,
        umask: Option<String>,
        state: Option<ProcessStateResponse>,
        threads: Option<u32>,
        groups: Option<Vec<String>>,
        monitored: bool,
        gids: Option<Vec<String>>,
        uids: Option<Vec<String>>,
    ) -> ProcessResponse {
        ProcessResponse {
            pid, 
            parent_pid, 
            name, 
            umask, 
            state, 
            threads, 
            groups, 
            uids, 
            gids, 
            monitored
        }
    }

    /**
     * Create a new `ProcessResponse` from a `ProcsProcess`.
     * 
     * `proc`: The `ProcsProcess` object.
     * 
     * Returns a new `ProcessResponse`.
     * 
     */
    pub fn from_process(proc: &ProcsProcess, monitered_application_names: &[String]) -> ProcessResponse {
        ProcessResponse::new(
            proc.pid,
            proc.parent_pid,
            proc.name.clone(),
            proc.umask.clone(),
            ProcessStateResponse::from_state(&proc.state),
            proc.threads,
            proc.groups.clone(),
            Self::is_monitored(&proc.name, monitered_application_names),
            proc.gid.clone(),
            proc.uid.clone(),
        )
    }

    /**
     * Check if the process is monitored.
     * 
     * `name`: The name of the process.
     * `monitered_application_names`: The names of the monitored applications.
     * 
     * Returns true if the process is monitored, false otherwise.
     * 
     */
    fn is_monitored(name: &Option<String>, monitered_application_names: &[String]) -> bool {  
        match name {
            Some(name) => { 
                monitered_application_names.contains(name)
            },
            None => false
        }
    }

   /** 
    * Create a new `ProcessResponse` from a `ProcsProcess`.
    * 
    * `proc`: The `ProcsProcess` object.
    * 
    * Returns a new `ProcessResponse`.
    * 
    */
    pub fn from_processes(proc: &[ProcsProcess], monitered_application_names: &[String]) -> Vec<ProcessResponse> {
        proc.iter().map(|process| ProcessResponse::from_process(process, monitered_application_names)).collect()
    }
}

/**
 * The `ProcessStateResponse` enum represents the response of the process state.
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub enum ProcessStateResponse {
    /// The process is running.
    Running,
    /// The process is in an interruptable sleep.
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

impl ProcessStateResponse {
    /**
     * Create a new `ProcessStateResponse`.
     * 
     * `state`: The process state.
     * 
     * Returns a new `ProcessStateResponse`.
     * 
     */
    pub fn from_state(state: &Option<ProcessState>) -> Option<ProcessStateResponse> {
        let Some(state) = state else { return None };
        let new_state = match state {
            ProcessState::Running => ProcessStateResponse::Running,
            ProcessState::InterruptableSleep => ProcessStateResponse::InterruptableSleep,
            ProcessState::Stopped => ProcessStateResponse::Stopped,
            ProcessState::Zombie => ProcessStateResponse::Zombie,
            ProcessState::Idle => ProcessStateResponse::Idle,
            ProcessState::DiskSleep => ProcessStateResponse::DiskSleep,
            ProcessState::Dead => ProcessStateResponse::Dead,
            ProcessState::TracingStop => ProcessStateResponse::TracingStop,
            ProcessState::Unknown => ProcessStateResponse::Unknown
        };
        Some(new_state)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorResponse {
    /// Name of the monitor.
    #[serde(rename = "name")]
    name: String,
    /// Description of the monitor.
    #[serde(skip_serializing_if = "Option::is_none", rename = "description")]
    description: Option<String>,
    /// The status of the monitor.
    #[serde(rename = "status")]
    status: MonitorStatusResponse,
    /// The last time the monitor was successful.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastSuccessfulTime")]
    last_successful_time: Option<DateTime<Utc>>,
    /// The last error message.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastError")]
    last_error: Option<String>,
    /// The last time the monitor encountered an error.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastErrorTime")]
    last_error_time: Option<DateTime<Utc>>,
}

impl MonitorResponse {
    /**
     * Create a new `MonitorResponse`.
     * 
     * `name`: The name of the monitor.
     * `status`: The status of the monitor.
     * `last_successful_time`: The last time the monitor was successful.
     * `last_error`: The last error message.
     * `last_error_time`: The last time the monitor encountered an error.
     * 
     */
    pub fn new(
        name: String,
        description: Option<String>,
        status: MonitorStatusResponse,
        last_successful_time: Option<DateTime<Utc>>,
        last_error: Option<String>,
        last_error_time: Option<DateTime<Utc>>,
    ) -> MonitorResponse {
        MonitorResponse {
            name,
            description,
            status,
            last_successful_time,
            last_error,
            last_error_time,
        }
    }

    /**
     * Create a new `MonitorResponse` from a `MonitorStatus`.
     * 
     * `monitor_status`: The `MonitorStatus` object.
     * 
     * Returns a new `MonitorResponse`.
     * 
     */
    pub fn from_monitor_status_message(monitor_status: &MonitorStatus) -> MonitorResponse {
        MonitorResponse::new(
            monitor_status.name.clone(),
            monitor_status.description.clone(),
            MonitorStatusResponse::from_status(&monitor_status.status),
            monitor_status.last_successful_time,
            monitor_status.last_error.clone(),
            monitor_status.last_error_time,
        )
    }

    /**
     * Create a new `MonitorResponse` from a `MonitorStatus`.
     * 
     * `monitor_statuses`: The `MonitorStatus` object.
     * 
     * Returns a new `MonitorResponse`.
     * 
     */
    pub fn from_monitor_status_messages(monitor_statuses: &[MonitorStatus]) -> Vec<MonitorResponse> {
        monitor_statuses.iter().map(MonitorResponse::from_monitor_status_message).collect()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonitorStatusResponse {
    /// The monitor is ok.
    Ok,
    /// The monitor status is unknown.
    Unknown,
    /// The monitor has an error.
    Error,
}

impl MonitorStatusResponse {
    /**
     * Create a new `MonitorStatusResponse`.
     * 
     * `status`: The status of the monitor.
     * 
     * Returns a new `MonitorStatusResponse`.
     * 
     */
    pub fn from_status(status: &Status) -> MonitorStatusResponse {
        match status {
            Status::Ok => MonitorStatusResponse::Ok,
            Status::Unknown => MonitorStatusResponse::Unknown,
            Status::Error { message: _ } => MonitorStatusResponse::Error,
        }
    }
}

/**
 * The `StatmResponse` struct represents the response of the statm endpoint.
 * 
 * The statm file provides information about memory usage, measured in pages.
 * 
 * The statm file contains the following columns:
 * * Total program size (pages)
 * * Size of memory portions (pages)
 * * Number of pages that are shared
 * * Number of pages that are ‘code’
 * * Number of pages of data/stack
 * * Number of pages of library
 * * Number of dirty pages
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatmResponse {
    /// Total program size (pages)
    #[serde(skip_serializing_if = "Option::is_none", rename = "totalSize")]    
    pub size: Option<u32>,
    /// Size of memory portions (pages)
    #[serde(skip_serializing_if = "Option::is_none", rename = "residentSize")]         
    pub resident: Option<u32>,
    /// Number of pages that are shared
    #[serde(skip_serializing_if = "Option::is_none", rename = "sharedSize")]              
    pub share: Option<u32>,
    /// Number of pages that are ‘code’
    #[serde(skip_serializing_if = "Option::is_none", rename = "trsSize")]             
    pub trs: Option<u32>,
    /// Number of pages of data/stack
    #[serde(skip_serializing_if = "Option::is_none", rename = "drsSize")]             
    pub drs: Option<u32>,
    /// Number of pages of library
    #[serde(skip_serializing_if = "Option::is_none", rename = "lrsSize")]             
    pub lrs: Option<u32>,
    /// Number of dirty pages
    #[serde(skip_serializing_if = "Option::is_none", rename = "dtSize")]             
    pub dt: Option<u32>,
    /// Pagesize
    #[serde(skip_serializing_if = "Option::is_none", rename = "pagesize")]             
    pub pagesize: Option<u32>,
}

impl StatmResponse {

    /**
     * Create a new `StatmResponse`.
     * 
     * `procs_statm`: The procs statm.
     * 
     * Returns a new `StatmResponse`.
     */
    pub fn from_current_statm(
        procs_statm :&ProcsStatm
    ) -> StatmResponse {
        StatmResponse {
            size: procs_statm.size,
            resident: procs_statm.resident,
            share: procs_statm.share,
            trs: procs_statm.trs,
            drs: procs_statm.drs,
            lrs: procs_statm.lrs,
            dt: procs_statm.dt,
            pagesize: procs_statm.pagesize,
        }
    }   
}

/**
 * The `PingResponse` struct represents the response of the ping endpoint.
 * The ping endpoint is used to check if the monitoring agent daemon is running.
 * 
 * `status`: The status of the monitoring agent daemon. Should always be Ok.
 * `system`: The system the monitoring agent daemon is running on. Should always be monitoring-agent-daemon.
 * `name`: The name of the monitoring agent daemon. From configuration file.
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    status: String,
    system: String,
    name: String,
}

impl PingResponse {
    pub fn new(status: &str, system: &str, name: &str) -> PingResponse {
        PingResponse {
            status: status.to_string(),
            system: system.to_string(),
            name: name.to_string(),
        }
    }
}

/**
 * The `StatResponse` struct represents the response of the stat endpoint.
 * The stat endpoint is used to get the current statistics of the cpu.
 * 
 * 
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatResponse {
    /// Cpu statistics
    #[serde(skip_serializing_if = "Option::is_none", rename = "cpus")]               
    pub cpus: Option<Vec<CpuStat>>,
    /// Number of interrupts serviced since boot
    #[serde(skip_serializing_if = "Option::is_none", rename = "numInterrupts")]                    
    pub intr: Option<u64>,
    /// Number of context switches since boot
    #[serde(skip_serializing_if = "Option::is_none", rename = "contextSwitches")]
    pub ctxt: Option<u64>,
    /// Time at which the system booted, in seconds since the Unix epoch
    #[serde(skip_serializing_if = "Option::is_none", rename = "bootTime")]
    pub btime: Option<u64>,
    /// Time at which the system booted
    #[serde(skip_serializing_if = "Option::is_none", rename = "bootTimeDate")]
    pub boot_date: Option<DateTime<Utc>>,    
    /// Number of processes and threads created since boot
    #[serde(skip_serializing_if = "Option::is_none", rename = "numProcesses")]
    pub processes: Option<u64>,
    /// Number of processes currently running on CPUs
    #[serde(skip_serializing_if = "Option::is_none", rename = "processesRunning")]
    pub procs_running: Option<u64>,
    /// Number of processes currently blocked, waiting for I/O to complete
    #[serde(skip_serializing_if = "Option::is_none", rename = "processesBlocked")]    
    pub procs_blocked: Option<u64>,
}

/**
 * The `CpuStat` struct represents the cpu statistics.
 * 
 * The cpu statistics are:
 * * Time spent in user mode
 * * Time spent in system mode
 * * Time spent in user mode with low priority (nice)
 * * Time spent idle
 * * Time spent waiting for I/O to complete
 * * Time spent servicing interrupts
 * * Time spent servicing softirqs
 * * Time spent in other operating systems when running in a virtualized environment
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStat {
    /// Name of the cpu
    #[serde(skip_serializing_if = "Option::is_none", rename = "name")]
    pub name: Option<String>,    
    /// Time spent in user mode
    #[serde(skip_serializing_if = "Option::is_none", rename = "user")]
    pub user: Option<u64>,
    /// Time spent in system mode
    #[serde(skip_serializing_if = "Option::is_none", rename = "system")]
    pub system: Option<u64>,
    /// Time spent in user mode with low priority (nice)
    #[serde(skip_serializing_if = "Option::is_none", rename = "nice")]
    pub nice: Option<u64>,
    /// Time spent idle
    #[serde(skip_serializing_if = "Option::is_none", rename = "idle")]
    pub idle: Option<u64>,
    /// Time spent waiting for I/O to complete
    #[serde(skip_serializing_if = "Option::is_none", rename = "iowait")]
    pub iowait: Option<u64>,
    /// Time spent servicing interrupts
    #[serde(skip_serializing_if = "Option::is_none", rename = "irq")]
    pub irq: Option<u64>,
    /// Time spent servicing softirqs
    #[serde(skip_serializing_if = "Option::is_none", rename = "softirq")]
    pub softirq: Option<u64>,
    /// Time spent in other operating systems when running in a virtualized environment
    #[serde(skip_serializing_if = "Option::is_none", rename = "steal")]
    pub steal: Option<u64>,
}

impl StatResponse {

    /**
     * Create a new `StatResponse`.
     * 
     * `cpus`: The cpu statistics.
     * `intr`: Number of interrupts serviced since boot.
     * `ctxt`: Number of context switches since boot.
     * `btime`: Time at which the system booted, in seconds since the Unix epoch.
     * `processes`: Number of processes and threads created since boot.
     * `procs_running`: Number of processes currently running on CPUs.
     * `procs_blocked`: Number of processes currently blocked, waiting for I/O to complete.
     *  

     */
    pub fn from_stat(
        procs_stat: &ProcStat
    ) -> StatResponse {

        StatResponse {
            cpus: procs_stat.clone().cpus.map(|f| f.iter().map(|cpu| CpuStat {
                name: cpu.name.clone(),
                user: cpu.user,
                system: cpu.system,
                nice: cpu.nice,
                idle: cpu.idle,
                iowait: cpu.iowait,
                irq: cpu.irq,
                softirq: cpu.softirq,
                steal: cpu.steal,
            }).collect()),
            intr: procs_stat.intr,
            ctxt: procs_stat.ctxt,
            btime: procs_stat.btime,
            boot_date: Self::get_boot_datetime(procs_stat.btime),
            processes: procs_stat.processes,
            procs_running: procs_stat.procs_running,
            procs_blocked: procs_stat.procs_blocked,
        }
    }

    /**
     * Get the boot date and time.
     * 
     * `time`: The time at which the system booted, in seconds since the Unix epoch.
     * 
     * Returns the boot date and time.
     * 
     */
    fn get_boot_datetime(time: Option<u64>) -> Option<DateTime<Utc>> {
        let time = time?;
        let time = i64::try_from(time).map_err(|_| "Invalid time").ok()?;
        Utc.timestamp_opt(time, 0).single()
    }

}

/**
 * The `LoadavgHistoricalResponse` struct represents the response of the loadavg historical endpoint.
 * The loadavg historical endpoint is used to get the historical load average.
 * 
 * The loadavg historical endpoint contains the following columns:
 * * The 1 minute load average.
 * * The 5 minute load average.
 * * The 15 minute load average.
 */
#[allow(clippy::similar_names)]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadavgHistoricalResponse {  
    /// The 1 minute load average.  
    pub loadavg1min: Vec<HistoryElement<f64>>,
    /// The 5 minute load average.
    pub loadavg5min: Vec<HistoryElement<f64>>,
    /// The 15 minute load average.
    pub loadavg15min: Vec<HistoryElement<f64>>,
}

impl LoadavgHistoricalResponse {
    /**
     * Create a new `LoadavgHistoricalResponse`.
     * 
     * `loadavg1min`: The 1 minute load average.
     * `loadavg5min`: The 5 minute load average.
     * `loadavg15min`: The 15 minute load average.
     * 
     * Returns a new `LoadavgHistoricalResponse`.
     * 
     */
    #[allow(clippy::similar_names)]
    pub fn new(
        loadavg1min: Vec<HistoryElement<f64>>,
        loadavg5min: Vec<HistoryElement<f64>>,
        loadavg15min: Vec<HistoryElement<f64>>,
    ) -> LoadavgHistoricalResponse {
        LoadavgHistoricalResponse {
            loadavg1min,
            loadavg5min,
            loadavg15min,
        }
    }

    pub fn from_loadavg_historical(loadavg: &[LoadavgElement]) -> LoadavgHistoricalResponse {
        LoadavgHistoricalResponse::new(
            loadavg.iter().map(|element| HistoryElement {
                timestamp: element.timestamp,
                value: element.loadavg1min,
            }).collect(),
            loadavg.iter().map(|element| HistoryElement {
                timestamp: element.timestamp,
                value: element.loadavg5min,
            }).collect(),
            loadavg.iter().map(|element| HistoryElement {
                timestamp: element.timestamp,
                value: element.loadavg15min,
            }).collect(),
        )
    }
}

/**
 * The `MeminfoHistoricalResponse` struct represents the response of the meminfo historical endpoint.
 * The meminfo historical endpoint is used to get the historical memory information.
 * 
 * The meminfo historical endpoint contains the following columns:
 * * The free memory.
 */
#[allow(clippy::similar_names)]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeminfoHistoricalResponse {  
    /// The free memory.
    pub freemem: Vec<HistoryElement<u64>>,
}

impl MeminfoHistoricalResponse {
    /**
     * Create a new `MeminfoHistoricalResponse`.
     * 
     * `freemem`: The free memory.
     * 
     * Returns a new `MeminfoHistoricalResponse`.
     * 
     */
    #[allow(clippy::similar_names)]
    pub fn new(
        freemem: Vec<HistoryElement<u64>>,
    ) -> MeminfoHistoricalResponse {
        MeminfoHistoricalResponse {
            freemem,
        }
    }

    pub fn from_meminfo_historical(meminfo: &[MeminfoElement]) -> MeminfoHistoricalResponse {
        MeminfoHistoricalResponse::new(
            meminfo.iter().map(|element| HistoryElement {
                timestamp: element.timestamp,
                value: element.freemem,
            }).collect(),            
        )
    }
}

/**
 * The `ProcessMeminfoHistoricalResponse` struct represents the response of the process meminfo historical endpoint.
 * The process meminfo historical endpoint is used to get the historical memory information of a process.
 * 
 * The process meminfo historical endpoint contains the following columns:
 * `used_memory` - The size of memory portions (pages).
 */
#[allow(clippy::similar_names)]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMeminfoHistoricalResponse {  
    /// Size of memory portions (pages).
    #[serde(rename = "usedMemory")]    
    pub used_memory: Vec<ProcessMemoryHistoryElement>,
}

impl ProcessMeminfoHistoricalResponse {

    /**
     * Create a new `ProcessMeminfoHistoricalResponse`.
     * 
     * `used_memory` Used memory of the process.
     * 
     * Returns a new `ProcessMeminfoHistoricalResponse`.
     * 
     */
    #[allow(clippy::similar_names)]
    pub fn new(
        used_memory: Vec<ProcessMemoryHistoryElement>,
    ) -> ProcessMeminfoHistoricalResponse {
        ProcessMeminfoHistoricalResponse {
            used_memory
        }
    }
    /**
     * Create a new `ProcessMeminfoHistoricalResponse` from a `ProcessMemoryElement`.
     * 
     * `process_meminfo_elements`: The `ProcessMemoryElement` object.
     * 
     * Returns a new `ProcessMeminfoHistoricalResponse`.
     * 
     */
    pub fn from_process_meminfo_historical(process_meminfo_elements: &[ProcessMemoryElement]) -> ProcessMeminfoHistoricalResponse {
        let mut elements = Vec::new();
        
        for element in process_meminfo_elements {
            elements.push(ProcessMemoryHistoryElement::new(
                element.timestamp,
                element.resident,
                element.share,
                element.trs,
                element.drs,
                element.lrs,
                element.dt,
            ));
        }
        ProcessMeminfoHistoricalResponse::new(elements)
    }

}

/**
 * The `ProcessStatmHistoricalResponse` struct represents the response of the process statm historical endpoint.
 * The process statm historical endpoint is used to get the historical memory information of a process.
 * 
 * The process statm historical endpoint contains the following columns:
 * * Total program size (pages)
 * * Size of memory portions (pages)
 * * Number of pages that are shared
 * * Number of pages that are ‘code’
 * * Number of pages of data/stack
 * * Number of pages of library
 * * Number of dirty pages
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMemoryHistoryElement {
    /// Timestamp of the memory information.
    #[serde(rename = "timestamp")]
    pub timestamp: DateTime<Utc>,  
    /// Size of memory portions (pages)
    #[serde(skip_serializing_if = "Option::is_none", rename = "residentSize")]         
    pub resident: Option<u64>,
    /// Number of pages that are shared
    #[serde(skip_serializing_if = "Option::is_none", rename = "sharedSize")]              
    pub share: Option<u64>,
    /// Number of pages that are ‘code’
    #[serde(skip_serializing_if = "Option::is_none", rename = "trsSize")]             
    pub trs: Option<u64>,
    /// Number of pages of data/stack
    #[serde(skip_serializing_if = "Option::is_none", rename = "drsSize")]             
    pub drs: Option<u64>,
    /// Number of pages of library
    #[serde(skip_serializing_if = "Option::is_none", rename = "lrsSize")]             
    pub lrs: Option<u64>,
    /// Number of dirty pages
    #[serde(skip_serializing_if = "Option::is_none", rename = "dtSize")]             
    pub dt: Option<u64>,     
}

impl ProcessMemoryHistoryElement {
    /**
     * Create a new `ProcessMemoryHistoryElement`.
     * 
     * `timestamp`: The timestamp of the memory information.
     * `resident`: The size of memory portions (pages).
     * `share`: The number of pages that are shared.
     * `trs`: The number of pages that are ‘code’.
     * `drs`: The number of pages of data/stack.
     * `lrs`: The number of pages of library.
     * `dt`: The number of dirty pages.
     * 
     * Returns a new `ProcessMemoryHistoryElement`.
     * 
     */
    pub fn new(
        timestamp: DateTime<Utc>,
        resident: Option<u64>,
        share: Option<u64>,
        trs: Option<u64>,
        drs: Option<u64>,
        lrs: Option<u64>,
        dt: Option<u64>,
    ) -> ProcessMemoryHistoryElement {
        ProcessMemoryHistoryElement {
            timestamp,
            resident,
            share,
            trs,
            drs,
            lrs,
            dt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryElement<T> {
    #[serde(rename = "timestamp")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "value")]
    pub value: T,
}


#[cfg(test)]
mod test {
    use std::vec;

    use monitoring_agent_lib::proc::ProcCpuStat;

    use super::*;

    #[test]
    fn test_meminfo_response_new() {
        let meminfo_response = MeminfoResponse::new(Some(100), Some(50), Some(25), Some(200), Some(100));
        assert_eq!(meminfo_response.total_mem, Some(100));
        assert_eq!(meminfo_response.free_mem, Some(50));
        assert_eq!(meminfo_response.available_mem, Some(25));
        assert_eq!(meminfo_response.swap_total, Some(200));
        assert_eq!(meminfo_response.swap_free, Some(100));
    }

    #[test]
    fn test_meminfo_response_from_meminfo() {
        let procs_meminfo = ProcsMeminfo {
            memtotal: Some(100),
            memfree: Some(50),
            memavailable: Some(25),
            swaptotal: Some(200),
            swapfree: Some(100),
        };
        let meminfo_response = MeminfoResponse::from_meminfo(&procs_meminfo);
        assert_eq!(meminfo_response.total_mem, Some(100));
        assert_eq!(meminfo_response.free_mem, Some(50));
        assert_eq!(meminfo_response.available_mem, Some(25));
        assert_eq!(meminfo_response.swap_total, Some(200));
        assert_eq!(meminfo_response.swap_free, Some(100));
    }

    #[test]
    fn test_cpuinfo_response_new() {
        let cpuinfo_response = CpuinfoResponse::new(Some(1), Some("AuthenticAMD".to_string()), Some("cpuFamily".to_string()), Some("model".to_string()), Some("modelName".to_string()), Some(2), Some(3.0));
        assert_eq!(cpuinfo_response.apicid, Some(1));
        assert_eq!(cpuinfo_response.vendor_id, Some("AuthenticAMD".to_string()));
        assert_eq!(cpuinfo_response.cpu_family, Some("cpuFamily".to_string()));
        assert_eq!(cpuinfo_response.model, Some("model".to_string()));
        assert_eq!(cpuinfo_response.model_name, Some("modelName".to_string()));
        assert_eq!(cpuinfo_response.cpu_cores, Some(2));
        assert_eq!(cpuinfo_response.cpu_mhz, Some(3.0));
    }

    #[test]
    fn test_cpuinfo_response_from_cpuinfo() {
        let procs_cpuinfo = vec![ProcsCpuinfo {
            apicid: Some(1),
            vendor_id: Some("AuthenticAMD".to_string()),
            cpu_family: Some("cpuFamily".to_string()),
            model: Some("model".to_string()),
            model_name: Some("modelName".to_string()),
            cpu_cores: Some(2),
            cpu_mhz: Some(3.0),
        }];
        let cpuinfo_response = CpuinfoResponse::from_cpuinfo(&procs_cpuinfo);
        assert_eq!(cpuinfo_response[0].apicid, Some(1));
        assert_eq!(cpuinfo_response[0].vendor_id, Some("AuthenticAMD".to_string()));
        assert_eq!(cpuinfo_response[0].cpu_family, Some("cpuFamily".to_string()));
        assert_eq!(cpuinfo_response[0].model, Some("model".to_string()));
        assert_eq!(cpuinfo_response[0].model_name, Some("modelName".to_string()));
        assert_eq!(cpuinfo_response[0].cpu_cores, Some(2));
        assert_eq!(cpuinfo_response[0].cpu_mhz, Some(3.0));
    }

    #[test]
    fn test_loadavg_response_new() {
        let loadavg_response = LoadavgResponse::new(Some(1.0), Some(2.0), Some(3.0), Some(4), Some(5));
        assert_eq!(loadavg_response.loadavg1min, Some(1.0));
        assert_eq!(loadavg_response.loadavg5min, Some(2.0));
        assert_eq!(loadavg_response.loadavg15min, Some(3.0));
        assert_eq!(loadavg_response.current_running_processes, Some(4));
        assert_eq!(loadavg_response.total_number_of_processes, Some(5));
    }

    #[test]
    fn test_loadavg_response_from_loadavg() {
        let procs_loadavg = ProcsLoadavg {
            loadavg1min: Some(1.0),
            loadavg5min: Some(2.0),
            loadavg15min: Some(3.0),
            current_running_processes: Some(4),
            total_number_of_processes: Some(5),
        };
        let loadavg_response = LoadavgResponse::from_loadavg(&procs_loadavg);
        assert_eq!(loadavg_response.loadavg1min, Some(1.0));
        assert_eq!(loadavg_response.loadavg5min, Some(2.0));
        assert_eq!(loadavg_response.loadavg15min, Some(3.0));
        assert_eq!(loadavg_response.current_running_processes, Some(4));
        assert_eq!(loadavg_response.total_number_of_processes, Some(5));
    }

    #[test]
    fn test_process_response_new() {
        let process_response = ProcessResponse::new(Some(1), Some(2), Some("name".to_string()), Some("umask".to_string()), Some(ProcessStateResponse::Running), Some(3), Some(vec!["group1".to_string(), "group2".to_string()]), true, Some(Vec::new()), Some(Vec::new()));
        assert_eq!(process_response.pid, Some(1));
        assert_eq!(process_response.parent_pid, Some(2));
        assert_eq!(process_response.name, Some("name".to_string()));
        assert_eq!(process_response.umask, Some("umask".to_string()));
        assert_eq!(process_response.state, Some(ProcessStateResponse::Running));
        assert_eq!(process_response.threads, Some(3));
        assert_eq!(process_response.groups, Some(vec!["group1".to_string(), "group2".to_string()]));
    }

    #[test]
    fn test_process_response_from_process() {
        let procs_process = ProcsProcess {
            pid: Some(1),
            parent_pid: Some(2),
            name: Some("name".to_string()),
            umask: Some("umask".to_string()),
            state: Some(ProcessState::Running),
            threads: Some(3),
            groups: Some(vec!["group1".to_string(), "group2".to_string()]),
            uid: Some(Vec::new()),
            gid: Some(Vec::new()),
        };
        let process_response = ProcessResponse::from_process(&procs_process, &Vec::new());
        assert_eq!(process_response.pid, Some(1));
        assert_eq!(process_response.parent_pid, Some(2));
        assert_eq!(process_response.name, Some("name".to_string()));
        assert_eq!(process_response.umask, Some("umask".to_string()));
        assert_eq!(process_response.state, Some(ProcessStateResponse::Running));
        assert_eq!(process_response.threads, Some(3));
        assert_eq!(process_response.groups, Some(vec!["group1".to_string(), "group2".to_string()]));
    }

    #[test]
    fn test_process_state_response_from_state() {
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Running)), Some(ProcessStateResponse::Running));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Running)), Some(ProcessStateResponse::Running));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::InterruptableSleep)), Some(ProcessStateResponse::InterruptableSleep));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Stopped)), Some(ProcessStateResponse::Stopped));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Zombie)), Some(ProcessStateResponse::Zombie));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Idle)), Some(ProcessStateResponse::Idle));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::DiskSleep)), Some(ProcessStateResponse::DiskSleep));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Dead)), Some(ProcessStateResponse::Dead));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::TracingStop)), Some(ProcessStateResponse::TracingStop));
        assert_eq!(ProcessStateResponse::from_state(&Some(ProcessState::Unknown)), Some(ProcessStateResponse::Unknown));
        assert_eq!(ProcessStateResponse::from_state(&None), None);        
    }

    #[test]
    fn test_from_processes() {
        let procs_process = vec![ProcsProcess {
            pid: Some(1),
            parent_pid: Some(2),
            name: Some("name".to_string()),
            umask: Some("umask".to_string()),
            state: Some(ProcessState::Running),
            threads: Some(3),
            groups: Some(vec!["group1".to_string(), "group2".to_string()]),
            uid: Some(Vec::new()),
            gid: Some(Vec::new()),
            
        }];
        let process_response = ProcessResponse::from_processes(&procs_process, &Vec::new());
        assert_eq!(process_response[0].pid, Some(1));
        assert_eq!(process_response[0].parent_pid, Some(2));
        assert_eq!(process_response[0].name, Some("name".to_string()));
        assert_eq!(process_response[0].umask, Some("umask".to_string()));
        assert_eq!(process_response[0].state, Some(ProcessStateResponse::Running));
        assert_eq!(process_response[0].threads, Some(3));
        assert_eq!(process_response[0].groups, Some(vec!["group1".to_string(), "group2".to_string()]));
    }

    #[test]
    fn test_from_monitor_status_message() {
        let monitor_status = MonitorStatus {
            name: "name".to_string(),
            description: None,
            status: Status::Ok,
            last_successful_time: Some(Utc::now()),
            last_error: Some("error".to_string()),
            last_error_time: Some(Utc::now()),
        };
        let monitor_response = MonitorResponse::from_monitor_status_message(&monitor_status);
        assert_eq!(monitor_response.name, "name".to_string());
        assert_eq!(monitor_response.status, MonitorStatusResponse::Ok);
        assert_eq!(monitor_response.last_successful_time.is_some(), true);
        assert_eq!(monitor_response.last_error, Some("error".to_string()));
        assert_eq!(monitor_response.last_error_time.is_some(), true);
    }

    #[test]
    fn test_new_moniorresponse() {
        let monitor_response = MonitorResponse::new("name".to_string(), None, MonitorStatusResponse::Ok, Some(Utc::now()), Some("error".to_string()), Some(Utc::now()));
        assert_eq!(monitor_response.name, "name".to_string());
        assert_eq!(monitor_response.status, MonitorStatusResponse::Ok);
        assert_eq!(monitor_response.last_successful_time.is_some(), true);
        assert_eq!(monitor_response.last_error, Some("error".to_string()));
        assert_eq!(monitor_response.last_error_time.is_some(), true);        
    }

    #[test]
    fn test_from_monitor_status_messages() {
        let monitor_status = vec![MonitorStatus {
            name: "name".to_string(),
            description: None,
            status: Status::Ok,
            last_successful_time: Some(Utc::now()),
            last_error: Some("error".to_string()),
            last_error_time: Some(Utc::now()),
        }];
        let monitor_response = MonitorResponse::from_monitor_status_messages(&monitor_status);
        assert_eq!(monitor_response[0].name, "name".to_string());
        assert_eq!(monitor_response[0].status, MonitorStatusResponse::Ok);
        assert_eq!(monitor_response[0].last_successful_time.is_some(), true);
        assert_eq!(monitor_response[0].last_error, Some("error".to_string()));
        assert_eq!(monitor_response[0].last_error_time.is_some(), true);        
    }

    #[test]
    fn test_from_status() {
        assert_eq!(MonitorStatusResponse::from_status(&Status::Ok), MonitorStatusResponse::Ok);
        assert_eq!(MonitorStatusResponse::from_status(&Status::Unknown), MonitorStatusResponse::Unknown);
        assert_eq!(MonitorStatusResponse::from_status(&Status::Error { message: "error".to_string() }), MonitorStatusResponse::Error);
    }

    #[test]
    fn test_from_statm() {
        let procs_statm = ProcsStatm {
            size: Some(1),
            resident: Some(2),
            share: Some(3),
            trs: Some(4),
            drs: Some(5),
            lrs: Some(6),
            dt: Some(7),
            pagesize: Some(4096),
        };
        let statm_response = StatmResponse::from_current_statm(&procs_statm);
        assert_eq!(statm_response.size, Some(1));
        assert_eq!(statm_response.resident, Some(2));
        assert_eq!(statm_response.share, Some(3));
        assert_eq!(statm_response.trs, Some(4));
        assert_eq!(statm_response.drs, Some(5));
        assert_eq!(statm_response.lrs, Some(6));
        assert_eq!(statm_response.dt, Some(7));
        assert_eq!(statm_response.pagesize, Some(4096));        
    }

    #[test]
    fn test_from_statm_none() {
        let procs_statm = ProcsStatm {
            size: None,
            resident: None,
            share: None,
            trs: None,
            drs: None,
            lrs: None,
            dt: None,
            pagesize: None,
        };
        let statm_response = StatmResponse::from_current_statm(&procs_statm);
        assert_eq!(statm_response.size, None);
        assert_eq!(statm_response.resident, None);
        assert_eq!(statm_response.share, None);
        assert_eq!(statm_response.trs, None);
        assert_eq!(statm_response.drs, None);
        assert_eq!(statm_response.lrs, None);
        assert_eq!(statm_response.dt, None);        
        assert_eq!(statm_response.pagesize, None);        
    }    

    #[test]
    fn test_stat_response_none() {
        let procs_stat = ProcStat {
            cpus: None,
            intr: None,
            ctxt: None,
            btime: None,
            processes: None,
            procs_running: None,
            procs_blocked: None,
        };
        let stat_response = StatResponse::from_stat(&procs_stat);
        assert_eq!(stat_response.intr, None);
        assert_eq!(stat_response.ctxt, None);
        assert_eq!(stat_response.btime, None);
        assert_eq!(stat_response.processes, None);
        assert_eq!(stat_response.procs_running, None);
        assert_eq!(stat_response.procs_blocked, None);
    }

    #[test]
    fn test_stat_response_some() {
        let procs_stat = ProcStat {
            cpus: Some(vec![ProcCpuStat {
                name: Some("cpu".to_string()),
                user: Some(1),
                system: Some(2),
                nice: Some(3),
                idle: Some(4),
                iowait: Some(5),
                irq: Some(6),
                softirq: Some(7),
                steal: Some(8),
            }]),
            intr: Some(1),
            ctxt: Some(2),
            btime: Some(3),
            processes: Some(4),
            procs_running: Some(5),
            procs_blocked: Some(6),
        };
        let stat_response = StatResponse::from_stat(&procs_stat);
        assert_eq!(stat_response.cpus.clone().is_some(), true);
        assert_eq!(stat_response.cpus.clone().unwrap()[0].name, Some("cpu".to_string()));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].user, Some(1));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].system, Some(2));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].nice, Some(3));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].idle, Some(4));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].iowait, Some(5));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].irq, Some(6));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].softirq, Some(7));
        assert_eq!(stat_response.cpus.clone().unwrap()[0].steal, Some(8));
        assert_eq!(stat_response.intr, Some(1));
        assert_eq!(stat_response.ctxt, Some(2));
        assert_eq!(stat_response.btime, Some(3));
        assert_eq!(stat_response.processes, Some(4));
        assert_eq!(stat_response.procs_running, Some(5));
        assert_eq!(stat_response.procs_blocked, Some(6));
    }
        
}