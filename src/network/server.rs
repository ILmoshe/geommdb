use crate::network::handler::handle_client;
use crate::network::replica::{Replica, Role};
use crate::persistence::Persistence;
use log::info;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;

pub async fn start_server(addr: SocketAddr, leader_addr: Option<SocketAddr>, role: Role) {
    info!("Starting server on {}...", addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    let replica = Arc::new(Replica::new(addr, role.clone(), leader_addr).await);

    if let Role::Replica = role {
        let replica_clone = Arc::clone(&replica);
        tokio::spawn(async move {
            replica_clone.send_heartbeat().await;
        });
    } else if let Role::Leader = role {
        let replica_clone = Arc::clone(&replica);
        tokio::spawn(async move {
            replica_clone.monitor_replicas().await;
        });
    }

    info!("Server is ready to accept connections.");
    let replica_clone = Arc::clone(&replica);
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let replica = Arc::clone(&replica_clone);
            let leader_addr = leader_addr.clone();
            tokio::spawn(async move {
                handle_client(socket, replica, leader_addr).await;
            });
        }
    });

    // Wait for a signal to shut down
    signal::ctrl_c().await.expect("Failed to listen for event");

    // Create a snapshot on shutdown
    info!("Creating snapshot before shutdown...");
    Persistence::create_snapshot(&replica.db.lock().unwrap()).unwrap();
    info!("Snapshot created, shutting down.");
}
