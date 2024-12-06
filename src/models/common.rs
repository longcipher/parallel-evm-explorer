use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::error;

pub struct AppError(eyre::Error);

impl<E> From<E> for AppError
where
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("internal server error: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {}", self.0),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResp {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailResponse {
    pub code: u16,
    pub message: String,
}
