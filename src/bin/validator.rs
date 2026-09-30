use clap::Parser;
use myrix_chain::config::Config;
use myrix_chain::validator::{ValidatorInfo, ValidatorSet};

#[derive(Parser)]
#[command(name = "MYRIX Validator")]
struct Args {
    #[arg(long, default_value = "validator-node-1")]
    id: String,

    #[arg(long)]
    stake: u64,

    #[arg(long, default_value = "0.0.0.0:9000")]
    listen_addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let config = Config::from_env();

    if args.stake < 1_000_000 {
        return Err(anyhow::anyhow!("minimum stake is 1,000,000 tokens"));
    }

    let validator = ValidatorInfo::new(
        args.id.clone(),
        format!("0x{}", hex::encode(args.id.as_bytes())),
        args.stake,
    );

    let mut validator_set = ValidatorSet::new();
    validator_set.add_validator(validator)?;

    tracing::info!("Starting validator {} on {}", args.id, args.listen_addr);
    tracing::info!("Stake: {} | quorum: {} | network: {}", args.stake, validator_set.get_quorum(), config.chain_id);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        if let Some(leader) = validator_set.get_leader(0) {
            tracing::info!("Current leader: {}", leader);
        }
    }
}
