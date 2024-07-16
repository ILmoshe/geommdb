use crate::network::command::{parse_command, Command};
use crate::network::replica::{Replica, Role};
use crate::persistence::WalEntry;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_client(
    mut stream: TcpStream,
    replica: Arc<Replica>,
    leader_addr: Option<SocketAddr>,
) {
    info!("Client connected: {}", stream.peer_addr().unwrap());
    let mut buffer = [0; 1024];

    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                info!("Client disconnected: {}", stream.peer_addr().unwrap());
                break;
            }
            Ok(n) => n,
            Err(e) => {
                error!("Failed to read from socket; err = {:?}", e);
                break;
            }
        };

        let input = String::from_utf8_lossy(&buffer[..n]);
        info!("Received command: {}", input.trim());
        let response = if let Some(command) = parse_command(&input) {
            match command {
                Command::GeoAdd { key, coords } => {
                    let new_coord = coords.clone();

                    if let Role::Leader = replica.role {
                        let mut db = replica.db.lock().unwrap();
                        db.geo_add(key.clone(), coords);

                        let mut persistence = replica.persistence.lock().unwrap();
                        if let Err(e) = persistence.log_entry(WalEntry::GeoAdd {
                            key: key.clone(),
                            coords: new_coord,
                        }) {
                            error!("Failed to log entry; err = {:?}", e);
                        }
                        info!("GeoAdd command processed: key={}", key);
                        "OK\n".to_string()
                    } else {
                        // Forward write requests to the leader
                        if let Some(leader_addr) = leader_addr {
                            if let Ok(mut leader_stream) = TcpStream::connect(leader_addr).await {
                                // TODO: Do we actually need to connect to the leader in every request?, heartbeat can ensure that we are connected to the leader.
                                leader_stream.write_all(input.as_bytes()).await.unwrap();
                                let mut leader_response = [0; 1024];
                                let n = leader_stream.read(&mut leader_response).await.unwrap();
                                String::from_utf8_lossy(&leader_response[..n]).to_string()
                            } else {
                                error!("Failed to connect to leader at {}", leader_addr);
                                "ERROR\n".to_string()
                            }
                        } else {
                            error!("Leader address not provided.");
                            "ERROR\n".to_string()
                        }
                    }
                }
                Command::GeoSearch { lat, lon, radius } => {
                    let db = replica.db.lock().unwrap();
                    let results = db.geo_search(lat, lon, radius);
                    info!(
                        "GeoSearch command processed: lat={}, lon={}, radius={}",
                        lat, lon, radius
                    );
                    results.join("\n") + "\n"
                }
                Command::GeoGet { key } => {
                    let db = replica.db.lock().unwrap();
                    match db.geo_get(&key) {
                        Some(data) => {
                            info!("GeoGet command processed: key={}", key);
                            data + "\n"
                        }
                        None => {
                            info!("GeoGet command: key={} not found", key);
                            "Not Found\n".to_string()
                        }
                    }
                }
                Command::Heartbeat => {
                    if let Role::Leader = replica.role {
                        if let Ok(addr) = stream.peer_addr() {
                            replica.handle_heartbeat(addr).await;
                        }
                        "OK\n".to_string()
                    } else {
                        "ERROR\n".to_string()
                    }
                }
            }
        } else {
            error!("Invalid command received: {}", input.trim());
            "ERROR\n".to_string()
        };

        if let Err(e) = stream.write_all(response.as_bytes()).await {
            error!("Failed to write to socket; err = {:?}", e);
            break;
        }
    }
    info!(
        "Handler finished for client: {}",
        stream.peer_addr().unwrap()
    );
}
