use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum WebSocketMessage {
    Register { nickname: String },
    DisconnectUser(String),
    Test(String),

    RegisterSuccess,

    ErrorDeserializingJson(String),
}
