use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}};

use crate::common::CommonLibError;

/**
 * User struct
 * 
 * This struct represents a user in the system.
 * 
 * name: The name of the user.
 * uid: The group id.
 */
pub struct User {
    name: String,
    uid: u32,
}

impl User {

    /**
     * Create a new Group instance.
     * 
     * name: The name of the group.
     * uid: The user id.
     * 
     */
    #[must_use] pub fn new(name: String, uid: u32) -> User {
        User {
            name,
            uid,
        }
    }

    /**
     * Get the name of the user.
     * 
     * Returns users
     * 
     * # Errors
     * - Error opening user file
     * - Error reading user line
     * - Error parsing user id
     */
    pub fn get_users() -> Result<Vec<User>, CommonLibError> {
        User::read_users("/etc/passwd")
    }

    /**
     * Get user Hashmap
     * 
     * Returns user map.
     * 
     * # Errors
     * - Error opening user file
     * - Error reading user line
     * - Error parsing user id
     */
    pub fn get_users_map() -> Result<HashMap<u32, String>, CommonLibError> {
        let users = User::read_users("/etc/passwd")?;
        let mut user_map: HashMap<u32, String> = HashMap::new();
        for user in users {
            user_map.insert(user.uid, user.name);
        }
        Ok(user_map)
    }    

    /**
     * Get the users.
     * 
     * Returns users.
     * 
     * # Errors
     * - Error opening user file
     * - Error reading user line
     * - Error parsing user id
     * 
     */
    fn read_users(file: &str) -> Result<Vec<User>, CommonLibError> {        
        let mut users: Vec<User> = Vec::new();
        let file = File::open(file).map_err(|err| CommonLibError::new(&format!("Error opening user file {err:?}")))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let user_line = line.map_err(|err| CommonLibError::new(&format!("Error reading user line {err:?}")))?;
            let parts = user_line
                .split(':')
                .collect::<Vec<&str>>();
            let name = parts[0].to_string();
            let uid = parts[2].parse::<u32>().map_err(|err| CommonLibError::new(&format!("Error parsing user id {err:?}")))?;
            users.push(User::new(name, uid));
        }
        Ok(users)
    }

}

mod tests {

    #[test]
    fn test_users() {
        let users = super::User::get_users().unwrap();
        assert!(users.len() > 0);
    }

    #[test]
    fn test_users_map() {
        let users = super::User::get_users_map().unwrap();
        assert!(users.len() > 0);
    }  

    #[test]
    fn read_users_file() {
        let users = super::User::read_users("resources/test/test_passwd").unwrap();
        assert!(users.len() > 0);
    }
}   