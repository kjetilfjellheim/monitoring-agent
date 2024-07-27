use monitoring_agent_lib::proc::{ProcsLoadavg, ProcsCpuinfo, ProcsMeminfo};
use serde::{Deserialize, Serialize};

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
        MeminfoResponse::new(procs_mem_info.memtotal, procs_mem_info.memfree, procs_mem_info.memavailable, procs_mem_info.swaptotal, procs_mem_info.swaptotal)
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
    #[serde(skip_serializing_if = "Option::is_none", rename = "loadAvg10Min")]        
    pub loadavg10min:  Option<f32>,
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
     * `loadavg10min`: The 10 minute load average.
     * 
     * Returns a new `CpuinfoResponse`.
     */
    #[allow(clippy::similar_names)]
    pub fn new(        
        loadavg1min: Option<f32>,
        loadavg5min: Option<f32>,
        loadavg10min: Option<f32>,
        current_running_processes: Option<u32>,
        total_number_of_processes: Option<u32>,
    ) -> LoadavgResponse {
        LoadavgResponse {
            loadavg1min,
            loadavg5min,
            loadavg10min,
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
            procs_loadavg.loadavg10min, 
            procs_loadavg.current_running_processes, 
            procs_loadavg.total_number_of_processes)
    }    

}