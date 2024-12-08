use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Default, FromRow, Deserialize, Serialize)]
pub struct Transaction {
    pub block_number: u64,
    pub index: u64,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub gas_price: u128,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub max_fee_per_blob_gas: u128,
    pub gas: u64,
    pub value: String,
    pub input: String,
    pub nonce: u64,
    pub tx_type: u8,
}
