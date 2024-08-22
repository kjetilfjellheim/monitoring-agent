/**
 * API module. Contains the APIs made available by the monitoring agent daemon.
 * 
 * `meminfo`: The memory information API.
 * `state`: The state API. This object is injected into all web service handlers.
 * `response`: The response API.
 * `cpuinfo`: The cpu information API.
 * `loadavg`: The load average API.
 * `process`: The process API.
 * `monitor`: The monitor API.
 * `statm`: The statm API.
 * `common`: The common API.
 * `stat`: The stat API.
 * `ping`: The ping API.
 */
mod meminfo;
mod state;
mod response;
mod cpuinfo;
mod loadavg;
mod process;
mod monitor;
mod common;
mod stat;
mod ping;

pub use crate::api::meminfo::get_current_meminfo;
pub use crate::api::cpuinfo::get_current_cpuinfo;
pub use crate::api::loadavg::get_current_loadavg;
pub use crate::api::process::{get_processes, get_process, get_threads, get_current_statm};
pub use crate::api::monitor::get_monitor_status;
pub use crate::api::stat::get_stat;
pub use crate::api::ping::get_ping;

#[allow(clippy::module_name_repetitions)]
pub use crate::api::state::StateApi;

