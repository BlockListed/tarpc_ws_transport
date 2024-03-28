use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt};
use tarpc::client::Config;

#[tokio::main]
async fn main() {
    let ws = tokio_tungstenite::connect_async("ws://localhost:3000/tarpc").await.unwrap().0;
    let transport = tarpc_ws_transport::ClientTransport::new(ws);

    let mut config = Config::default();
    config.max_in_flight_requests = 2_000;
    config.pending_request_buffer = 200;
    let client = example_service::HelloWorldClient::new(tarpc::client::Config::default(), transport).spawn();

    let name = "Aaron".to_string();

    let futures = (0..200_000).map(|_| {
        client.hello(tarpc::context::current(), name.clone()).map(|_| ())
    }).collect::<FuturesUnordered<_>>();

    futures.collect::<()>().await;
}
