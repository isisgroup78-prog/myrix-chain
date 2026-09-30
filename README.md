# MYRIX Chain

MYRIX Chain is a modular blockchain foundation built in Rust. This release adds the mainnet operational layer, including:
- network and peer registry
- staking and validator basics
- consensus primitives
- storage abstraction
- governance primitives
- bridge / rollup / SDK foundations
- public API and web explorer shell

## Build

```bash
cargo build
```

## Run API

```bash
cargo run --bin myrix_chain
```

## Docker

```bash
docker-compose up --build
```

## Project modules

- `core` - transactions, blocks, ledger
- `wallet` - signing and key management
- `staking` - validator and staking logic
- `consensus` - epochs and leadership
- `network` - peer discovery and p2p foundation
- `storage` - persistent chain storage
- `governance` - voting and proposals
- `bridge` - route definitions for chains
- `sdk` - client abstraction and RPC integration
- `rollup` - optimistic rollup cost foundations
- `vm` - gas billing and execution layer

## Roadmap

- validator multi-node network
- stronger consensus validation
- genesis bootstrap
- wallet and explorer production UI
- governance on-chain execution
- bridge and runtime integration
