/* 
 * Common structures for reading proc files.
 */
pub mod cpuinfo;
pub mod meminfo;
pub mod loadavg;
pub mod process;

pub use crate::proc::cpuinfo::ProcsCpuinfo;
pub use crate::proc::meminfo::ProcsMeminfo;
pub use crate::proc::loadavg::Loadavg;
pub use crate::proc::process::ProcsProcess;