# MYRIX Chain Mainnet Launch Runbook

## Pre-launch checklist

### Infrastructure
- Cloud environment provisioned
- DDoS protection active
- TLS certificates configured
- Load balancers configured
- Backups tested
- Prometheus + Grafana configured

### Validators
- Validator identities created
- Stake configured
- Bootnodes established
- Public keys distributed
- Observer nodes ready

### Operational readiness
- Status page live
- Alerts configured
- Incident escalation paths defined
- Support channels published

## Launch sequence

1. Start bootstrap nodes
2. Start validators
3. Confirm peer connectivity
4. Validate genesis sync
5. Confirm block production
6. Launch public API endpoints
7. Publish explorer and status dashboard

## Post-launch monitoring

- Block height
- Validator activity
- Peer count
- Resource usage
- Transaction success rate

## Recovery procedure

- Restore snapshots
- Restart validators
- Rejoin peer network
- Verify state sync
- Re-open public access
