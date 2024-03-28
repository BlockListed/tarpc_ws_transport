use axum::extract::WebSocketUpgrade;
use axum::routing::get;
use example_service::HelloWorld;
use futures_util::StreamExt;
use tarpc::server::{BaseChannel, Channel, Config};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Server;

impl HelloWorld for Server {
    async fn hello(self, _ctx: tarpc::context::Context, name: String) -> String {
        let res = format!("Hello, {name}!");
        res
    }
}

#[tokio::main]
async fn main() {
    let app = axum::Router::<()>::new().route(
        "/tarpc",
        get(|ws: WebSocketUpgrade| async {
            ws.on_upgrade(|ws| async {
                let transport = tarpc_ws_transport::ServerTransport::new(ws);
                let mut config = Config::default();
                config.pending_response_buffer = 200;
                let channel = BaseChannel::new(config, transport);

                channel
                    .execute(Server.serve())
                    .for_each(|f| async move {
                        tokio::spawn(f);
                    })
                    .await;
            })
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
