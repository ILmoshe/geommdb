use log::{error, info};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;

use crate::persistence::{Persistence, WalEntry};
use crate::storage::GeoDatabase;

enum Command {
    GeoAdd { key: String, lat: f64, lon: f64 },
    GeoSearch { lat: f64, lon: f64, radius: f64 },
}

fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["GEOADD", key, lat, lon] => Some(Command::GeoAdd {
            key: key.to_string(),
            lat: lat.parse().ok()?,
            lon: lon.parse().ok()?,
        }),
        ["GEOSEARCH", lat, lon, radius] => Some(Command::GeoSearch {
            lat: lat.parse().ok()?,
            lon: lon.parse().ok()?,
            radius: radius.parse().ok()?,
        }),
        _ => None,
    }
}

async fn handle_client(
    mut stream: TcpStream,
    db: Arc<Mutex<GeoDatabase>>,
    persistence: Arc<Mutex<Persistence>>,
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
                Command::GeoAdd { key, lat, lon } => {
                    let mut db = db.lock().unwrap();
                    db.geo_add(key.clone(), lat, lon);
                    let mut persistence = persistence.lock().unwrap();
                    if let Err(e) = persistence.log_entry(WalEntry::GeoAdd {
                        key: key.clone(),
                        lat,
                        lon,
                    }) {
                        error!("Failed to log entry; err = {:?}", e);
                    }
                    info!(
                        "GeoAdd command processed: key={}, lat={}, lon={}",
                        key, lat, lon
                    );
                    "OK\n".to_string()
                }
                Command::GeoSearch { lat, lon, radius } => {
                    let db = db.lock().unwrap();
                    let results = db.geo_search(lat, lon, radius);
                    info!(
                        "GeoSearch command processed: lat={}, lon={}, radius={}",
                        lat, lon, radius
                    );
                    results.join("\n") + "\n"
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

pub async fn start_server() {
    info!("Starting server on 127.0.0.1:6379...");
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // Check if snapshot file exists before trying to load it
    let snapshot_path = Path::new("snapshot.bincode");
    let db = if snapshot_path.exists() {
        match Persistence::load_snapshot() {
            Ok(db) => {
                info!("Loaded database from snapshot.");
                db
            }
            Err(e) => {
                error!(
                    "Failed to load snapshot, starting with a new database. Error: {:?}",
                    e
                );
                GeoDatabase::new()
            }
        }
    } else {
        info!("No snapshot file found, starting with a new database.");
        GeoDatabase::new()
    };

    // Load WAL to recover any missed entries
    let db = Arc::new(Mutex::new(db));
    if Persistence::load_wal(&mut db.lock().unwrap()).is_ok() {
        info!("Loaded write-ahead log (WAL).");
    } else {
        error!("Failed to load write-ahead log (WAL).");
    }

    let persistence = Arc::new(Mutex::new(Persistence::new().unwrap()));

    info!("Server is ready to accept connections.");
    let db_clone = Arc::clone(&db);
    let persistence_clone = Arc::clone(&persistence);
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let db = Arc::clone(&db_clone);
            let persistence = Arc::clone(&persistence_clone);
            tokio::spawn(async move {
                handle_client(socket, db, persistence).await;
            });
        }
    });

    // Wait for a signal to shut down
    signal::ctrl_c().await.expect("Failed to listen for event");

    // Create a snapshot on shutdown
    info!("Creating snapshot before shutdown...");
    Persistence::create_snapshot(&db.lock().unwrap()).unwrap();
    info!("Snapshot created, shutting down.");
}
