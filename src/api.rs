use axum::{extract::{Path, State}, http::StatusCode, response::Json, routing::{get, post}, Json as AxumJson, Router};
use std::sync::Arc;
use crate::{config::Config, metrics::MetricsCollector, runtime::ChainRuntime};

#[derive(Clone, serde::Serialize)]
pub struct ChainStatus { pub network_name:String, pub chain_id:String, pub latest_height:u64, pub peer_count:u64, pub validators:u64, pub block_time_ms:u64, pub version:String }

pub struct AppState { pub config:Config, pub metrics:Arc<MetricsCollector>, pub runtime:Arc<ChainRuntime> }

pub fn create_router() -> anyhow::Result<Router> {
    let config=Config::from_env();
    let db_path=std::env::var("DB_PATH").unwrap_or_else(|_| "./data/myrix".to_string());
    let runtime=Arc::new(ChainRuntime::open(&db_path).map_err(anyhow::Error::msg)?);
    let metrics=Arc::new(MetricsCollector::new());
    let state=Arc::new(AppState{config,metrics,runtime});
    Ok(Router::new()
        .route("/health",get(health_check))
        .route("/status",get(chain_status))
        .route("/block/latest",get(latest_block))
        .route("/block/:height",get(block_by_height))
        .route("/tx",post(submit_transaction))
        .route("/tx/:hash",get(tx_by_hash))
        .route("/account/:address",get(account_by_address))
        .route("/validators",get(validators_list))
        .route("/metrics",get(prometheus_metrics))
        .with_state(state))
}

async fn health_check()->(StatusCode,Json<serde_json::Value>){
    (StatusCode::OK,Json(serde_json::json!({"status":"ok","network":"MYRIX Chain","timestamp":chrono::Utc::now().to_rfc3339()})))
}

async fn chain_status(State(state):State<Arc<AppState>>)->(StatusCode,Json<ChainStatus>){
    let s=state.runtime.status();
    (StatusCode::OK,Json(ChainStatus{
        network_name:state.config.network_name.clone(), chain_id:state.config.chain_id.clone(),
        latest_height:s.height, peer_count:0, validators:0, block_time_ms:1200,
        version:env!("CARGO_PKG_VERSION").to_string()
    }))
}

async fn latest_block(State(state):State<Arc<AppState>>)->(StatusCode,Json<serde_json::Value>){
    let height=state.runtime.status().height;
    match state.runtime.block(height) {
        Ok(Some(block)) => (StatusCode::OK,Json(serde_json::to_value(block).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"no committed block available"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e})))
    }
}

async fn block_by_height(Path(height):Path<u64>,State(state):State<Arc<AppState>>)->(StatusCode,Json<serde_json::Value>){
    match state.runtime.block(height) {
        Ok(Some(block)) => (StatusCode::OK,Json(serde_json::to_value(block).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"block not found","height":height}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e})))
    }
}

async fn submit_transaction(State(state):State<Arc<AppState>>, AxumJson(tx):AxumJson<crate::core::Transaction>)->(StatusCode,Json<serde_json::Value>){
    match state.runtime.submit_transaction(&tx) {
        Ok(hash)=>(StatusCode::ACCEPTED,Json(serde_json::json!({"status":"accepted","hash":hash}))),
        Err(e)=>(StatusCode::BAD_REQUEST,Json(serde_json::json!({"error":e})))
    }
}

async fn tx_by_hash(Path(hash):Path<String>,State(state):State<Arc<AppState>>)->(StatusCode,Json<serde_json::Value>){
    match state.runtime.transaction(&hash) {
        Ok(Some(tx)) => (StatusCode::OK,Json(serde_json::to_value(tx).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"transaction not found","hash":hash}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e})))
    }
}

async fn account_by_address(Path(address):Path<String>,State(state):State<Arc<AppState>>)->(StatusCode,Json<serde_json::Value>){
    match state.runtime.account(&address) {
        Ok(Some(account)) => (StatusCode::OK,Json(serde_json::to_value(account).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"account not found","address":address}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e})))
    }
}

async fn validators_list()->(StatusCode,Json<serde_json::Value>){
    let raw=include_str!("../genesis/validators.json");
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(v)=>(StatusCode::OK,Json(v)),
        Err(e)=>(StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e.to_string()})))
    }
}

async fn prometheus_metrics(State(state):State<Arc<AppState>>)->(StatusCode,String){
    match state.metrics.render(){Ok(v)=>(StatusCode::OK,v),Err(e)=>(StatusCode::INTERNAL_SERVER_ERROR,e)}
}
