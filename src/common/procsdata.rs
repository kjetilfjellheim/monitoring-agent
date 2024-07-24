use serde::{Deserialize, Serialize};

/**
 * cpu information from /cat/cpuinfo
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcsCpuinfo {
    pub apicid: Option<u8>,
    pub vendor_id:  Option<String>,
    pub cpu_family:  Option<String>,
    pub model:  Option<String>,
    pub model_name:  Option<String>,
    pub cpu_cores:  Option<u8>,
    pub cpu_mhz:  Option<f32>,
}

impl ProcsCpuinfo {
    /**
     * Create a new `Cpuinfo`.
     *
     * `apicid`: The apicid of the cpu.
     * `vendor_id`: The vendor id of the cpu.
     * `cpu_family`: The cpu family.
     * `model`: The model of the cpu.
     * `model_name`: The model name of the cpu.
     * `cpu_cores`: The number of cores in the cpu.
     * `cpu_mhz`: The speed of the cpu in mhz.
     */
    pub fn new(
        apicid: Option<u8>,
        vendor_id: Option<String>,
        cpu_family: Option<String>,
        model: Option<String>,
        model_name: Option<String>,
        cpu_cores: Option<u8>,
        cpu_mhz: Option<f32>,
    ) -> ProcsCpuinfo {
        ProcsCpuinfo {
            apicid,
            vendor_id,
            cpu_family,
            model,
            model_name,
            cpu_cores,
            cpu_mhz,
        }
    }
}

/**
 * Memory information from /cat/meminfo
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcsMeminfo {
    pub memtotal: Option<u64>,
    pub memfree: Option<u64>,
    pub memavailable: Option<u64>,
    pub swaptotal: Option<u64>,
    pub swapfree: Option<u64>,
}

impl ProcsMeminfo {

    /**
     * Create a new `Meminfo`.
     *
     * `memtotal`: The total memory.
     * `memfree`: The free memory.
     * `memavailable`: The available memory.
     * `swaptotal`: The total swap.
     * `swapfree`: The free swap.
     */
    pub fn new(
        memtotal: Option<u64>,
        memfree: Option<u64>,
        memavailable: Option<u64>,
        swaptotal: Option<u64>,
        swapfree: Option<u64>,
    ) -> ProcsMeminfo {
        ProcsMeminfo {
            memtotal,
            memfree,
            memavailable,
            swaptotal,
            swapfree,
        }
    }
}