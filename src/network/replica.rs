use crate::persistence::Persistence;
use crate::storage::GeoDatabase;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

const DEAD_REPLICA_TIMEOUT_SECONDS: u64 = 10;

#[derive(Clone, PartialEq)]
pub enum Role {
    Leader,
    Replica,
}

pub struct Replica {
    pub addr: SocketAddr,
    pub role: Role,
    pub db: Arc<Mutex<GeoDatabase>>,
    pub persistence: Arc<Mutex<Persistence>>,
    pub leader_addr: Option<SocketAddr>,
    pub replicas: Arc<Mutex<HashMap<SocketAddr, std::time::Instant>>>, // Track replica heartbeats
}

impl Replica {
    pub async fn new(addr: SocketAddr, role: Role, leader_addr: Option<SocketAddr>) -> Self {
        let db = Arc::new(Mutex::new(GeoDatabase::new()));
        let persistence = Arc::new(Mutex::new(Persistence::new().unwrap()));
        let replicas = Arc::new(Mutex::new(HashMap::new()));

        if role == Role::Leader {
            // Load the database from snapshot, if available, otherwise create a new one
            let snapshot_path = Path::new("snapshot.bincode");
            let wal_path = Path::new("wal.log");

            if snapshot_path.exists() {
                match Persistence::load_snapshot() {
                    Ok(loaded_db) => {
                        let mut db_guard = db.lock().unwrap();
                        *db_guard = loaded_db;
                        info!("Loaded database from snapshot.");
                    }
                    Err(e) => {
                        error!(
                            "Failed to load snapshot, starting with a new database. Error: {:?}",
                            e
                        );
                    }
                }
            }

            if wal_path.exists() {
                // Load WAL to recover any missed entries
                if Persistence::load_wal(&mut db.lock().unwrap()).is_ok() {
                    info!("Loaded write-ahead log (WAL).");
                } else {
                    error!("Failed to load write-ahead log (WAL).");
                }
            }
        }

        Replica {
            addr,
            role,
            db,
            persistence,
            leader_addr,
            replicas,
        }
    }

    pub async fn send_heartbeat(&self) {
        let heartbeat_rate = env::var("HEARTBEAT_EVERY_X_SECONDS")
            .unwrap_or("5".to_string())
            .parse::<u64>()
            .unwrap();

        if let Some(leader_addr) = self.leader_addr {
            let mut stream = TcpStream::connect(leader_addr).await;

            loop {
                sleep(Duration::from_secs(heartbeat_rate)).await;

                match &mut stream {
                    Ok(ref mut stream) => {
                        if stream.write_all(b"HEARTBEAT\n").await.is_ok() {
                            info!("Sent heartbeat to leader at {}", leader_addr);
                        } else {
                            info!("Failed to send heartbeat, attempting to reconnect...");
                            *stream = TcpStream::connect(leader_addr).await.unwrap();
                        }
                    }
                    Err(_) => {
                        info!("Failed to connect to leader, retrying...");
                        stream = TcpStream::connect(leader_addr).await;
                    }
                }
            }
        }
    }

    pub async fn monitor_replicas(&self) {
        loop {
            sleep(Duration::from_secs(10)).await; // checks every 10 seconds
            let mut replicas_to_remove = Vec::new();
            {
                let replicas = self.replicas.lock().unwrap();
                let now = std::time::Instant::now();
                for (addr, last_heartbeat) in replicas.iter() {
                    if now.duration_since(*last_heartbeat).as_secs() > DEAD_REPLICA_TIMEOUT_SECONDS
                    {
                        info!("Replica at {} is considered dead", addr);
                        replicas_to_remove.push(*addr);
                    }
                }
            }
            // Remove dead replicas outside the lock
            let mut replicas = self.replicas.lock().unwrap();
            for addr in replicas_to_remove {
                replicas.remove(&addr);
            }
        }
    }

    pub async fn handle_heartbeat(&self, addr: SocketAddr) {
        info!("Heartbeat from replica at {}", addr);
        let mut replicas = self.replicas.lock().unwrap();
        replicas.insert(addr, std::time::Instant::now());
    }
}
