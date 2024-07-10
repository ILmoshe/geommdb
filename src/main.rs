mod commands;
mod geospatial;
mod network;
mod persistence;
mod storage;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Creating geomemdb D:");
    network::start_server().await;
}
