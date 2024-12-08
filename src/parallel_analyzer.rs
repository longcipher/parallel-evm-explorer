use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use alloy::{
    consensus::Transaction,
    eips::BlockNumberOrTag,
    network::primitives::BlockTransactionsKind,
    primitives::{Address, TxHash, B256},
    providers::{ext::DebugApi, Provider, ProviderBuilder, RootProvider},
    rpc::types::{
        trace::geth::{
            AccountState, GethDebugTracingOptions, GethTrace, PreStateConfig, PreStateFrame,
        },
        Transaction as AlloyTransaction,
    },
    transports::http::{Client, Http},
};
use eyre::Result;
use reqwest::Url;
use sqlx::PgPool;
use tracing::{error, info};

use crate::db::{
    block::Block, transaction::Transaction as DbTransaction, transaction_dag::TransactionDag,
};

#[derive(Debug)]
pub struct ParallelAnalyzer {
    pub db: PgPool,
    pub execution_api_client: Arc<RootProvider<Http<Client>>>,
    pub start_block: u64,
}

#[derive(Debug, Clone)]
pub struct StateSet {
    pub balance_set: HashSet<Address>,
    pub code_set: HashSet<Address>,
    pub storage_set: HashSet<B256>,
}

#[derive(Debug, Clone)]
pub struct TransactionStateSet {
    pub read_set: StateSet,
    pub write_set: StateSet,
}

impl ParallelAnalyzer {
    pub fn new(db: PgPool, execution_api: Url, start_block: u64) -> Self {
        let provider = ProviderBuilder::new().on_http(execution_api);
        Self {
            db,
            execution_api_client: Arc::new(provider),
            start_block,
        }
    }

