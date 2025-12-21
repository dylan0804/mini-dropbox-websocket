use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{any, post},
    Json, Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use dashmap::{DashMap, Map};
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::net::TcpListener;

mod payloads;
pub mod response;

#[derive(Clone)]
struct AppState {
    users_list: DashMap<String, String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            users_list: DashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(AppState::new());

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("server running at {:?}", listener.local_addr());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("{addr} connected");

    ws.on_upgrade(move |socket| handle_socket(socket, state, addr))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(write(sender));
    tokio::spawn(read(receiver));
    // if socket
    //     .send(axum::extract::ws::Message::Ping(Bytes::new()))
    //     .await
    //     .is_ok()
    // {
    //     state.users_list.insert(key, value)
    //     println!("Pinged {who}")
    // } else {
    //     println!("Could not ping {who}");
    //     return;
    // }
}

async fn write(sender: SplitSink<WebSocket, Message>) {}

async fn read(receiver: SplitStream<WebSocket>) {}
