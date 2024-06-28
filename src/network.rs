use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use tokio;

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

async fn handle_client(mut stream: TcpStream, db: Arc<Mutex<GeoDatabase>>) {
    println!("Started the handler!");
    let mut buffer = [0; 1024];

    loop {
        let n = stream.read(&mut buffer).await.unwrap();
        if n == 0 {
            break;
        }

        let input = String::from_utf8_lossy(&buffer[..n]);
        let response = if let Some(command) = parse_command(&input) {
            match command {
                Command::GeoAdd { key, lat, lon } => {
                    let mut db = db.lock().unwrap();
                    db.geo_add(key, lat, lon);
                    "OK\n".to_string()
                }
                Command::GeoSearch { lat, lon, radius } => {
                    let db = db.lock().unwrap();
                    let results = db.geo_search(lat, lon, radius);
                    results.join("\n") + "\n"
                }
            }
        } else {
            "ERROR\n".to_string()
        };

        stream.write_all(response.as_bytes()).await.unwrap();
    }
    println!("Here in handler");
}

pub async fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Arc::new(Mutex::new(GeoDatabase::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = Arc::clone(&db);
        tokio::spawn(async move {
            handle_client(socket, db).await;
        });
    }
}
