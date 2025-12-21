use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUsername {
    pub username: String,
}
