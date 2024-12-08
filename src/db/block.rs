use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Default, FromRow, Deserialize, Serialize)]
pub struct Block {
    pub parent_hash: String,
    pub block_hash: String,
    pub block_number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub base_fee_per_gas: u64,
    pub blob_gas_used: u64,
    pub excess_blob_gas: u64,
}
