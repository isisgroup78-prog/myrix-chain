# Mainnet operational stack for MYRIX Chain

This document defines the next operational layer required to move from a local prototype toward a deployable blockchain network.

## 1. Core chain services
- ledger state
- transaction validation
- block validation
- genesis block configuration
- validator set bootstrap

## 2. Consensus and validator network
- PoS validator registry
- leader election
- epoch changes
- slashing conditions
- quorum finality

## 3. P2P and RPC
- libp2p peer discovery
- gossip message propagation
- JSON-RPC access
- WebSocket real-time data
- admin dashboard integration

## 4. Explorer and UX
- blocks page
- tx detail page
- account page
- validators page
- metrics dashboard

## 5. Security and operations
- monitoring stack
- alerting service
- key management
- crash recovery
- uptime checks

## 6. Mainnet launch checklist
- validator onboarding
- initial staking pools
- node deployment automation
- chain config generation
- public release checklist
