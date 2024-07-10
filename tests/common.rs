use std::net::SocketAddr;
use tokio::task;

use geommdb::network::server::start_server;
use geommdb::network::replica::{Replica, Role};

pub async fn start_leader(addr: SocketAddr) {
    task::spawn(async move {
        start_server(addr, None, Role::Leader).await;
    });
}

pub async fn start_replica(addr: SocketAddr, leader_addr: SocketAddr) {
    task::spawn(async move {
        start_server(addr, Some(leader_addr), Role::Replica).await;
    });
}
