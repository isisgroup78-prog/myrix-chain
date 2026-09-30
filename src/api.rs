use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::config::Config;
use crate::metrics::MetricsCollector;

#[derive(Clone, Serialize)]
pub struct ChainStatus {
    pub network_name: String,
    pub chain_id: String,
    pub latest_height: u64,
    pub peer_count: u64,
    pub validators: u64,
    pub block_time_ms: u64,
    pub version: String,
}

#[derive(Clone, Serialize)]
pub struct BlockData {
    pub height: u64,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: u64,
    pub tx_count: u64,
    pub proposer: String,
    pub gas_used: u64,
}

#[derive(Clone, Serialize)]
pub struct TxData {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub block_height: u64,
    pub status: String,
    pub gas_used: u64,
}

#[derive(Clone, Serialize)]
pub struct AccountData {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transactions: u64,
    pub token_balance: u64,
}

#[derive(Clone, Serialize)]
pub struct ValidatorData {
    pub id: String,
    pub stake: u64,
    pub commission: u64,
    pub active: bool,
    pub jailed: bool,
}

pub struct AppState {
    pub config: Config,
    pub metrics: Arc<MetricsCollector>,
}

pub fn create_router() -> Router {
    let config = Config::from_env();
    let metrics = Arc::new(MetricsCollector::new());
    let state = Arc::new(AppState { config, metrics });

    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(chain_status))
        .route("/block/latest", get(latest_block))
        .route("/block/:height", get(block_by_height))
        .route("/tx/:hash", get(tx_by_hash))
        .route("/account/:address", get(account_by_address))
        .route("/validators", get(validators_list))
        .route("/metrics", get(prometheus_metrics))
        .with_state(state)
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "status": "ok",
            "network": "MYRIX Chain",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
}

async fn chain_status(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ChainStatus>) {
    let status = ChainStatus {
        network_name: state.config.network_name.clone(),
        chain_id: state.config.chain_id.clone(),
        latest_height: 512891,
        peer_count: 128,
        validators: 21,
        block_time_ms: 1200,
        version: "1.0.0".to_string(),
    };
    (StatusCode::OK, axum::Json(status))
}

async fn latest_block(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<BlockData>) {
    let block = BlockData {
        height: 512891,
        hash: "0x7f4d9a01d2f3b0fd7c7a21b7c1f3ab9d3fe7e77d".to_string(),
        prev_hash: "0x7a1d83ea3b9ad6c6ec9d242be79d5d0f8ee7d9fb".to_string(),
        timestamp: 1727770000,
        tx_count: 147,
        proposer: "validator-07".to_string(),
        gas_used: 8_500_000,
    };
    (StatusCode::OK, axum::Json(block))
}

async fn block_by_height(Path(height): Path<u64>) -> (StatusCode, Json<BlockData>) {
    let block = BlockData {
        height,
        hash: format!("0x{:x}", height * 17),
        prev_hash: format!("0x{:x}", (height - 1) * 17),
        timestamp: 1727770000 + height,
        tx_count: 89,
        proposer: "validator-09".to_string(),
        gas_used: 7_200_000,
    };
    (StatusCode::OK, axum::Json(block))
}

async fn tx_by_hash(Path(hash): Path<String>) -> (StatusCode, Json<TxData>) {
    let tx = TxData {
        hash: hash.clone(),
        from: "0xabc123def456".to_string(),
        to: "0x789def012345".to_string(),
        amount: 250_000,
        fee: 500,
        block_height: 512891,
        status: "confirmed".to_string(),
        gas_used: 21_000,
    };
    (StatusCode::OK, axum::Json(tx))
}

async fn account_by_address(Path(address): Path<String>) -> (StatusCode, Json<AccountData>) {
    let account = AccountData {
        address: address.clone(),
        balance: 123_450_000,
        nonce: 87,
        transactions: 342,
        token_balance: 5000,
    };
    (StatusCode::OK, axum::Json(account))
}

async fn validators_list(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<Vec<ValidatorData>>) {
    let validators = vec![
        ValidatorData {
            id: "validator-01".to_string(),
            stake: 500_000_000,
            commission: 50,
            active: true,
            jailed: false,
        },
        ValidatorData {
            id: "validator-02".to_string(),
            stake: 450_000_000,
            commission: 50,
            active: true,
            jailed: false,
        },
        ValidatorData {
            id: "validator-03".to_string(),
            stake: 400_000_000,
            commission: 60,
            active: true,
            jailed: false,
        },
    ];
    (StatusCode::OK, axum::Json(validators))
}

async fn prometheus_metrics() -> (StatusCode, String) {
    (StatusCode::OK, "# MYRIX Chain Prometheus Metrics\n".to_string())
}
