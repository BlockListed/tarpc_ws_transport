use std::time::Duration;

use example_service::HelloWorldClient;
use tarpc::{client::{Config, RpcError}, context::current};

async fn handle_error(err: RpcError, client: &mut HelloWorldClient) {
    if let RpcError::Shutdown = err {
        *client = retry_get_client(8).await.unwrap();
    }

    println!("error: {err}");
}

#[tokio::main]
async fn main() {
    let mut client = retry_get_client(10).await.unwrap();

    for _ in 0..200 {
        match client.hello(current(), "Aaron".to_string()).await {
            Ok(r) => println!("ran client rpc: {r}"),
            Err(e) => handle_error(e, &mut client).await,
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

async fn retry_get_client(mut attempts: usize) -> std::io::Result<HelloWorldClient> {
    if attempts == 0 {
        attempts = usize::MAX;
    }
    
    let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(100).max_delay(Duration::from_secs(10)).map(tokio_retry::strategy::jitter).take(attempts);

    tokio_retry::Retry::spawn(strategy, get_client).await
}

async fn get_client() -> std::io::Result<HelloWorldClient> {
    let ws = tokio_tungstenite::connect_async("ws://localhost:3000/tarpc").await.map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))?.0;
    let transport = tarpc_ws_transport::ClientTransport::new(ws);

    let mut config = Config::default();
    config.max_in_flight_requests = 2_000;
    config.pending_request_buffer = 200;

    Ok(example_service::HelloWorldClient::new(config, transport).spawn())
}
