use clap::Parser;
use myrix_chain::config::Config;

#[derive(Parser)]
#[command(name = "MYRIX Node")]
struct Args {
    #[arg(long, default_value = "node-1")]
    id: String,

    #[arg(long, default_value = "0.0.0.0:9001")]
    listen_addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let config = Config::from_env();

    tracing::info!("Starting full node {} on {}", args.id, args.listen_addr);
    tracing::info!("Network: {}", config.chain_id);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
