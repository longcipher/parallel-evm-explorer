use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::DB;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct TransactionDag {
    pub block_number: i64,
    pub source: i64,
    pub target: i64,
    /// 0x1: balance, 0x10: code, 0x100: storage
    pub dep_type: i16,
}

pub trait TransactionDagDB {
    async fn insert_transaction_dag(
        &self,
        transaction_dag: &TransactionDag,
    ) -> Result<(), sqlx::Error>;
    async fn get_transaction_dags_by_block_number(
        &self,
        block_number: i64,
    ) -> Result<Vec<TransactionDag>, sqlx::Error>;
}

impl TransactionDagDB for DB {
    async fn insert_transaction_dag(
        &self,
        transaction_dag: &TransactionDag,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO transaction_dags (block_number, source, target, dep_type)
            VALUES (?,?,?,?)
            "#,
        )
        .bind(transaction_dag.block_number)
        .bind(transaction_dag.source)
        .bind(transaction_dag.target)
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
            SELECT * FROM transaction_dags WHERE block_number = ?
            "#,
        )
        .bind(block_number)
        .fetch_all(&self.db)
        .await?;
        Ok(transaction_dags)
    }
}
