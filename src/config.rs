use std::path::PathBuf;

use clap::Parser;
use config::{Config as FileConfig, ConfigError, Environment, File};
use reqwest::Url;
use serde_derive::Deserialize;

#[derive(Clone, Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    #[clap(short, long, default_value = "false")]
    pub version: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub execution_api: Url,
    pub start_block: i64,
    pub chain_id: i64,
    pub server_addr: String,
    pub database_url: String,
}

impl Config {
    pub fn new(config: Option<PathBuf>) -> Result<Self, ConfigError> {
        let c = FileConfig::builder()
            .add_source(File::from(config.expect("Config file not found")))
            .add_source(Environment::with_prefix("PEVM"))
            .build()?;
        c.try_deserialize()
    }
}
