use axum::{extract::FromRequest, response::IntoResponse};
use serde::Serialize;

use crate::errors::AppError;

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}
