use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum WebSocketMessage {
    Register { nickname: String },
    DisconnectUser(String),

    RegisterSuccess,

    GetActiveUsersList(String),
    ActiveUsersList(Vec<String>),

    ErrorDeserializingJson(String),
}
