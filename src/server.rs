use std::sync::Arc;

use axum::{routing::get, Router};
use eyre::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};

use crate::{
    config::Config,
    handlers::common::{handle_404, handle_panic, health_check},
};

#[derive(Clone)]
pub struct ServerState {
    pub db: PgPool,
    pub config: Arc<Config>,
}

impl ServerState {
    pub fn new(db: PgPool, config: Config) -> Result<Self> {
        Ok(Self {
            db,
            config: Arc::new(config),
        })
    }

    fn config_router(&self, server_state: Arc<ServerState>) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .fallback(get(handle_404))
            .layer(CatchPanicLayer::custom(handle_panic))
            .layer(CorsLayer::permissive())
            .with_state(server_state.clone())
    }

    pub async fn run(&self) -> Result<()> {
        let server = self.config_router(Arc::new(self.clone()));

        let listener = TcpListener::bind(&self.config.server_addr).await?;
        axum::serve(listener, server).await?;
        Ok(())
    }
}
