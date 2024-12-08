use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Default, FromRow, Deserialize, Serialize)]
pub struct TransactionDag {
    pub block_number: u64,
    pub source: u64,
    pub target: u64,
    /// 0x1: balance, 0x10: code, 0x100: storage
    pub dep_type: u16,
}
