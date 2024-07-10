mod network;
mod persistence;
mod storage;

use dotenv::dotenv;
use network::replica::Role;
use network::server::start_server;
use std::env;
use std::net::SocketAddr;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    info!("Creating geomemdb D:");

    dotenv().ok();

    let role = env::var("ROLE").unwrap_or_else(|_| "leader".to_string());
    let leader_addr: SocketAddr = env::var("LEADER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:6379".to_string())
        .parse()
        .unwrap();
    let this_addr: SocketAddr = env::var("THIS_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:6379".to_string())
        .parse()
        .unwrap();

    match role.as_str() {
        "leader" => {
            start_server(this_addr, None, Role::Leader).await;
        }
        "replica" => {
            start_server(this_addr, Some(leader_addr), Role::Replica).await;
        }
        _ => {
            eprintln!("Invalid role specified. Use 'leader' or 'replica'.");
        }
    }
}
