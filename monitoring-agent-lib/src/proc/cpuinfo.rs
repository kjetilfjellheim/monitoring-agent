use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}};
use std::str::FromStr;

use log::error;
use serde::{Deserialize, Serialize};

use crate::common::CommonLibError;

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

    /**
     * Get the apicid of the cpu.
     */
    pub fn get_cpuinfo() -> Result<Vec<ProcsCpuinfo>, CommonLibError> {
        let cpuinfo_file = "/proc/cpuinfo";
        ProcsCpuinfo::read_cpuinfo(cpuinfo_file)
    }

    /**
     * Read the cpuinfo file.
     * 
     * `file`: The file to read.
     * 
     * Returns the cpuinfo data or an error.
     */
    fn read_cpuinfo(file: &str) -> Result<Vec<ProcsCpuinfo>, CommonLibError> {
        let mut cpuinfo_data: Vec<ProcsCpuinfo> = Vec::new();
        let cpuinfo_file = File::open(file);
        match cpuinfo_file {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut parts: HashMap<String, String> = HashMap::new();                
                for line in reader.lines() {
                    let line = ProcsCpuinfo::get_line(line)?;                    
                    if line.is_empty() {
                        let cpuinfo = ProcsCpuinfo::new(
                            parts.get("apicid").and_then(|f| u8::from_str(f).ok()),
                            parts.get("vendor_id").cloned(),
                            parts.get("cpu family").cloned(),
                            parts.get("model").cloned(),
                            parts.get("model name").cloned(),
                            parts.get("cpu cores").and_then(|f| u8::from_str(f).ok()),
                            parts.get("cpu MHz").and_then(|f| f32::from_str(f).ok()),
                        );
                        cpuinfo_data.push(cpuinfo);
                        parts.clear();
                    } else {
                        let parts_data: Vec<&str> = line.split(':').collect();
                        if parts_data.len() == 2 {
                            parts.insert(parts_data[0].trim().to_string(), parts_data[1].trim().to_string());
                        } 
                    }                                                                
                }
            },
            Err(err) => {
                error!("Error reading cpuinfo: {:?}", err);
                return Err(CommonLibError::new("Error reading cpuinfo"));
            }
        }
        Ok(cpuinfo_data)
    }

    /**
     * Get a line from the cpuinfo file.
     * 
     * `line`: The line to get.
     * 
     * Returns the line or an error.
     * 
     */
    fn get_line(line: Result<String, std::io::Error>) -> Result<String, CommonLibError> {
        match line {
            Ok(line) => {
                Ok(line)
            },
            Err(err) => {
                Err(CommonLibError::new(format!("Error reading line: {err:?}").as_str()))
            }
        }
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_current() {
        let binding = ProcsCpuinfo::get_cpuinfo();
        assert!(binding.is_ok());
    }

    #[test]
    fn test_read_predefined_cpuinfo() {
        let binding = ProcsCpuinfo::read_cpuinfo("resources/test/test_cpuinfo").unwrap();
        let first = binding.first().unwrap();
        assert_eq!(&first.apicid.unwrap(), &0);
        assert_eq!(&first.vendor_id.clone().unwrap(), "AuthenticAMD");
        assert_eq!(&first.cpu_family.clone().unwrap(), "25");
        assert_eq!(&first.model.clone().unwrap(), "116");
        assert_eq!(&first.model_name.clone().unwrap(), "AMD Ryzen 7 7840HS w/ Radeon 780M Graphics");
        assert_eq!(&first.cpu_cores.unwrap(), &8);
        assert_eq!(&first.cpu_mhz.unwrap(), &3404.518);
    }

}