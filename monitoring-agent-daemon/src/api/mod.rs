mod meminfo;
mod state;
mod response;
mod cpuinfo;
mod loadavg;
mod process;

pub use crate::api::meminfo::get_current_meminfo;
pub use crate::api::cpuinfo::get_current_cpuinfo;
pub use crate::api::loadavg::get_current_loadavg;
pub use crate::api::process::{get_processes, get_process, get_threads};

#[allow(clippy::module_name_repetitions)]
pub use crate::api::state::StateApi;

