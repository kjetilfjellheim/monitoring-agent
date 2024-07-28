/**
 * API module. Contains the APIs made available by the monitoring agent daemon.
 * 
 * `meminfo`: The memory information API.
 * `state`: The state API. This object is injected into all web service handlers.
 * `response`: The response API.
 * `cpuinfo`: The cpu information API.
 * `loadavg`: The load average API.
 * `process`: The process API.
 */
mod meminfo;
mod state;
mod response;
mod cpuinfo;
mod loadavg;
mod process;
mod monitor;

pub use crate::api::meminfo::get_current_meminfo;
pub use crate::api::cpuinfo::get_current_cpuinfo;
pub use crate::api::loadavg::get_current_loadavg;
pub use crate::api::process::{get_processes, get_process, get_threads};
pub use crate::api::monitor::get_monitor_status;

#[allow(clippy::module_name_repetitions)]
pub use crate::api::state::StateApi;

