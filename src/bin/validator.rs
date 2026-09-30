use clap::Parser;
use myrix_chain::{config::Config, validator::{ValidatorInfo, ValidatorSet}};

#[derive(Parser)]
#[command(name="MYRIX Validator")]
struct Args { #[arg(long, default_value="validator-node-1")] id:String, #[arg(long)] stake:u64, #[arg(long, default_value="0.0.0.0:9000")] listen_addr:String }

#[tokio::main]
async fn main()->anyhow::Result<()>{
    tracing_subscriber::fmt().init();
    let args=Args::parse();
    let config=Config::from_env();
    if args.stake<1_000_000{return Err(anyhow::anyhow!("minimum stake is 1,000,000 tokens"));}
    let validator=ValidatorInfo::new(args.id.clone(),config.validator_key.clone().unwrap_or_default(),args.stake);
    if validator.public_key.is_empty(){return Err(anyhow::anyhow!("VALIDATOR_KEY must contain a real Ed25519 public key")); }
    let mut set=ValidatorSet::new();
    set.add_validator(validator)?;
    tracing::info!("Validator scaffold {} on {}",args.id,args.listen_addr);
    tracing::info!("Stake: {} | quorum: {} | network: {}",args.stake,set.quorum(),config.chain_id);
    anyhow::bail!("block production/voting engine is not implemented yet; refusing to pretend this process is a live validator")
}
