CREATE TABLE IF NOT EXISTS blocks (
    parent_hash TEXT NOT NULL,
    block_hash TEXT NOT NULL PRIMARY KEY,
    block_number BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,
    gas_limit BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    base_fee_per_gas BIGINT NOT NULL,
    blob_gas_used BIGINT NOT NULL,
    excess_blob_gas BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX blocks_block_number_idx ON blocks (block_number);

CREATE TABLE IF NOT EXISTS transactions (
    block_number BIGINT NOT NULL,
    tx_index BIGINT NOT NULL,
    tx_hash TEXT NOT NULL PRIMARY KEY,
    tx_from TEXT NOT NULL,
    tx_to TEXT NOT NULL,
    gas_price TEXT NOT NULL,
    max_fee_per_gas TEXT NOT NULL,
    max_priority_fee_per_gas TEXT NOT NULL,
    max_fee_per_blob_gas TEXT NOT NULL,
    gas BIGINT NOT NULL,
    tx_value TEXT NOT NULL,
    input TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    tx_type SMALLINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX transactions_block_tx_idx ON transactions (block_number, tx_index);

CREATE TABLE IF NOT EXISTS transaction_dags (
    block_number BIGINT PRIMARY KEY NOT NULL,
    source_tx BIGINT NOT NULL,
    target_tx BIGINT NOT NULL,
    dep_type SMALLINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX transaction_dags_block_source_target_idx ON transaction_dags (block_number, source_tx, target_tx);
