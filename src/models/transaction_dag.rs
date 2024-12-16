use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionDagQuery {
    pub block_number: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub index: i64,
    pub tx_hash: String,
    pub tx_type: i16,
    pub gas_used: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionDag {
    pub source: i64,
    pub target: i64,
    pub dep_type: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionDagResponse {
    pub block_number: i64,
    pub transactions: Vec<Transaction>,
    pub dags: Vec<TransactionDag>,
}
