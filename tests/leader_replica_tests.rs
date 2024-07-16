use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

mod common;

#[tokio::test]
#[cfg(test)]
async fn test_leader_replica_interaction() {
    dotenv::from_filename(".env.test").ok();

    let leader_addr = "127.0.0.1:6379".parse().unwrap();
    // let replica_addr = "127.0.0.1:6380".parse().unwrap();

    // Start leader and replica
    common::start_leader(leader_addr).await;
    sleep(Duration::from_secs(1)).await; // Allow leader to start

    // common::start_replica(replica_addr, leader_addr).await;
    // sleep(Duration::from_secs(1)).await; // Allow replica to start

    let mut leader_stream = TcpStream::connect(leader_addr).await.unwrap();
    leader_stream
        .write_all(b"GEOADD point1 40.7128 -74.0060\n")
        .await
        .unwrap();
    let mut buffer = [0; 1024];
    let n = leader_stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);
    assert_eq!(response, "OK\n");

    // let mut replica_stream = TcpStream::connect(replica_addr).await.unwrap();
    // replica_stream
    //     .write_all(b"GEOSEARCH 40.7128 -74.0060 10\n")
    //     .await
    //     .unwrap();
    // let n = replica_stream.read(&mut buffer).await.unwrap();
    // let response = String::from_utf8_lossy(&buffer[..n]);
    // assert_eq!(response, "point1\n");

    sleep(Duration::from_secs(1)).await;
}
