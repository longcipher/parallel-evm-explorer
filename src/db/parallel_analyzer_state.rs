use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::time::OffsetDateTime};

use super::DB;

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct ParallelAnalyzerState {
    pub latest_block: i64,
    pub chain_id: i64,
    pub start_block: i64,
    pub latest_analyzed_block: i64,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[allow(unused)]
pub trait ParallelAnalyzerStateDB {
    async fn insert_parallel_analyzer_state(
        &self,
        parallel_analyzer_state: &ParallelAnalyzerState,
    ) -> Result<(), sqlx::Error>;
    async fn get_parallel_analyzer_state_by_chainid(
        &self,
        chain_id: i64,
    ) -> Result<Option<ParallelAnalyzerState>, sqlx::Error>;
    async fn update_parallel_analyzer_state_by_chainid(
        &self,
        parallel_analyzer_state: &ParallelAnalyzerState,
    ) -> Result<(), sqlx::Error>;
}

impl ParallelAnalyzerStateDB for DB {
    async fn insert_parallel_analyzer_state(
        &self,
        parallel_analyzer_state: &ParallelAnalyzerState,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO parallel_analyzer_state (latest_block, chain_id, start_block, latest_analyzed_block)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (chain_id) DO NOTHING
            "#,
        )
        .bind(parallel_analyzer_state.latest_block)
        .bind(parallel_analyzer_state.chain_id)
        .bind(parallel_analyzer_state.start_block)
        .bind(parallel_analyzer_state.latest_analyzed_block)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_parallel_analyzer_state_by_chainid(
        &self,
        chain_id: i64,
    ) -> Result<Option<ParallelAnalyzerState>, sqlx::Error> {
        let parallel_analyzer_state = sqlx::query_as::<_, ParallelAnalyzerState>(
            r#"
            SELECT * FROM parallel_analyzer_state WHERE chain_id = $1
            "#,
        )
        .bind(chain_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(parallel_analyzer_state)
    }
    async fn update_parallel_analyzer_state_by_chainid(
        &self,
        parallel_analyzer_state: &ParallelAnalyzerState,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE parallel_analyzer_state
            SET latest_block = $1, start_block = $2, latest_analyzed_block = $3
            WHERE chain_id = $4
            "#,
        )
        .bind(parallel_analyzer_state.latest_block)
        .bind(parallel_analyzer_state.start_block)
        .bind(parallel_analyzer_state.latest_analyzed_block)
        .bind(parallel_analyzer_state.chain_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
