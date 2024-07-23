use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
    sync::{Arc, Mutex},
};

use log::error;
use warp::{
    reply::{json, with_status},
    Filter,
};

/**
 * Server struct.
 *
 * This struct represents a server.
 * It is used to start the monitoring server.
 *
 */
use crate::common::{MonitorStatus, ProcsCpuinfo};

pub struct Server {
    ip: String,
    port: u16,
    status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
    cpuinfo: Option<Arc<Mutex<Vec<ProcsCpuinfo>>>>,
}

impl Server {
    pub fn new(
        ip: &String,
        port: u16,
        status: &Arc<Mutex<HashMap<String, MonitorStatus>>>,
        cpuinfo: &Option<Arc<Mutex<Vec<ProcsCpuinfo>>>>,
    ) -> Server {
        Server {
            ip: ip.to_owned(),
            port,
            status: status.clone(),
            cpuinfo: cpuinfo.clone(),
        }
    }
    /**
     * Start the server.
     */
    pub fn start(&self) {
        let ip_addr = self.ip.parse::<Ipv4Addr>();
        let socket_addr = match ip_addr {
            Ok(ip) => SocketAddrV4::new(ip, self.port),
            Err(err) => {
                error!("Error parsing IP address: {:?}. Server not started", err);
                return;
            }
        };
        let status = Arc::clone(&self.status);
        let cpuinfo = self.cpuinfo.clone();
        tokio::spawn(async move {
            Server::start_server(&socket_addr, status, &cpuinfo).await;
        });
    }

    /**
     * Start the server.
     *
     * `socket_addr`: The socket address to bind to.
     * status: The status of the monitors.
     */
    pub async fn start_server(
        socket_addr: &SocketAddrV4,
        status: Arc<Mutex<HashMap<String, MonitorStatus>>>,
        cpuinfo: &Option<Arc<Mutex<Vec<ProcsCpuinfo>>>>,
    ) {
        let cpuinfo = cpuinfo.clone();
        let route = warp::path!("status").map(move || {
            let status = status.lock();
            let response = match status {
                Ok(status) => status.clone(),
                Err(_) => HashMap::new(),
            };
            with_status(json(&response), warp::http::StatusCode::OK)
        }).or(warp::path!("cpuinfo").map(move || {            
            let response = match &cpuinfo {
                Some(cpuinfo) => {
                    let cpuinfo = cpuinfo.lock();
                    match cpuinfo {
                        Ok(cpuinfo) => cpuinfo.clone(),
                        Err(_) => Vec::new(),
                    }
                },
                None => Vec::new(),
            };
            with_status(json(&response), warp::http::StatusCode::OK)
        }));

        warp::serve(route).run(*socket_addr).await;
    }
}
