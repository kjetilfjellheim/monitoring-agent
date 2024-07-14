
pub struct Monitor {
    pub status: Status,
    pub name: String,
    pub uuid: String,
}

pub enum MonitorStatus {
    Ok,
    Unknown,
    Error { message: String }
}

impl Monitor {
    pub fn new(name: &str, uuid: &str) -> Monitor {
        Monitor {
            status: Status::Unknown,
            name: name.to_string(),
            uuid: uuid.to_string(),
        }
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

 