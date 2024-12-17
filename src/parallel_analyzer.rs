use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use alloy::{
    consensus::Transaction,
    eips::BlockNumberOrTag,
    network::primitives::BlockTransactionsKind,
    primitives::{Address, TxHash, B256},
    providers::{ext::DebugApi, Provider, ProviderBuilder, RootProvider},
    rpc::types::{
        trace::geth::{AccountState, GethDebugTracingOptions, PreStateConfig},
        Transaction as AlloyTransaction,
    },
    transports::http::{Client, Http},
};
use eyre::Result;
use reqwest::Url;
use tracing::{debug, error, info};

use crate::db::{
    block::{Block, BlockDB},
    parallel_analyzer_state::ParallelAnalyzerStateDB,
    transaction::{Transaction as DbTransaction, TransactionDB},
    transaction_dag::{TransactionDag, TransactionDagDB},
    DB,
};

#[derive(Debug, Clone)]
pub struct ParallelAnalyzer {
    pub db: Arc<DB>,
    pub execution_api_client: Arc<RootProvider<Http<Client>>>,
    pub start_block: i64,
    pub chain_id: i64,
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
    pub fn new(db: Arc<DB>, execution_api: Url, start_block: i64, chain_id: i64) -> Self {
        let provider = ProviderBuilder::new().on_http(execution_api);
        Self {
            db,
            execution_api_client: Arc::new(provider),
            start_block,
            chain_id,
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
            block_number: full_block.header.number as i64,
            gas_used: full_block.header.gas_used as i64,
            gas_limit: full_block.header.gas_limit as i64,
            block_timestamp: full_block.header.timestamp as i64,
            base_fee_per_gas: full_block.header.base_fee_per_gas.unwrap_or_default() as i64,
            blob_gas_used: full_block.header.blob_gas_used.unwrap_or_default() as i64,
            excess_blob_gas: full_block.header.excess_blob_gas.unwrap_or_default() as i64,
            created_at: None,
            updated_at: None,
        };
        self.db.insert_block(&data).await?;
        let transactions = full_block.transactions.as_transactions().unwrap().to_vec();
        for tx in transactions.clone() {
            let data = DbTransaction {
                block_number: tx.block_number.unwrap() as i64,
                tx_index: tx.transaction_index.unwrap() as i64,
                tx_hash: tx.inner.tx_hash().to_string(),
                tx_from: tx.from.to_string(),
                tx_to: tx.to().unwrap_or_default().to_string(),
                gas_price: tx.gas_price().unwrap_or_default().to_string(),
                max_fee_per_gas: tx.max_fee_per_gas().to_string(),
                max_priority_fee_per_gas: tx
                    .max_priority_fee_per_gas()
                    .unwrap_or_default()
                    .to_string(),
                max_fee_per_blob_gas: tx.max_fee_per_blob_gas().unwrap_or_default().to_string(),
                gas: tx.gas_limit() as i64,
                tx_value: tx.value().to_string(),
                input: tx.input().to_string(),
                nonce: tx.nonce() as i64,
                tx_type: tx.inner.tx_type() as i16,
                created_at: None,
                updated_at: None,
            };
            self.db.insert_transaction(&data).await?;
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
        debug!(
            "tx_hash: {:?}, Read set: {:?}",
            tx_hash, read_set.storage_set
        );
        let write_set = account_state_to_set(write_state);
        debug!(
            "tx_hash: {:?}, Write set: {:?}",
            tx_hash, write_set.storage_set
        );

        Ok(TransactionStateSet {
            read_set,
            write_set,
        })
    }

    pub async fn analyse_block(&self, block_number: i64, latest_block_number: i64) -> Result<()> {
        let transactions = self.get_block_transactions(block_number as u64).await?;
        let mut tx_states = BTreeMap::new();
        for tx in transactions {
            let tx_hash = tx.inner.tx_hash();
            let tx_index = tx.transaction_index.unwrap();
            let state = self.trace_transaction_state(tx_hash).await?;
            tx_states.insert(tx_index, state);
        }
        self.db
            .delete_transaction_dags_by_block_number(block_number)
            .await?;
        for (tx_index, state) in tx_states.clone() {
            for index in 1..tx_index {
                let prev_state = tx_states.get(&index).unwrap();
                let mask = check_tx_dependency(prev_state, &state);
                if mask != 0 {
                    debug!(
                        "Transaction {} depends on transaction {} with mask {:x}",
                        tx_index, index, mask
                    );
                    let data = TransactionDag {
                        block_number,
                        source_tx: tx_index as i64,
                        target_tx: index as i64,
                        dep_type: mask,
                        created_at: None,
                        updated_at: None,
                    };
                    debug!("dag: {:?}", (tx_index, index));
                    self.db.insert_transaction_dag(&data).await?;
                }
            }
        }
        let mut parallel_analyzer_state = self
            .db
            .get_parallel_analyzer_state_by_chainid(self.chain_id)
            .await?
            .unwrap();
        parallel_analyzer_state.latest_analyzed_block = block_number;
        parallel_analyzer_state.latest_block = latest_block_number;
        self.db
            .update_parallel_analyzer_state_by_chainid(&parallel_analyzer_state)
            .await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let mut block_number = self.start_block;
        loop {
            let latest_block_number = self.execution_api_client.get_block_number().await? as i64;
            info!(
                "Analysing block {}, latest_block: {}",
                block_number, latest_block_number
            );
            if block_number > latest_block_number {
                tokio::time::sleep(tokio::time::Duration::from_secs(12)).await;
                continue;
            }
            match self.analyse_block(block_number, latest_block_number).await {
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

pub fn check_tx_dependency(prev_state: &TransactionStateSet, state: &TransactionStateSet) -> i16 {
    let mut mask = 0;
    // check balance dependency
    if prev_state
        .write_set
        .balance_set
        .intersection(&state.read_set.balance_set)
        .count()
        > 0
    {
        mask |= 0x1;
    }

    // check code dependency
    if prev_state
        .write_set
        .code_set
        .intersection(&state.read_set.code_set)
        .count()
        > 0
    {
        mask |= 0x10;
    }
    // check storage dependency
    if prev_state
        .write_set
        .storage_set
        .intersection(&state.read_set.storage_set)
        .count()
        > 0
    {
        mask |= 0x100;
    }
    mask
}
