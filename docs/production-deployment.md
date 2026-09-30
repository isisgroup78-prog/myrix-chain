# MYRIX Chain Production Deployment Guide

## Architecture

- API nodes
- Validator nodes
- Full nodes
- Monitoring stack
- Public explorer
- Database for indexing

## Deployment flow

1. Provision infrastructure
2. Deploy monitoring services
3. Deploy validators and full nodes
4. Validate health endpoints
5. Enable public explorer and RPC
6. Publish status and validator set

## Production recommendations

- Dedicated machines for validators
- 10Gbps+ connectivity
- SSD-backed storage
- TLS termination at load balancer
- Key backup and encrypted storage
- regular snapshotting

## Security

- firewall restrictions
- SSH key-only access
- automatic updates
- encrypted secrets
- incident response process
