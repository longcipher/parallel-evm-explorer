use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, FromRow};
use tracing::debug;

use super::DB;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct TransactionDag {
    pub block_number: i64,
    pub source_tx: i64,
    pub target_tx: i64,
    /// 0x1: balance, 0x10: code, 0x100: storage
    pub dep_type: i16,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[allow(unused)]
pub trait TransactionDagDB {
    async fn insert_transaction_dag(
        &self,
        transaction_dag: &TransactionDag,
    ) -> Result<(), sqlx::Error>;
    async fn get_transaction_dags_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<Vec<TransactionDag>, sqlx::Error>;
    async fn delete_transaction_dags_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<(), sqlx::Error>;
}

impl TransactionDagDB for DB {
    async fn insert_transaction_dag(
        &self,
        transaction_dag: &TransactionDag,
    ) -> Result<(), sqlx::Error> {
        debug!("insert transaction_dag {:?}", transaction_dag);
        sqlx::query(
            r#"
            INSERT INTO transaction_dags (block_number, source_tx, target_tx, dep_type)
            VALUES ($1,$2,$3,$4)
            "#,
        )
        .bind(transaction_dag.block_number)
        .bind(transaction_dag.source_tx)
        .bind(transaction_dag.target_tx)
        .bind(transaction_dag.dep_type)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_transaction_dags_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<Vec<TransactionDag>, sqlx::Error> {
        let transaction_dags = sqlx::query_as::<_, TransactionDag>(
            r#"
            SELECT * FROM transaction_dags WHERE block_number = $1
            "#,
        )
        .bind(block_number)
        .fetch_all(&self.db)
        .await?;
        Ok(transaction_dags)
    }

    async fn delete_transaction_dags_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM transaction_dags WHERE block_number = $1
            "#,
        )
        .bind(block_number)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
