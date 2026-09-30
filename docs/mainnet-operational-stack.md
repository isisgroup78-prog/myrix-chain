# MYRIX Chain operational stack

This document describes the implemented node stack on the hardening branch and the remaining production-hardening items.

## 1. Core chain services — implemented
- persistent RocksDB ledger state
- deterministic genesis allocation bootstrap
- Ed25519 transaction signatures and sender/public-key binding
- nonce and balance validation
- atomic block application
- persistent mempool, proposals, votes and commit certificates
- REST API for health, status, blocks, transactions, accounts and validators
- Prometheus metrics endpoint

## 2. Consensus and validator network — implemented baseline
- PoS validator registry loaded from canonical genesis
- deterministic stake-weighted leader selection
- signed block proposals
- signed votes bound to chain ID, height, round and block hash
- one-vote-per-validator-per-round protection
- stake-weighted 2/3 quorum
- commit-certificate verification
- validator process with proposal, voting, finalization and retry rounds
- block catch-up from peers using certificate-verified bundles

The current implementation is a compact BFT baseline. It is not a formal proof of Byzantine safety/liveness under every network fault model.

## 3. P2P and RPC — implemented baseline
- bounded framed TCP transport
- chain/protocol handshake
- transaction submission
- proposal and vote propagation
- certificate propagation
- block-range synchronization
- REST RPC surface

Production hardening still recommended:
- authenticated/encrypted transport such as Noise/libp2p or an equivalent secure channel
- peer discovery and persistent peer scoring
- connection/IP rate limiting
- replay protection and message IDs
- WebSocket subscriptions
- JSON-RPC compatibility if required by clients

## 4. Validator deployment
Production compose now includes three validator services with separate persistent volumes.

Each validator must receive a real Ed25519 private seed through:
- `VALIDATOR_01_PRIVATE_KEY`
- `VALIDATOR_02_PRIVATE_KEY`
- `VALIDATOR_03_PRIVATE_KEY`

The corresponding public keys must be registered in `genesis/genesis.json` and `genesis/validators.json` before launch.

Generate a key with:
`cargo run --bin keygen`

Never commit validator private keys.

## 5. Security and operations
Implemented:
- no hard-coded validator private key
- required production Grafana password
- bounded P2P frame size and peer concurrency
- persistent state
- fail-closed validator key/public-key matching

Still required before an irreversible public mainnet launch:
- independent security audit
- fault-injection and multi-node integration tests
- key custody/HSM or equivalent operational key protection
- snapshot/restore procedures
- alerting and SLOs
- upgrade/migration policy
- chain halt/recovery runbook
- genesis ceremony and independently verified validator keys

## 6. Launch checklist
1. Generate validator keys.
2. Verify each public key against the validator private seed.
3. Replace genesis placeholders with the real public keys.
4. Verify genesis allocations and validator stakes.
5. Build all binaries.
6. Run the multi-validator integration test suite.
7. Test restart and peer catch-up.
8. Test invalid signatures, conflicting votes and invalid proposals.
9. Back up validator configuration and establish key custody.
10. Only then perform the genesis launch.


## 7. Smart-contract layer

A Foundry/Solidity workspace now exists under `contracts/` with OpenZeppelin-based example contracts and automated Solidity tests.

The chain is **not yet EVM-mainnet ready**. Before contracts can execute on MYRIX itself, the node needs:

- deterministic EVM transaction types and signing/replay protection
- persistent EVM account, contract-code and storage state
- REVM execution integrated into block application
- contract creation and message-call semantics
- deterministic gas schedule and block gas limits
- transaction receipts, logs and bloom/indexing strategy
- EVM-compatible JSON-RPC methods such as `eth_chainId`, `eth_getBalance`, `eth_call`, `eth_sendRawTransaction`, `eth_getTransactionReceipt`
- EVM address/account compatibility with the existing MYRIX account model
- state-root commitment and deterministic state transition tests
- Solidity integration tests against a MYRIX devnet
- explorer support for contract addresses, events and internal calls

REVM is the planned Rust execution backend. Its current documentation exposes the EVM database interface and execution builder APIs, which can be integrated once the MYRIX state model is adapted to EVM account/code/storage semantics. citeturn2search1turn2search2

## 8. Mainnet gates

The following are launch gates, not optional polish:

1. CI, security audit and Solidity tests green.
2. Three-or-more independent validator environments running the same genesis.
3. Real validator key ceremony with independently verified public keys.
4. Authenticated/encrypted P2P transport and peer identity.
5. Byzantine/fault-injection tests covering conflicting proposals, equivocation, delayed messages and validator restarts.
6. Snapshot/restore and disaster-recovery drill.
7. Deterministic state-root/replay tests.
8. Smart-contract execution audit before enabling value-bearing contracts.
9. RPC rate limits, authentication for privileged endpoints, and abuse protection.
10. Public testnet soak period with monitoring, alerting and incident runbooks.
11. Independent review of consensus, cryptography, storage, P2P and contract execution.
12. Only after those gates: freeze genesis, publish binaries/checksums, launch validators and enable public RPC.

OpenZeppelin's mainnet guidance emphasizes testing, independent security review, source verification and secure key management; audits reduce risk but do not guarantee absence of vulnerabilities. citeturn0search1turn0search2
