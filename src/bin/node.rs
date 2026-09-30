use clap::Parser;
use myrix_chain::config::Config;

#[derive(Parser)]
#[command(name="MYRIX Node")]
struct Args { #[arg(long, default_value="node-1")] id:String, #[arg(long, default_value="0.0.0.0:9001")] listen_addr:String }

#[tokio::main]
async fn main()->anyhow::Result<()>{
    tracing_subscriber::fmt().init();
    let args=Args::parse();
    let config=Config::from_env();
    tracing::info!("Starting MYRIX full-node scaffold {} on {}",args.id,args.listen_addr);
    tracing::info!("Network: {} / chain_id: {}",config.network_name,config.chain_id);
    anyhow::bail!("P2P node engine is not implemented yet; refusing to pretend this process is a live full node")
}
