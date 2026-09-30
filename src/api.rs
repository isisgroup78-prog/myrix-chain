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
        .route("/rpc",post(json_rpc))
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
    state.metrics.sync(s.height, s.tx_count, s.validator_count);
    (StatusCode::OK,Json(ChainStatus{
        network_name:state.config.network_name.clone(),
        chain_id:state.config.chain_id.clone(),
        latest_height:s.height,
        peer_count:0,
        validators:s.validator_count,
        block_time_ms:1200,
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

async fn validators_list(State(state):State<Arc<AppState>>)->(StatusCode,Json<serde_json::Value>){
    match serde_json::to_value(&*state.runtime.validators) {
        Ok(v)=>(StatusCode::OK,Json(v)),
        Err(e)=>(StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e.to_string()})))
    }
}

async fn prometheus_metrics(State(state):State<Arc<AppState>>)->(StatusCode,String){
    let s = state.runtime.status();
    state.metrics.sync(s.height, s.tx_count, s.validator_count);
    match state.metrics.render(){Ok(v)=>(StatusCode::OK,v),Err(e)=>(StatusCode::INTERNAL_SERVER_ERROR,e)}
}


fn parse_evm_address(value: &str) -> Result<revm::primitives::Address, String> {
    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 40 { return Err("EVM address must contain 20 bytes".to_string()); }
    let bytes = hex::decode(raw).map_err(|_| "invalid EVM address".to_string())?;
    revm::primitives::Address::try_from(bytes.as_slice()).map_err(|_| "invalid EVM address".to_string())
}

fn parse_evm_u256(value: &str) -> Result<revm::primitives::U256, String> {
    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.is_empty() { return Ok(revm::primitives::U256::ZERO); }
    revm::primitives::U256::from_str_radix(raw, 16).map_err(|_| "invalid EVM quantity".to_string())
}

fn execute_json_rpc(state: &AppState, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    match method {
        "status" => serde_json::to_value(state.runtime.status()).map_err(|e| e.to_string()),
        "getBalance" => {
            let address = params.get("address").and_then(|v| v.as_str()).ok_or("address is required".to_string())?;
            let account = state.runtime.account(address)?;
            Ok(serde_json::json!({
                "address": address,
                "balance": account.as_ref().map(|a| a.balance).unwrap_or(0),
                "nonce": account.as_ref().map(|a| a.nonce).unwrap_or(0)
            }))
        }
        "getBlock" => {
            let height = if params.get("height").and_then(|v| v.as_str()) == Some("latest") || params.get("height").is_none() {
                state.runtime.status().height
            } else {
                params.get("height").and_then(|v| v.as_u64()).ok_or("invalid height".to_string())?
            };
            state.runtime.block(height)?
                .map(serde_json::to_value).transpose().map_err(|e| e.to_string())?
                .ok_or_else(|| "block not found".to_string())
        }
        "getTransaction" => {
            let hash = params.get("hash").and_then(|v| v.as_str()).ok_or("hash is required".to_string())?;
            state.runtime.transaction(hash)?
                .map(serde_json::to_value).transpose().map_err(|e| e.to_string())?
                .ok_or_else(|| "transaction not found".to_string())
        }
        "getValidators" => serde_json::to_value(&*state.runtime.validators).map_err(|e| e.to_string()),
        "eth_chainId" => {
            let chain_id = std::env::var("EVM_CHAIN_ID").map_err(|_| "EVM_CHAIN_ID is not configured".to_string())?;
            let value = chain_id.parse::<u64>().map_err(|_| "EVM_CHAIN_ID must be a decimal u64".to_string())?;
            Ok(serde_json::Value::String(format!("0x{value:x}")))
        }
        "eth_blockNumber" => Ok(serde_json::Value::String(format!("0x{:x}", state.runtime.status().height))),
        "eth_getBalance" => {
            let address = params.get(0).and_then(|v| v.as_str()).or_else(|| params.get("address").and_then(|v| v.as_str())).ok_or("address is required".to_string())?;
            let address = parse_evm_address(address)?;
            let balance = state.runtime.evm.account(address)?.map(|a| revm::primitives::U256::from_be_bytes(a.balance));
            Ok(serde_json::Value::String(format!("0x{:x}", balance.unwrap_or(revm::primitives::U256::ZERO))))
        }
        "eth_getCode" => {
            let address = params.get(0).and_then(|v| v.as_str()).or_else(|| params.get("address").and_then(|v| v.as_str())).ok_or("address is required".to_string())?;
            let address = parse_evm_address(address)?;
            let code_hash = state.runtime.evm.account(address)?.map(|a| revm::primitives::B256::from(a.code_hash));
            let code = code_hash.and_then(|hash| state.runtime.evm.code(hash)).unwrap_or_default();
            Ok(serde_json::Value::String(format!("0x{}", hex::encode(code))))
        }
        "eth_getStorageAt" => {
            let address = params.get(0).and_then(|v| v.as_str()).or_else(|| params.get("address").and_then(|v| v.as_str())).ok_or("address is required".to_string())?;
            let slot = params.get(1).and_then(|v| v.as_str()).or_else(|| params.get("slot").and_then(|v| v.as_str())).ok_or("slot is required".to_string())?;
            let address = parse_evm_address(address)?;
            let slot = parse_evm_u256(slot)?;
            let value = state.runtime.evm.storage(address, slot)?;
            Ok(serde_json::Value::String(format!("0x{}", hex::encode(value.to_be_bytes::<32>()))))
        }
        "myrix_evmStateCommitment" => Ok(serde_json::Value::String(format!("0x{}", hex::encode(state.runtime.evm.commitment())))),
        "sendTransaction" => {
            let raw = params.get("transaction").cloned().ok_or("transaction is required".to_string())?;
            let tx: crate::core::Transaction = serde_json::from_value(raw).map_err(|e| e.to_string())?;
            state.runtime.submit_transaction(&tx).map(|hash| serde_json::json!({
                "accepted": true, "hash": hash
            }))
        }
        _ => Err("method not found".to_string()),
    }
}

async fn json_rpc(
    State(state): State<Arc<AppState>>,
    AxumJson(request): AxumJson<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);

    match execute_json_rpc(&state, method, params) {
        Ok(value) => (StatusCode::OK, Json(serde_json::json!({
            "jsonrpc": "2.0", "id": id, "result": value
        }))),
        Err(error) => (StatusCode::OK, Json(serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": {"code": -32000, "message": error}
        }))),
    }
}
