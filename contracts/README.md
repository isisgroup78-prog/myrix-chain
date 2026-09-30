# MYRIX EVM smart contracts

This directory is the application-layer Solidity workspace for MYRIX.

## Current contracts

- **MyrixToken (MYX)** — fixed initial supply of 1 billion MYX, with holder burn support.
- **MyrixRegistry** — small owner-controlled registry for application metadata.

These contracts use OpenZeppelin Contracts rather than copying security-sensitive primitives. OpenZeppelin documents its published releases as the preferred installation path.

## Local setup

Install Foundry, then:

```bash
cd contracts
forge install OpenZeppelin/openzeppelin-contracts --no-commit
forge install foundry-rs/forge-std --no-commit
forge test
```

## Important mainnet distinction

The Solidity contracts are **not yet deployable on MYRIX mainnet** merely because they compile. The Rust chain still needs a consensus-integrated EVM execution layer, persistent EVM account/code/storage state, contract-creation and contract-call transactions, deterministic gas accounting, receipts/logs, EVM JSON-RPC compatibility, and state-root integration.

The planned execution backend is REVM, a Rust EVM implementation focused on EVM compatibility and speed. citeturn1search12turn2search2

Before deploying value-bearing contracts to mainnet, the contracts and the MYRIX EVM integration need independent security review and extensive testing. OpenZeppelin specifically recommends testing before mainnet and treating key management and audits as mainnet concerns. citeturn0search2turn0search1
