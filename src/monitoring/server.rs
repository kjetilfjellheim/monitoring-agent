use std::{collections::HashMap, net::SocketAddrV4, sync::{Arc, Mutex}};

use warp::{reply::{json, with_status}, Filter};

use crate::common::MonitorStatus;
pub struct Server {
    socket_addr: SocketAddrV4,
    status: Arc<Mutex<HashMap<String, MonitorStatus>>>
}

impl Server {
    pub fn new(socket_addr: SocketAddrV4, status: &Arc<Mutex<HashMap<String, MonitorStatus>>>) -> Server {
        Server {
            socket_addr,
            status: status.clone()
        }
    }
        
    pub async fn start(&self) {
        let socket_addr = self.socket_addr.clone();
        let status = Arc::clone(&self.status);
        tokio::spawn(async move {
            Server::start_server(&socket_addr, status).await;
        });
    }

    pub async fn start_server(socket_addr: &SocketAddrV4, status: Arc<Mutex<HashMap<String, MonitorStatus>>>) {
        let route = warp::path!("status")
            .map(move || {
                let status = status.lock();
                let response = match status {
                    Ok(status) => {
                        status.clone()                  
                    }
                    Err(_) => {
                        HashMap::new()
                    }
                };
                with_status(json(&response), warp::http::StatusCode::OK)
            });

        warp::serve(route)
            .run(socket_addr.clone())
            .await;
    }

}

    
