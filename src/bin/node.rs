use clap::Parser;
use std::sync::Arc;
use myrix_chain::{config::Config, p2p, runtime::ChainRuntime};

#[derive(Parser)]
#[command(name="MYRIX Node")]
struct Args {
    #[arg(long, default_value="node-1")] id:String,
    #[arg(long, default_value="0.0.0.0:9001")] listen_addr:String,
}

#[tokio::main]
async fn main()->anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let args=Args::parse();
    let config=Config::from_env();
    let db_path=std::env::var("DB_PATH").unwrap_or_else(|_| format!("./data/{}", args.id));
    let runtime=Arc::new(ChainRuntime::open(&db_path).map_err(anyhow::Error::msg)?);
    tracing::info!("MYRIX node {} on {} / chain {}",args.id,args.listen_addr,config.chain_id);
    p2p::run_server(&args.listen_addr,args.id,config.chain_id,runtime,config.max_peers as usize)
        .await.map_err(anyhow::Error::msg)
}