    pub async fn get_block_transactions(&self, block_number: u64) -> Result<Vec<AlloyTransaction>> {
        let full_block = self
            .execution_api_client
            .get_block_by_number(
                BlockNumberOrTag::Number(block_number),
                BlockTransactionsKind::Full,
            )
            .await?
            .unwrap();
        let data = Block {
            parent_hash: full_block.header.parent_hash.to_string(),
            block_hash: full_block.header.hash.to_string(),
            block_number: full_block.header.number,
            gas_used: full_block.header.gas_used,
            gas_limit: full_block.header.gas_limit,
            timestamp: full_block.header.timestamp,
            base_fee_per_gas: full_block.header.base_fee_per_gas.unwrap_or_default(),
            blob_gas_used: full_block.header.blob_gas_used.unwrap_or_default(),
            excess_blob_gas: full_block.header.excess_blob_gas.unwrap_or_default(),
        };
        sqlx::query(
      r#"
      INSERT INTO blocks (parent_hash, block_hash, block_number, gas_used, gas_limit, timestamp, base_fee_per_gas, blob_gas_used, excess_blob_gas)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(data.parent_hash)
    .bind(data.block_hash)
    .bind(data.block_number as f32)
    .bind(data.gas_used as f32)
    .bind(data.gas_limit as f32)
    .bind(data.timestamp as f32)
    .bind(data.base_fee_per_gas as f32)
    .bind(data.blob_gas_used as f32)
    .bind(data.excess_blob_gas as f32)
    .execute(&self.db)
    .await?;
        let transactions = full_block.transactions.as_transactions().unwrap().to_vec();
        for tx in transactions.clone() {
            let data = DbTransaction {
                block_number: tx.block_number.unwrap(),
                index: tx.transaction_index.unwrap(),
                hash: tx.inner.tx_hash().to_string(),
                from: tx.from.to_string(),
                to: tx.to().unwrap_or_default().to_string(),
                gas_price: tx.gas_price().unwrap_or_default(),
                max_fee_per_gas: tx.max_fee_per_gas(),
                max_priority_fee_per_gas: tx.max_priority_fee_per_gas().unwrap_or_default(),
                max_fee_per_blob_gas: tx.max_fee_per_blob_gas().unwrap_or_default(),
                gas: tx.gas_limit(),
                value: tx.value().to_string(),
                input: tx.input().to_string(),
                nonce: tx.nonce(),
                tx_type: tx.inner.tx_type() as u8,
            };
            sqlx::query(
        r#"
        INSERT INTO transactions (block_number, index, hash, from, to, gas_price, max_fee_per_gas, max_priority_fee_per_gas, max_fee_per_blob_gas, gas, value, input, nonce, tx_type)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
      )
      .bind(data.block_number as f32)
      .bind(data.index as i32)
      .bind(data.hash)
      .bind(data.from)
      .bind(data.to)
      .bind(data.gas_price.to_string())
      .bind(data.max_fee_per_gas.to_string())
      .bind(data.max_priority_fee_per_gas.to_string())
      .bind(data.max_fee_per_blob_gas.to_string())
      .bind(data.gas as f32)
      .bind(data.value.to_string())
      .bind(data.input)
      .bind(data.nonce as i32)
      .bind(data.tx_type as i16)
      .execute(&self.db)
      .await.unwrap();
        }
        Ok(transactions)
    }

    pub async fn trace_transaction_state(&self, tx_hash: &TxHash) -> Result<TransactionStateSet> {
        // fetch transaction read states
        let read_trace = self
            .execution_api_client
            .debug_trace_transaction(
                *tx_hash,
                GethDebugTracingOptions::prestate_tracer(PreStateConfig::default()),
            )
            .await?;
        // balance read
        let frame = read_trace.try_into_pre_state_frame().unwrap();
        let read_state = frame.as_default().unwrap().clone().0;
        // fetch transaction write states
        let write_trace = self
            .execution_api_client
            .debug_trace_transaction(
                *tx_hash,
                GethDebugTracingOptions::prestate_tracer(PreStateConfig {
                    diff_mode: Some(true),
                    ..Default::default()
                }),
            )
            .await?;
        let frame = write_trace.try_into_pre_state_frame().unwrap();
        let write_state = frame.as_diff().unwrap().post.clone();
        let read_set = account_state_to_set(read_state);
        let write_set = account_state_to_set(write_state);

        Ok(TransactionStateSet {
            read_set,
            write_set,
        })
    }

    pub async fn analyse_block(&self, block_number: u64) -> Result<()> {
        let transactions = self.get_block_transactions(block_number).await?;
        let mut tx_states = BTreeMap::new();
        for tx in transactions {
            let tx_hash = tx.inner.tx_hash();
            let tx_index = tx.transaction_index.unwrap();
            let state = self.trace_transaction_state(tx_hash).await?;
            tx_states.insert(tx_index, state);
        }
        for (tx_index, state) in tx_states.clone() {
            for index in 1..tx_index {
                let prev_state = tx_states.get(&index).unwrap();
                let mask = check_tx_dependency(prev_state, &state);
                if mask != 0 {
                    info!(
                        "Transaction {} depends on transaction {} with mask {:x}",
                        tx_index, index, mask
                    );
                    let data = TransactionDag {
                        block_number,
                        source: tx_index,
                        target: index,
                        dep_type: mask,
                    };
                    sqlx::query(
                        r#"
            INSERT INTO transaction_dags (block_number, source, target, dep_type)
            VALUES (?, ?, ?, ?)
            "#,
                    )
                    .bind(data.block_number as f32)
                    .bind(data.source as i32)
                    .bind(data.target as i32)
                    .bind(data.dep_type as i16)
                    .execute(&self.db)
                    .await
                    .unwrap();
                }
            }
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let mut block_number = self.start_block;
        loop {
            let latest_block_number = self.execution_api_client.get_block_number().await?;
            if block_number > latest_block_number {
                tokio::time::sleep(tokio::time::Duration::from_secs(12)).await;
                continue;
            }
            match self.analyse_block(block_number).await {
                Ok(_) => {
                    info!("Block {} analysed successfully", block_number);
                    block_number += 1;
                }
                Err(e) => {
                    error!("Error analysing block {}: {:?}", block_number, e);
                    break;
                }
            }
        }
        Ok(())
    }
}

pub fn account_state_to_set(account_state: BTreeMap<Address, AccountState>) -> StateSet {
    let mut balance_set = HashSet::new();
    let mut code_set = HashSet::new();
    let mut storage_set = HashSet::new();
    for (address, state) in account_state {
        if state.balance.is_some() {
            balance_set.insert(address);
        }
        if state.code.is_some() {
            code_set.insert(address);
        }
        for (key, _) in state.storage {
            storage_set.insert(key);
        }
    }
    StateSet {
        balance_set,
        code_set,
        storage_set,
    }
}

pub fn check_tx_dependency(prev_state: &TransactionStateSet, state: &TransactionStateSet) -> u16 {
    let mut mask = 0;
    // check balance dependency
    if prev_state
        .write_set
        .balance_set
        .intersection(&state.read_set.balance_set)
        .count()
        > 0
    {
        mask = mask | 0x1;
    }

    // check code dependency
    if prev_state
        .write_set
        .code_set
        .intersection(&state.read_set.code_set)
        .count()
        > 0
    {
        mask = mask | 0x10;
    }
    // check storage dependency
    if prev_state
        .write_set
        .storage_set
        .intersection(&state.read_set.storage_set)
        .count()
        > 0
    {
        mask = mask | 0x100;
    }
    return mask;
}
