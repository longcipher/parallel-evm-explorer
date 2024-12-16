use std::sync::Arc;

use clap::Parser;
use db::DB;
use eyre::{Context, Result};
use server::ServerState;
use shadow_rs::shadow;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use crate::{
    config::{Cli, Config},
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
    let parallel_analyzer = parallel_analyzer::ParallelAnalyzer::new(
        db,
        config.execution_api.clone(),
        config.start_block,
    );
    let _ = tokio::join!(server_state.run(), parallel_analyzer.run());

    Ok(())
}
