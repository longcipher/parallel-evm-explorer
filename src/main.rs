use std::sync::Arc;

use clap::Parser;
use db::{DB, parallel_analyzer_state::ParallelAnalyzerState};
use eyre::{Context, Result};
use server::ServerState;
use shadow_rs::shadow;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use crate::{
    config::{Cli, Config},
    db::parallel_analyzer_state::ParallelAnalyzerStateDB,
    log::init_log,
};

mod config;
mod db;
mod handlers;
mod log;
mod models;
mod parallel_analyzer;
mod server;

shadow!(build);

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.version {
        println!("{}", build::VERSION);
        return Ok(());
    }
    init_log("info")?;
    let config = Config::new(cli.config)?;
    info!("{:?}", config);

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;
    let db = Arc::new(DB::new(db));
    let server_state = ServerState::new(db.clone(), config.clone())?;
    let parallel_analyzer_state = db
        .get_parallel_analyzer_state_by_chainid(config.chain_id)
        .await?;
    let start_block = if let Some(state) = parallel_analyzer_state {
        std::cmp::max(state.latest_analyzed_block + 1, config.start_block)
    } else {
        // init parallel_analyzer_state
        db.insert_parallel_analyzer_state(&ParallelAnalyzerState {
            latest_block: 0,
            chain_id: config.chain_id,
            start_block: config.start_block,
            latest_analyzed_block: config.start_block - 1,
            created_at: None,
            updated_at: None,
        })
        .await?;
        config.start_block
    };
    let parallel_analyzer = parallel_analyzer::ParallelAnalyzer::new(
        db,
        config.execution_api.clone(),
        start_block,
        config.chain_id,
    );
    let _ = tokio::join!(server_state.run(), parallel_analyzer.run());

    Ok(())
}
