/// Structure and methods to read and parse /proc/cpuinfo 
pub mod cpuinfo;
/// Structure and methods to read and parse /proc/meminfo 
pub mod meminfo;
/// Structure and methods to read and parse /proc/loadavg
pub mod loadavg;
/// Structure and methods to read and parse /proc/*/status 
pub mod process;

pub use crate::proc::cpuinfo::ProcsCpuinfo;
pub use crate::proc::meminfo::ProcsMeminfo;
pub use crate::proc::loadavg::Loadavg;
pub use crate::proc::process::ProcsProcess;