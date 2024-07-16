use crate::monitoring::monitoring::MonitorTrait;
use crate::monitoring::monitoring::MonitorStatus;

#[derive(Debug, Clone)]
pub struct TcpMonitor {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: MonitorStatus
}

impl TcpMonitor {
    pub fn new(host: &str, port: &u16, name: &str) -> TcpMonitor {
        TcpMonitor {
            name: name.to_string(),
            host: host.to_string(),
            port: port.clone(),
            status: MonitorStatus::Unknown
        }
    }
    
    fn close_connection(&mut self, tcp_stream: std::net::TcpStream) {
        match tcp_stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => { },
            Err(err) => {
                eprint!("Error closing connection: {:?}", err);
            }
        }
    }
}

impl MonitorTrait for TcpMonitor {

    fn check(&mut self) -> MonitorStatus {
        match std::net::TcpStream::connect(format!("{}:{}", self.host, self.port)) {
            Ok(tcp_stream) => {
                self.close_connection(tcp_stream);   
                MonitorStatus::Ok           
            },
            Err(err) => {
                MonitorStatus::Error { message: format!("Error connecting to {}:{} with error: {}", self.host, self.port, err) }
            }
        }
    }    
    
    fn get_name(&self) -> String {
        self.name.clone()
    }
}