use std::{any::Any, sync::Arc};

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
};
use tracing::error;

use crate::{
    models::common::{AppError, HealthResp},
    server::ServerState,
};

pub fn handle_panic(err: Box<dyn Any + Send + 'static>) -> AxumResponse {
    let detail = if let Some(s) = err.downcast_ref::<String>() {
        s.as_str()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s
    } else {
        "no error details"
    };
    error!("Internal Server Error: {:}", detail);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal Server Error: {detail:}"),
    )
        .into_response()
}

pub async fn handle_404() -> AxumResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource could not be found.",
    )
        .into_response()
}

pub async fn health_check(
    State(_data): State<Arc<ServerState>>,
) -> Result<Json<HealthResp>, AppError> {
    Ok(Json(HealthResp {}))
}
