use clap::Parser;
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

    let server_state = ServerState::new(db, config)?;
    server_state.run().await?;

    Ok(())
}
