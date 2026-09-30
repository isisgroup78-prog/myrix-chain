# MYRIX Chain: Mainnet Final Deployment Guide

## Reality check

A public mainnet launch requires real infrastructure, validator keys, secure environment, and network access. This document prepares the production blueprint, but the live launch itself must happen on real cloud or bare-metal infrastructure with operational control over the node environment.

## Objective

Deploy a real MYRIX Chain mainnet with:
- validator nodes
- full nodes
- public RPC endpoint
- public explorer
- monitoring and alerting
- secure key management

## Mainnet configuration

- Chain ID: myrix-mainnet-1
- Network name: MYRIX Chain
- Consensus: BFT PoS
- Block time: 1.2 seconds
- Finality threshold: 67%
- Minimum validator stake: 1,000,000 MYRIX
- Maximum validators: 128

## Infrastructure checklist

- Cloud or bare metal environment prepared
- SSH key-based access only
- Dedicated machines for validators
- 10Gbps network or equivalent
- SSD-backed storage
- TLS termination on public endpoint
- Firewalls and DDoS protection
- Backup and restore strategy

## Validator checklist

- Validator identity generated and backed up
- Private key stored in secure vault or HSM
- Public key published to the network
- Bootnode connectivity verified
- Stake deposited and verified
- Block production enabled

## Launch sequence

1. Provision machines
2. Deploy bootstrap nodes
3. Deploy validators
4. Start genesis-based sync
5. Confirm block production
6. Start public API
7. Start explorer UI
8. Publish status and validator dashboards
9. Open RPC endpoint to public

## Operational commands

```bash
cargo run --release --bin validator -- --id validator-01 --stake 1000000 --listen-addr 0.0.0.0:9000
cargo run --release --bin node -- --id node-01 --listen-addr 0.0.0.0:9001
cargo run --release --bin myrix_chain
```

## Security hardening

- Only SSH via key-based access
- Firewall allowlist for validator ports
- TLS enabled on public endpoints
- Secrets in vault/HSM
- Snapshots stored offsite
- 24/7 on-call coverage

## Monitoring

- scrape block height
- transaction rate
- peer count
- validator status
- node outages
- CPU and memory thresholds

## Post-launch operations

- collect validator feedback
- monitor uptime and block finality
- verify public explorer accuracy
- challenge rollup or bridge issues
- publish post-launch report

## Final statement

This repo contains the production blueprint for MYRIX Chain. A true public mainnet launch still requires real infrastructure, validator keys, and controlled deployment execution on actual hardware or cloud services.
