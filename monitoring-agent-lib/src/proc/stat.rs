use std::{fs::File, io::{BufRead, BufReader}};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::common::CommonLibError;

/**
 * CPU statistics from /proc/stat
 * 
 * The first line of /proc/stat contains overall CPU statistics. Each subsequent line contains CPU statistics for a single core.
 * 
 * The columns are as follows:
 * `cpus`: A list of CPU statistics for each core.
 * `intr`: Number of interrupts serviced since boot.
 * `ctxt`: Number of context switches since boot.
 * `btime`: Time at which the system booted, in seconds since the Unix epoch.
 * `processes`: Number of processes and threads created since boot.
 * `procs_running`: Number of processes currently running on CPUs.
 * `procs_blocked`: Number of processes currently blocked, waiting for I/O to complete.
 * 
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcStat {
    /// Cpu statistics
    pub cpus: Option<Vec<ProcCpuStat>>,
    /// Number of interrupts serviced since boot
    pub intr: Option<u64>,
    /// Number of context switches since boot
    pub ctxt: Option<u64>,
    /// Time at which the system booted, in seconds since the Unix epoch
    pub btime: Option<u64>,
    /// Number of processes and threads created since boot
    pub processes: Option<u64>,
    /// Number of processes currently running on CPUs
    pub procs_running: Option<u64>,
    /// Number of processes currently blocked, waiting for I/O to complete
    pub procs_blocked: Option<u64>,
}

impl ProcStat {

    /**
     * Create a new `ProcStat`.
     * 
     * ```
     * use monitoring_agent_lib::proc::stat::{ProcStat, ProcCpuStat};
     * ProcStat::new(None, None, None, None, None, None, None);
     * ```
     * 
     * ```
     * use monitoring_agent_lib::proc::stat::{ProcStat, ProcCpuStat};
     * ProcStat::new(Some(vec![ProcCpuStat::new(Some("cpu"), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0))]), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0));
     * ```
     * 
     * `cpus`: A list of CPU statistics for each core.
     * `intr`: Number of interrupts serviced since boot.
     * `ctxt`: Number of context switches since boot.
     * `btime`: Time at which the system booted, in seconds since the Unix epoch.
     * `processes`: Number of processes and threads created since boot.
     * `procs_running`: Number of processes currently running on CPUs.
     * `procs_blocked`: Number of processes currently blocked, waiting for I/O to complete.
     * 
     * Returns a new `ProcStat`.
     * 
     */
    #[must_use]
    pub fn new(cpus: Option<Vec<ProcCpuStat>>,
                    intr: Option<u64>,
                    ctxt: Option<u64>,
                    btime: Option<u64>,
                    processes: Option<u64>,
                    procs_running: Option<u64>,
                    procs_blocked: Option<u64>,) -> Self {
        ProcStat {
            cpus,
            intr,
            ctxt,
            btime,
            processes,
            procs_running,
            procs_blocked,
        }
    }

    /**
     * Get the cpu statistics.
     * 
     * ```
     * use monitoring_agent_lib::proc::stat::ProcStat;
     * ProcStat::get_stat();
     * ```
     * 
     * Returns the cpu statistics.
     * 
     */
    #[tracing::instrument(level = "debug")]
    pub fn get_stat() -> Result<ProcStat, CommonLibError> {
        let loadavg_file = "/proc/stat";
        ProcStat::read_stat(loadavg_file)
    }    

    /**
     * Read the cpu statistics from the file.
     * 
     * `file`: The file to read.
     * 
     * Returns the cpu statistics or an error.
     * 
     */
    fn read_stat(file: &str) -> Result<ProcStat, CommonLibError> {
        let file = File::open(file).map_err(|err| CommonLibError::new(&format!("Failed to read file {err:?}")))?;
        let reader = BufReader::new(file);
        let mut cpus: Vec<ProcCpuStat> = Vec::new();
        let mut intr: Option<u64> = None;
        let mut ctxt: Option<u64> = None;
        let mut btime: Option<u64> = None;
        let mut processes: Option<u64> = None;
        let mut procs_running: Option<u64> = None;
        let mut procs_blocked: Option<u64> = None;

        for line in reader.lines() {
            let line = line.map_err(|err| CommonLibError::new(&format!("Error reading line {err:?}")))?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            let first_part = parts.first().map_or("", |f| f);
            match first_part {
                name if name.starts_with("cpu") => {
                    let cpu = ProcCpuStat::read_line(&line);
                    cpus.push(cpu);
                },
                "intr" => {
                    intr = parts.get(1).and_then(|s| u64::from_str(s).ok());
                },
                "ctxt" => {
                    ctxt = parts.get(1).and_then(|s| u64::from_str(s).ok());
                },
                "btime" => {
                    btime = parts.get(1).and_then(|s| u64::from_str(s).ok());
                },
                "processes" => {
                    processes = parts.get(1).and_then(|s| u64::from_str(s).ok());
                },
                "procs_running" => {
                    procs_running = parts.get(1).and_then(|s| u64::from_str(s).ok());
                },
                "procs_blocked" => {
                    procs_blocked = parts.get(1).and_then(|s| u64::from_str(s).ok());
                }, 
                _ => {
                    continue;
                },
            }
        }
        Ok(ProcStat::new(Some(cpus), intr, ctxt, btime, processes, procs_running, procs_blocked))
    }
}

