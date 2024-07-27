use monitoring_agent_lib::proc::ProcsMeminfo;
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