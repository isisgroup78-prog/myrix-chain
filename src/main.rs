use myrix_chain::{api::create_router, config::Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    tracing::info!("MYRIX Chain API starting on {}", config.api_addr);
    tracing::info!("Network: {} / Chain ID: {}", config.network_name, config.chain_id);

    let app = create_router()?;
    let listener = tokio::net::TcpListener::bind(&config.api_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
