use std::sync::Arc;

use alloy::providers::Provider;
use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    db::{transaction::TransactionDB, transaction_dag::TransactionDagDB},
    models::{
        common::AppError,
        transaction_dag::{
            Transaction, TransactionDag, TransactionDagQuery, TransactionDagResponse,
        },
    },
    server::ServerState,
};

pub async fn handle_transaction_dag(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<TransactionDagQuery>,
) -> Result<Json<TransactionDagResponse>, AppError> {
    let block_number = if let Some(block_number) = query.block_number {
        block_number
    } else {
        let block_number = state.execution_api_client.get_block_number().await?;
        block_number as i64
    };
    let transactions = state
        .db
        .get_transactions_by_block_number(block_number)
        .await?;
    let transactions: Vec<Transaction> = transactions
        .into_iter()
        .map(|t| Transaction {
            index: t.tx_index,
            tx_hash: t.tx_hash,
            tx_type: t.tx_type,
            gas_used: t.gas_price,
            from: t.tx_from,
            to: t.tx_to,
        })
        .collect();
    let transaction_dags = state
        .db
        .get_transaction_dags_by_block_number(block_number)
        .await?;
    let transaction_dags: Vec<TransactionDag> = transaction_dags
        .into_iter()
        .map(|t| TransactionDag {
            source: t.source_tx,
            target: t.target_tx,
            dep_type: t.dep_type,
        })
        .collect();
    Ok(Json(TransactionDagResponse {
        block_number,
        transactions,
        dags: transaction_dags,
    }))
}
