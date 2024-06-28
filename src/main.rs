mod commands;
mod geospatial;
mod network;
mod storage;

#[tokio::main]
async fn main() {
    network::start_server().await;
}