/**
 * CPU statistics from /proc/stat
 * 
 * The columns are as follows:
 * `name`: The name of the CPU. Example cpu or cpu0. cpu is the total of all CPUs.
 * `user`: Time spent in user mode.
 * `nice`: Time spent in user mode with low priority (nice).
 * `system`: Time spent in system mode.
 * `idle`: Time spent in the idle task.
 * `iowait`: Time spent waiting for I/O to complete.
 * `irq`: Time spent servicing interrupts.
 * `softirq`: Time spent servicing softirqs.
 * `steal`: Time spent in other OS instances when running in a virtualized environment.
 * 
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcCpuStat {
    /// The name of the CPU. Example cpu or cpu0. cpu is the total of all CPUs.
    pub name: Option<String>,
    /// Time spent in user mode.
    pub user: Option<u64>,
    /// Time spent in user mode with low priority (nice).
    pub nice: Option<u64>,
    /// Time spent in system mode.
    pub system: Option<u64>,
    /// Time spent in the idle task.
    pub idle: Option<u64>,
    /// Time spent waiting for I/O to complete.
    pub iowait: Option<u64>,
    /// Time spent servicing interrupts.
    pub irq: Option<u64>,
    /// Time spent servicing softirqs.
    pub softirq: Option<u64>,
    /// Time spent in other OS instances when running in a virtualized environment.
    pub steal: Option<u64>,
}


impl ProcCpuStat {

    /**
     * Create a new `ProcCpuStat`.
     * 
     * ```
     * use monitoring_agent_lib::proc::stat::ProcCpuStat;
     * ProcCpuStat::new(None, None, None, None, None, None, None, None, None);
     * ```
     * 
     * ```
     * use monitoring_agent_lib::proc::stat::ProcCpuStat;
     * ProcCpuStat::new(Some("cpu"), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0));
     * ```
     * 
     * `name`: The name of the CPU. Example cpu or cpu0. cpu is the total of all CPUs.
     * `user`: Time spent in user mode.
     * `nice`: Time spent in user mode with low priority (nice).
     * `system`: Time spent in system mode.
     * `idle`: Time spent in the idle task.
     * `iowait`: Time spent waiting for I/O to complete.
     * `irq`: Time spent servicing interrupts.
     * `softirq`: Time spent servicing softirqs.
     * `steal`: Time spent in other OS instances when running in a virtualized environment.
     * 
     */
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new( name: Option<&str>,
                user: Option<u64>,
                nice: Option<u64>,
                system: Option<u64>,
                idle: Option<u64>,
                iowait: Option<u64>,
                irq: Option<u64>,
                softirq: Option<u64>,
                steal: Option<u64>) -> Self {
        ProcCpuStat {
            name: name.map(std::string::ToString::to_string),
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
        }        
    }

    /**
     * Read the cpu statistics from the line.
     * 
     * `line`: The line to read.
     * 
     * Returns the cpu statistics.
     * 
     */
    fn read_line(line: &str) -> ProcCpuStat {
        let parts: Vec<&str> = line.split_whitespace().collect();
        ProcCpuStat::new(
            parts.first().map(std::borrow::ToOwned::to_owned),
            parts.get(1).and_then(|s| u64::from_str(s).ok()),
            parts.get(2).and_then(|s| u64::from_str(s).ok()),
            parts.get(3).and_then(|s| u64::from_str(s).ok()),
            parts.get(4).and_then(|s| u64::from_str(s).ok()),
            parts.get(5).and_then(|s| u64::from_str(s).ok()),
            parts.get(6).and_then(|s| u64::from_str(s).ok()),
            parts.get(7).and_then(|s| u64::from_str(s).ok()),
            parts.get(8).and_then(|s| u64::from_str(s).ok()),
        )
    }

}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_stat() {
        let proc_stat = ProcStat::get_stat();
        assert!(proc_stat.is_ok());
    }

    #[test]
    fn test_read_stat() {
        let proc_stat = ProcStat::read_stat("resources/test/test_stat").unwrap();
        assert_eq!(proc_stat.clone().cpus.unwrap().len(), 17);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].name.clone().unwrap(), "cpu");
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].user.clone().unwrap(), 728050);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].nice.clone().unwrap(), 301008);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].system.clone().unwrap(), 1228186);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].idle.clone().unwrap(), 43365149);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].iowait.clone().unwrap(), 613178);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].irq.clone().unwrap(), 0);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].softirq.clone().unwrap(), 21734);
        assert_eq!(proc_stat.clone().cpus.unwrap()[0].steal.clone().unwrap(), 0);
        assert_eq!(proc_stat.clone().intr.unwrap(), 793571364);
        assert_eq!(proc_stat.clone().ctxt.unwrap(), 1526901585);
        assert_eq!(proc_stat.clone().btime.unwrap(), 1724165385);
        assert_eq!(proc_stat.clone().processes.unwrap(), 54612);
        assert_eq!(proc_stat.clone().procs_running.unwrap(), 3);
        assert_eq!(proc_stat.clone().procs_blocked.unwrap(), 0);
    }

}