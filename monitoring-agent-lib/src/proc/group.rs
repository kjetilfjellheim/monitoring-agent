use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}};

use crate::common::CommonLibError;

/**
 * Group struct
 * 
 * This struct represents a group in the system.
 * 
 * name: The name of the group.
 * gid: The group id.
 */
pub struct Group {
    name: String,
    gid: u32,
}

impl Group {

    /**
     * Create a new Group instance.
     * 
     * name: The name of the group.
     * gid: The group id.
     * 
     */
    #[must_use] pub fn new(name: String, gid: u32) -> Group {
        Group {
            name,
            gid,
        }
    }

    /**
     * Get the name of the group.
     * 
     * Returns groups.
     * 
     * # Errors
     * - Error opening group file
     * - Error reading group line
     * - Error parsing group id
     */
    pub fn get_groups() -> Result<Vec<Group>, CommonLibError> {
        Group::read_groups("/etc/group")
    }

    /**
     * Get group Hashmap
     * 
     * Returns group map.
     * 
     * # Errors
     * - Error opening group file
     * - Error reading group line
     * - Error parsing group id
     */
    pub fn get_groups_map() -> Result<HashMap<u32, String>, CommonLibError> {
        let groups = Group::read_groups("/etc/group")?;
        let mut group_map: HashMap<u32, String> = HashMap::new();
        for group in groups {
            group_map.insert(group.gid, group.name);
        }
        Ok(group_map)
    }    

    /**
     * Get the name of the group.
     * 
     * Returns the name of the group.
     * 
     * # Errors
     * - Error opening group file
     * - Error reading group line
     * - Error parsing group id
     */
    fn read_groups(file: &str) -> Result<Vec<Group>, CommonLibError> {        
        let mut groups: Vec<Group> = Vec::new();
        let file = File::open(file).map_err(|err| CommonLibError::new(&format!("Error opening group file {err:?}")))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let group_line = line.map_err(|err| CommonLibError::new(&format!("Error reading group line {err:?}")))?;
            let parts = group_line
                .split(':')
                .collect::<Vec<&str>>();
            let name = parts[0].to_string();
            let gid = parts[2].parse::<u32>().map_err(|err| CommonLibError::new(&format!("Error parsing group id {err:?}")))?;
            groups.push(Group::new(name, gid));
        }
        Ok(groups)
    }

}

mod tests {

    #[test]
    fn test_group() {
        let groups = super::Group::get_groups().unwrap();
        assert!(groups.len() > 0);
    }

    #[test]
    fn test_group_map() {
        let groups = super::Group::get_groups_map().unwrap();
        assert!(groups.len() > 0);
    }  

    #[test]
    fn read_groups_file() {
        let groups = super::Group::read_groups("resources/test/test_group").unwrap();
        assert!(groups.len() > 0);
    }
}   