use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Serialize)]
pub struct ChainStatus {
    pub network_name: String,
    pub chain_id: String,
    pub latest_height: u64,
    pub peer_count: u64,
    pub validators: u64,
    pub block_time_ms: u64,
}

#[derive(Clone, Serialize)]
pub struct BlockData {
    pub height: u64,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: u64,
    pub tx_count: u64,
    pub proposer: String,
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
}

#[derive(Clone, Serialize)]
pub struct AccountData {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transactions: u64,
    pub token_balance: u64,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub network_name: String,
    pub chain_id: String,
    pub latest_height: u64,
}

pub fn create_router() -> Router {
    let state = Arc::new(AppState {
        network_name: "MYRIX Chain".to_string(),
        chain_id: "myrix-mainnet-1".to_string(),
        latest_height: 128742,
    });

    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(chain_status))
        .route("/block/latest", get(latest_block))
        .route("/block/:height", get(block_by_height))
        .route("/tx/:hash", get(tx_by_hash))
        .route("/account/:address", get(account_by_address))
        .with_state(state)
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "status": "ok",
            "network": "MYRIX Chain"
        })),
    )
}

async fn chain_status(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ChainStatus>) {
    let status = ChainStatus {
        network_name: state.network_name.clone(),
        chain_id: state.chain_id.clone(),
        latest_height: state.latest_height,
        peer_count: 128,
        validators: 21,
        block_time_ms: 1200,
    };

    (StatusCode::OK, axum::Json(status))
}

async fn latest_block(State(state): State<Arc<AppState>>) -> (StatusCode, Json<BlockData>) {
    let block = BlockData {
        height: state.latest_height,
        hash: "0x7f4d9a01d2f3b0fd7c7a21b7c1f3ab9d3fe7e77d".to_string(),
        prev_hash: "0x7a1d83ea3b9ad6c6ec9d242be79d5d0f8ee7d9fb".to_string(),
        timestamp: 1727770000,
        tx_count: 14,
        proposer: "validator-07".to_string(),
    };

    (StatusCode::OK, axum::Json(block))
}

async fn block_by_height(Path(height): Path<u64>) -> (StatusCode, Json<BlockData>) {
    let block = BlockData {
        height,
        hash: format!("0x{:x}", height * 17),
        prev_hash: format!("0x{:x}", height * 17 - 1),
        timestamp: 1727770000 + height,
        tx_count: 9,
        proposer: "validator-09".to_string(),
    };

    (StatusCode::OK, axum::Json(block))
}

async fn tx_by_hash(Path(hash): Path<String>) -> (StatusCode, Json<TxData>) {
    let tx = TxData {
        hash: hash.clone(),
        from: "0xabc123...".to_string(),
        to: "0xdef456...".to_string(),
        amount: 250_000,
        fee: 500,
        block_height: 128742,
        status: "confirmed".to_string(),
    };

    (StatusCode::OK, axum::Json(tx))
}

async fn account_by_address(Path(address): Path<String>) -> (StatusCode, Json<AccountData>) {
    let account = AccountData {
        address: address.clone(),
        balance: 123_450_000,
        nonce: 25,
        transactions: 117,
        token_balance: 5000,
    };

    (StatusCode::OK, axum::Json(account))
}
