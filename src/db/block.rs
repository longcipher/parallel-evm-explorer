use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, FromRow};

use super::DB;

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct Block {
    pub parent_hash: String,
    pub block_hash: String,
    pub block_number: i64,
    pub gas_used: i64,
    pub gas_limit: i64,
    pub block_timestamp: i64,
    pub base_fee_per_gas: i64,
    pub blob_gas_used: i64,
    pub excess_blob_gas: i64,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[allow(unused)]
pub trait BlockDB {
    async fn insert_block(&self, block: &Block) -> Result<(), sqlx::Error>;
    async fn get_block_by_number(&self, block_number: i64) -> Result<Option<Block>, sqlx::Error>;
    async fn get_block_by_hash(&self, block_hash: &str) -> Result<Option<Block>, sqlx::Error>;
}

impl BlockDB for DB {
    async fn insert_block(&self, block: &Block) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO blocks (parent_hash, block_hash, block_number, gas_used, gas_limit, block_timestamp, base_fee_per_gas, blob_gas_used, excess_blob_gas)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (block_hash) DO NOTHING
            "#,
          )
          .bind(block.parent_hash.clone())
          .bind(block.block_hash.clone())
          .bind(block.block_number)
          .bind(block.gas_used)
          .bind(block.gas_limit)
          .bind(block.block_timestamp)
          .bind(block.base_fee_per_gas)
          .bind(block.blob_gas_used)
          .bind(block.excess_blob_gas)
          .execute(&self.db)
          .await?;
        Ok(())
    }
    async fn get_block_by_number(&self, block_number: i64) -> Result<Option<Block>, sqlx::Error> {
        let block = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks WHERE block_number = $1
            "#,
        )
        .bind(block_number)
        .fetch_optional(&self.db)
        .await?;
        Ok(block)
    }
    async fn get_block_by_hash(&self, block_hash: &str) -> Result<Option<Block>, sqlx::Error> {
        let block = sqlx::query_as::<_, Block>(
            r#"
            SELECT * FROM blocks WHERE block_hash = $1
            "#,
        )
        .bind(block_hash)
        .fetch_optional(&self.db)
        .await?;
        Ok(block)
    }
}
