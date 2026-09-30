use clap::Parser;
use ed25519_dalek::{SigningKey, VerifyingKey};
use myrix_chain::{config::Config, p2p, runtime::ChainRuntime, validator::ValidatorSet};
use std::sync::Arc;

#[derive(Parser)]
#[command(name="MYRIX Validator")]
struct Args {
    #[arg(long)]
    id: String,
    #[arg(long, default_value="0.0.0.0:9000")]
    listen_addr: String,
}

fn load_signing_key() -> anyhow::Result<SigningKey> {
    let raw = std::env::var("VALIDATOR_PRIVATE_KEY")
        .map_err(|_| anyhow::anyhow!("VALIDATOR_PRIVATE_KEY must contain a 32-byte Ed25519 seed in hex"))?;
    let bytes = hex::decode(raw.trim()).map_err(|_| anyhow::anyhow!("VALIDATOR_PRIVATE_KEY is not valid hex"))?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("VALIDATOR_PRIVATE_KEY must decode to exactly 32 bytes"))?;
    Ok(SigningKey::from_bytes(&bytes))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let args = Args::parse();
    let config = Config::from_env();
    let validators = ValidatorSet::from_genesis().map_err(anyhow::Error::msg)?;
    let info = validators.validators.get(&args.id)
        .ok_or_else(|| anyhow::anyhow!("validator ID is not present in genesis"))?;
    if !info.active || info.jailed || info.slashed {
        return Err(anyhow::anyhow!("validator is not active"));
    }

    let signing_key = load_signing_key()?;
    let public_key = hex::encode(VerifyingKey::from(&signing_key).to_bytes());
    let expected = info.public_key.strip_prefix("ed25519:").unwrap_or(&info.public_key);
    if public_key != expected {
        return Err(anyhow::anyhow!("VALIDATOR_PRIVATE_KEY does not match the public key registered in genesis"));
    }

    let peers = std::env::var("P2P_PEERS").unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| format!("./data/{}", args.id));
    let runtime = Arc::new(ChainRuntime::open(&db_path).map_err(anyhow::Error::msg)?);
    let block_time_ms = std::env::var("BLOCK_TIME_MS").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(1200);

    tracing::info!(
        validator=%args.id,
        chain=%config.chain_id,
        listen=%args.listen_addr,
        peers=peers.len(),
        stake=info.stake,
        quorum=validators.quorum(),
        "MYRIX validator starting"
    );

    p2p::run_validator(
        &args.listen_addr,
        config.node_id,
        config.chain_id,
        runtime,
        config.max_peers as usize,
        args.id,
        signing_key,
        peers,
        block_time_ms,
    ).await.map_err(anyhow::Error::msg)
}
