use std::sync::Arc;

use alloy::providers::Provider;
use axum::{
    extract::{Query, State},
    Json,
};
use eyre::eyre;

use crate::{
    db::{
        parallel_analyzer_state::ParallelAnalyzerStateDB, transaction::TransactionDB,
        transaction_dag::TransactionDagDB,
    },
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
        block_number as i64 - 10_i64
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

pub async fn handle_parallel_analyzer_state(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<i64>, AppError> {
    let analyzer_state = state
        .db
        .get_parallel_analyzer_state_by_chainid(state.chain_id)
        .await?;
    let analyzer_state =
        analyzer_state.ok_or(AppError(eyre!("parallel analyzer state not found")))?;

    Ok(Json(analyzer_state.latest_analyzed_block))
}
