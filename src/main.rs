use std::{env, net::SocketAddr};

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, Utf8Bytes, WebSocket},
        ConnectInfo, Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{any, post},
    Json, Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use dashmap::{DashMap, Map};
use futures_util::{
    future::ok,
    io::Write,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use iroh::Endpoint;
use serde_json::json;
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::message::WebSocketMessage;

mod message;

#[derive(Clone)]
struct AppState {
    users_list: DashMap<String, Option<IrohCredentials>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            users_list: DashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct IrohCredentials {
    endpoint: Endpoint,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(AppState::new());

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "4001".to_string());
    let addr = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&addr).await.unwrap();

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

    ws.on_failed_upgrade(|e| {
        println!("error upgrading ws: {:?}", e);
    })
    .on_upgrade(move |socket| handle_socket(socket, state, addr))
}

async fn handle_socket(socket: WebSocket, state: AppState, who: SocketAddr) {
    let (sender, receiver) = socket.split();

    let (tx, rx) = mpsc::channel::<WebSocketMessage>(100);

    tokio::spawn(write(sender, rx, state.clone()));
    tokio::spawn(read(receiver, tx, state));
}

async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: Receiver<WebSocketMessage>,
    state: AppState,
) {
    println!("write called");
    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                WebSocketMessage::RegisterSuccess => {
                    sender
                        .send(Message::Text(Utf8Bytes::from(
                            json!(WebSocketMessage::RegisterSuccess).to_string(),
                        )))
                        .await
                        .ok();
                }
                WebSocketMessage::ErrorDeserializingJson(e) => {
                    sender.send(Message::Text(Utf8Bytes::from(e))).await.ok();
                }
                _ => {}
            }
        }
    }
}

async fn read(mut receiver: SplitStream<WebSocket>, tx: Sender<WebSocketMessage>, state: AppState) {
    println!("read called");
    while let Some(ok_result) = receiver.next().await {
        println!("receiving smth, deserializing it...");
        match ok_result {
            Ok(_) => {}
            Err(e) => {
                println!("err {e:?}");
            }
        }
        // match msg {
        //     Message::Text(bytes) => {
        //         match serde_json::from_str::<WebSocketMessage>(bytes.as_str()) {
        //             Ok(websocket_msg) => match websocket_msg {
        //                 WebSocketMessage::Register { nickname } => {
        //                     state.users_list.insert(nickname, None);
        //                     println!("Users now {:?}", state.users_list);
        //                     tx.send(WebSocketMessage::RegisterSuccess).await.ok();
        //                 }
        //                 WebSocketMessage::DisconnectUser(nickname) => {
        //                     state.users_list.remove(&nickname);
        //                     println!("Users now {:?}", state.users_list);
        //                 }
        //                 _ => {}
        //             },
        //             Err(e) => {
        //                 tx.send(WebSocketMessage::ErrorDeserializingJson(e.to_string()))
        //                     .await
        //                     .ok();
        //             }
        //         }
        //         // tell writer
        //     }
        //     _ => {}
        // }
    }
}
