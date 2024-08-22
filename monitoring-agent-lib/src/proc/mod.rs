/// Structure and methods to read and parse /proc/cpuinfo 
pub mod cpuinfo;
/// Structure and methods to read and parse /proc/meminfo 
pub mod meminfo;
/// Structure and methods to read and parse /proc/loadavg
pub mod loadavg;
/// Structure and methods to read and parse /proc/*/status 
pub mod process;
/// Memory use of a process
pub mod statm;
/// Structure and methods to read and parse /proc/*/cmdline
pub mod cmdline;
/// Structure and methods to read and parse /proc/stat
pub mod stat;

pub use crate::proc::cpuinfo::ProcsCpuinfo;
pub use crate::proc::meminfo::ProcsMeminfo;
pub use crate::proc::loadavg::ProcsLoadavg;
pub use crate::proc::process::ProcsProcess;
pub use crate::proc::statm::ProcsStatm;
pub use crate::proc::cmdline::ProcsCmdLine;
#[allow(clippy::module_name_repetitions)]
pub use crate::proc::stat::{ ProcStat, ProcCpuStat };