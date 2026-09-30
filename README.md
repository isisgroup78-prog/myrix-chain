# MYRIX Chain

MYRIX Chain is a modular blockchain foundation designed for a future-ready production network. This release focuses on the operational layer needed for mainnet readiness: validator setup, network monitoring, deployment guidance, and launch preparation.

## Included
- Rust API backend for blockchain status and metadata
- validator node bootstrap scaffold
- node runtime scaffold
- monitoring configuration
- Docker production setup
- production deployment and launch runbooks

## Build

```bash
cargo build --release
```

## Run the API

```bash
cargo run --bin myrix_chain
```

## Run validator node

```bash
cargo run --bin validator -- --id validator-1 --stake 1000000
```

## Run full node

```bash
cargo run --bin node -- --id node-1
```

## Docker production stack

```bash
docker-compose -f docker-compose.prod.yml up --build
```

## Project direction

- validator-based network
- production RPC layer
- public explorer
- monitoring and observability
- mainnet deployment preparation

## Engineering status

The current hardening branch is an engineering baseline, not a claim of mainnet readiness or a benchmark against another network. Performance work includes parallel signature verification, deterministic weighted leader selection, persistent state, signed quorum certificates, P2P transport, RPC, monitoring, deployment automation, and a production explorer. Mainnet launch still requires passing CI, multi-node fault testing, a real genesis key ceremony, independent security review, and distributed validator infrastructure.
