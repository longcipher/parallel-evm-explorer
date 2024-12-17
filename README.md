# Parallel EVM Explorer

Parallel EVM explorer is a tool to analyze the parallel execution of EVM transactions in a specific block. It aims to support all EVM compatible chains.

## Parallel Scanner API Document

* <https://parallel-evm-analyzer.apifox.cn>

## DB Migration

```sh
sqlx db setup
```

## Run Server

```sh
parallel-evm-analyzer -c config.toml
```

example config.toml

```toml
execution_api = "rpc-with-debug-namespace"
start_block = 2954719 # analyzer run from this block
server_addr = "0.0.0.0:8327" # api server listen address
database_url = "postgresql://localhost:5432/pevm_explorer" # database url
chain_id = 17000 # holesky
```
