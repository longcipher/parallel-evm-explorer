use std::sync::Arc;

use alloy::{
    network::Ethereum,
    providers::{Provider, RootProvider},
};
use axum::{Router, routing::get};
use eyre::Result;
use tokio::net::TcpListener;
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};

use crate::{
    config::Config,
    db::DB,
    handlers::{
        common::{handle_404, handle_panic, health_check},
        transaction_dag::{handle_parallel_analyzer_state, handle_transaction_dag},
    },
};

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<DB>,
    pub config: Arc<Config>,
    pub execution_api_client: Arc<RootProvider<Ethereum>>,
    pub chain_id: i64,
}

impl ServerState {
    pub fn new(db: Arc<DB>, config: Config) -> Result<Self> {
        let provider = RootProvider::builder().on_http(config.execution_api.clone());
        Ok(Self {
            db,
            config: Arc::new(config.clone()),
            execution_api_client: Arc::new(provider),
            chain_id: config.chain_id,
        })
    }

    fn config_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/data/evm/transaction-dag", get(handle_transaction_dag))
            .route(
                "/data/evm/parallel-analyzer-state",
                get(handle_parallel_analyzer_state),
            )
            .fallback(get(handle_404))
            .layer(CatchPanicLayer::custom(handle_panic))
            .layer(CorsLayer::permissive())
            .with_state(Arc::new(self.clone()))
    }

    pub async fn run(&self) -> Result<()> {
        let server = self.config_router();

        let listener = TcpListener::bind(&self.config.server_addr).await?;
        axum::serve(listener, server).await?;
        Ok(())
    }
}
