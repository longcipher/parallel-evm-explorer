use clap::Parser;
use eyre::Result;
use shadow_rs::shadow;
use tracing::info;

use crate::{
    config::{Cli, Config},
    log::init_log,
};

mod config;
mod handlers;
mod log;
mod models;
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

    let server_state =
        server::ServerState::new(config.database_url.clone().as_str(), config).await?;
    server_state.run().await?;

    Ok(())
}
