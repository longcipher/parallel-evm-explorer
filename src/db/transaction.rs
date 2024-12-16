use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, FromRow};

use super::DB;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Transaction {
    pub block_number: i64,
    pub tx_index: i64,
    pub tx_hash: String,
    pub tx_from: String,
    pub tx_to: String,
    pub gas_price: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub max_fee_per_blob_gas: String,
    pub gas: i64,
    pub tx_value: String,
    pub input: String,
    pub nonce: i64,
    pub tx_type: i16,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[allow(unused)]
pub trait TransactionDB {
    async fn insert_transaction(&self, transaction: &Transaction) -> Result<(), sqlx::Error>;
    async fn get_transaction_by_hash(&self, hash: &str)
        -> Result<Option<Transaction>, sqlx::Error>;
    async fn get_transactions_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, sqlx::Error>;
}

impl TransactionDB for DB {
    async fn insert_transaction(&self, transaction: &Transaction) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO transactions (block_number, tx_index, tx_hash, tx_from, tx_to, gas_price, max_fee_per_gas, max_priority_fee_per_gas, max_fee_per_blob_gas, gas, tx_value, input, nonce, tx_type)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            ON CONFLICT (tx_hash) DO NOTHING
            "#,
            )
            .bind(transaction.block_number)
            .bind(transaction.tx_index)
            .bind(transaction.tx_hash.clone())
            .bind(transaction.tx_from.clone())
            .bind(transaction.tx_to.clone())
            .bind(transaction.gas_price.clone())
            .bind(transaction.max_fee_per_gas.clone())
            .bind(transaction.max_priority_fee_per_gas.clone())
            .bind(transaction.max_fee_per_blob_gas.clone())
            .bind(transaction.gas)
            .bind(transaction.tx_value.clone())
            .bind(transaction.input.clone())
            .bind(transaction.nonce)
            .bind(transaction.tx_type)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn get_transaction_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions WHERE hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.db)
        .await?;
        Ok(transaction)
    }
    async fn get_transactions_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions WHERE block_number = $1
            "#,
        )
        .bind(block_number)
        .fetch_all(&self.db)
        .await?;
        Ok(transactions)
    }
}
