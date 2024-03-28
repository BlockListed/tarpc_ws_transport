use std::pin::Pin;
use std::{fmt::Display, marker::PhantomData};

use futures_util::{Sink, Stream};

use pin_project_lite::pin_project;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug)]
pub struct Error(String);

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

pin_project! {
    pub struct ServerTransport<Req, Resp> {
        #[pin]
        ws: axum::extract::ws::WebSocket,
        _t: PhantomData<(Req, Resp)>,
    }
}

impl<Req, Resp> ServerTransport<Req, Resp> {
    pub fn new(ws: axum::extract::ws::WebSocket) -> Self {
        Self {
            ws,
            _t: PhantomData,
        }
    }
}

impl<Req, Resp> Stream for ServerTransport<Req, Resp>
where
    Req: for<'a> Deserialize<'a>,
    Resp: Serialize,
{
    type Item = Result<Req, <Self as Sink<Resp>>::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().ws.poll_next(cx).map(|r| match r? {
            Ok(m) => match m {
                axum::extract::ws::Message::Binary(b) => Some(bincode::deserialize(&b).map_err(|e| e.to_string().into())),
                _ => Some(Err("Only binary WebSocket messages are supported!"
                    .to_string()
                    .into())),
            },
            Err(e) => Some(Err(e.to_string().into())),
        })
    }
}

impl<Req, Resp> Sink<Resp> for ServerTransport<Req, Resp>
where
    Resp: Serialize,
{
    type Error = Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project()
            .ws
            .poll_ready(cx)
            .map_err(|e| e.to_string().into())
    }

    fn start_send(self: Pin<&mut Self>, item: Resp) -> Result<(), Self::Error> {
        self.project()
            .ws
            .start_send(axum::extract::ws::Message::Binary(
                bincode::serialize(&item).map_err(|e| e.to_string())?,
            ))
            .map_err(|e| e.to_string().into())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project()
            .ws
            .poll_flush(cx)
            .map_err(|e| e.to_string().into())
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project()
            .ws
            .poll_close(cx)
            .map_err(|e| e.to_string().into())
    }
}

pin_project! {
    pub struct ClientTransport<Req, Resp, Io> {
        #[pin]
        ws: tokio_tungstenite::WebSocketStream<Io>,
        _t: PhantomData<(Req, Resp)>,
    }
}

impl<Req, Resp, Io> ClientTransport<Req, Resp, Io> {
    pub fn new(ws: tokio_tungstenite::WebSocketStream<Io>) -> Self {
        Self {
            ws,
            _t: PhantomData,
        }
    }
}

impl<Req, Resp, Io> Stream for ClientTransport<Req, Resp, Io>
where
    // required for sink implementation below to be guaranteed
    Req: Serialize,
    Resp: for<'a> Deserialize<'a>,
    Io: AsyncRead + AsyncWrite + Unpin,
{
    type Item = Result<Resp, <Self as Sink<Req>>::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().ws.poll_next(cx).map(|m| match m? {
            Ok(m) => match m {
                tokio_tungstenite::tungstenite::protocol::Message::Binary(b) => {
                    Some(bincode::deserialize(&b).map_err(|e| e.to_string().into()))
                }
                _ => Some(Err("Only binary WebSocket messages are supported!"
                    .to_string()
                    .into())),
            },
            Err(e) => Some(Err(e.to_string().into())),
        })
    }
}

impl<Req, Resp, Io> Sink<Req> for ClientTransport<Req, Resp, Io>
where
    Req: Serialize,
    Io: AsyncRead + AsyncWrite + Unpin,
{
    type Error = Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project()
            .ws
            .poll_ready(cx)
            .map_err(|e| e.to_string().into())
    }

    fn start_send(self: Pin<&mut Self>, item: Req) -> Result<(), Self::Error> {
        self.project()
            .ws
            .start_send(tokio_tungstenite::tungstenite::protocol::Message::Binary(
                bincode::serialize(&item).map_err(|e| e.to_string())?,
            ))
            .map_err(|e| e.to_string().into())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project()
            .ws
            .poll_flush(cx)
            .map_err(|e| e.to_string().into())
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project()
            .ws
            .poll_close(cx)
            .map_err(|e| e.to_string().into())
    }
}
