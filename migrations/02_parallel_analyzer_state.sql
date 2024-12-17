CREATE TABLE IF NOT EXISTS parallel_analyzer_state (
    latest_block BIGINT NOT NULL,
    chain_id BIGINT NOT NULL PRIMARY KEY,
    start_block BIGINT NOT NULL,
    latest_analyzed_block BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
