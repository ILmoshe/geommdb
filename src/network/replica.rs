use crate::persistence::Persistence;
use crate::storage::GeoDatabase;
use log::{error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

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
        if let Some(leader_addr) = self.leader_addr {
            loop {
                sleep(Duration::from_secs(5)).await;
                if let Ok(mut stream) = TcpStream::connect(leader_addr).await {
                    if stream.write_all(b"HEARTBEAT\n").await.is_ok() {
                        info!("Sent heartbeat to leader at {}", leader_addr);
                    }
                }
            }
        }
    }

    pub async fn monitor_replicas(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;
            let mut replicas_to_remove = Vec::new();
            {
                let replicas = self.replicas.lock().unwrap();
                let now = std::time::Instant::now();
                for (addr, last_heartbeat) in replicas.iter() {
                    if now.duration_since(*last_heartbeat).as_secs() > 10 {
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
        let mut replicas = self.replicas.lock().unwrap();
        replicas.insert(addr, std::time::Instant::now());
    }
}
