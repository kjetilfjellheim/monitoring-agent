/* 
 * Common structures for reading proc files.
 */
pub mod cpuinfo;
pub mod meminfo;

pub use crate::proc::cpuinfo::ProcsCpuinfo;
pub use crate::proc::meminfo::ProcsMeminfo;