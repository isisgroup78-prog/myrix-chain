use axum::{extract::{Path, State}, http::StatusCode, response::Json, routing::get, Router};
use std::sync::Arc;
use crate::config::Config;
use crate::metrics::MetricsCollector;

#[derive(Clone, serde::Serialize)]
pub struct ChainStatus { pub network_name:String, pub chain_id:String, pub latest_height:u64, pub peer_count:u64, pub validators:u64, pub block_time_ms:u64, pub version:String }
#[derive(Clone, serde::Serialize)]
pub struct BlockData { pub height:u64, pub hash:String, pub prev_hash:String, pub timestamp:u64, pub tx_count:u64, pub proposer:String, pub gas_used:u64 }
#[derive(Clone, serde::Serialize)]
pub struct TxData { pub hash:String, pub from:String, pub to:String, pub amount:u64, pub fee:u64, pub block_height:u64, pub status:String, pub gas_used:u64 }
#[derive(Clone, serde::Serialize)]
pub struct AccountData { pub address:String, pub balance:u64, pub nonce:u64, pub transactions:u64, pub token_balance:u64 }
#[derive(Clone, serde::Serialize)]
pub struct ValidatorData { pub id:String, pub stake:u64, pub commission:u64, pub active:bool, pub jailed:bool }

pub struct AppState { pub config:Config, pub metrics:Arc<MetricsCollector> }

pub fn create_router() -> Router {
    let config=Config::from_env();
    let metrics=Arc::new(MetricsCollector::new());
    let state=Arc::new(AppState{config,metrics});
    Router::new()
        .route("/health",get(health_check))
        .route("/status",get(chain_status))
        .route("/block/latest",get(latest_block))
        .route("/block/:height",get(block_by_height))
        .route("/tx/:hash",get(tx_by_hash))
        .route("/account/:address",get(account_by_address))
        .route("/validators",get(validators_list))
        .route("/metrics",get(prometheus_metrics))
        .with_state(state)
}
async fn health_check()->(StatusCode,Json<serde_json::Value>){(StatusCode::OK,Json(serde_json::json!({"status":"ok","network":"MYRIX Chain","timestamp":chrono::Utc::now().to_rfc3339()})))}
async fn chain_status(State(state):State<Arc<AppState>>)->(StatusCode,Json<ChainStatus>){(StatusCode::OK,Json(ChainStatus{network_name:state.config.network_name.clone(),chain_id:state.config.chain_id.clone(),latest_height:0,peer_count:0,validators:0,block_time_ms:1200,version:env!("CARGO_PKG_VERSION").to_string()}))}
async fn latest_block()->(StatusCode,Json<serde_json::Value>){(StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"no committed block available"})))}
async fn block_by_height(Path(height):Path<u64>)->(StatusCode,Json<serde_json::Value>){if height==0{(StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"genesis block not loaded"})))}else{(StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"block not found","height":height})))}} 
async fn tx_by_hash(Path(hash):Path<String>)->(StatusCode,Json<serde_json::Value>){(StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"transaction not found","hash":hash})))}
async fn account_by_address(Path(address):Path<String>)->(StatusCode,Json<serde_json::Value>){(StatusCode::NOT_FOUND,Json(serde_json::json!({"error":"account not found","address":address})))}
async fn validators_list()->(StatusCode,Json<Vec<ValidatorData>>){(StatusCode::OK,Json(Vec::new()))}
async fn prometheus_metrics(State(state):State<Arc<AppState>>)->(StatusCode,String){match state.metrics.render(){Ok(v)=>(StatusCode::OK,v),Err(e)=>(StatusCode::INTERNAL_SERVER_ERROR,e)}}
