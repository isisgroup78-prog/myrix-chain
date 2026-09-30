use crate::{
    api::create_router,
    bridge::CrossChainBridge,
    consensus::Consensus,
    governance::Governance,
    network::Network,
    sdk::SdkClient,
    staking::StakePool,
    storage::RocksDbStore,
    vm::GasMeter,
};

#[tokio::main]
async fn main() {
    let app = create_router();

    let mut stake_pool = StakePool::new();
    stake_pool.add_validator(crate::staking::Validator::new("validator-1".to_string(), 30_000));
    stake_pool.add_validator(crate::staking::Validator::new("validator-2".to_string(), 25_000));
    stake_pool.add_validator(crate::staking::Validator::new("validator-3".to_string(), 20_000));

    let mut network = Network::new();
    network.add_peer("peer-1".to_string(), "/ip4/127.0.0.1/tcp/9000".to_string());
    network.add_peer("peer-2".to_string(), "/ip4/127.0.0.1/tcp/9001".to_string());

    let mut governance = Governance::new();
    governance.add_vote("proposal-1".to_string(), "voter-1".to_string(), 2_000, "yes".to_string());

    let mut bridge = CrossChainBridge::new();
    bridge.add_route("ethereum".to_string(), "0xabc".to_string());

    let mut gas = GasMeter::new(100_000);
    gas.charge(1500).unwrap();

    let sdk = SdkClient::new("http://localhost:3000".to_string());

    let consensus = Consensus::new();
    println!("Consensus leader: {}", consensus.leader);
    println!("Stake leader: {:?}", stake_pool.leader_for_round(0));
    println!("Peer count: {}", network.peers.len());
    println!("Bridge routes: {}", bridge.routes.len());
    println!("Governance votes: {}", governance.proposals["proposal-1"].len());
    println!("Gas used: {}", gas.used);
    println!("SDK ready: {}", sdk.is_ready());

    let addr = "0.0.0.0:3000".parse().unwrap();
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    )
    .await
    .unwrap();
}
