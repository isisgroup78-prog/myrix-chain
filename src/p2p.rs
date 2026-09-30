use crate::core::Transaction;
use crate::runtime::ChainRuntime;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::Semaphore};

const MAX_FRAME: usize = 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello { node_id: String, chain_id: String },
    Ping,
    Pong,
    SubmitTransaction(Transaction),
    Ack { ok: bool, error: Option<String> },
}

async fn read_message(stream: &mut TcpStream) -> Result<Message, String> {
    let len = stream.read_u32().await.map_err(|e| e.to_string())? as usize;
    if len == 0 || len > MAX_FRAME { return Err("invalid frame size".to_string()); }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
    serde_json::from_slice(&buf).map_err(|e| e.to_string())
}

async fn write_message(stream: &mut TcpStream, message: &Message) -> Result<(), String> {
    let bytes = serde_json::to_vec(message).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_FRAME { return Err("message too large".to_string()); }
    stream.write_u32(bytes.len() as u32).await.map_err(|e| e.to_string())?;
    stream.write_all(&bytes).await.map_err(|e| e.to_string())
}

async fn handle_connection(
    mut stream: TcpStream,
    expected_chain: String,
    runtime: Arc<ChainRuntime>,
) -> Result<(), String> {
    match read_message(&mut stream).await? {
        Message::Hello { chain_id, .. } if chain_id == expected_chain => {}
        _ => return Err("invalid or incompatible handshake".to_string()),
    }
    write_message(&mut stream, &Message::Pong).await?;

    loop {
        match read_message(&mut stream).await {
            Ok(Message::Ping) => write_message(&mut stream, &Message::Pong).await?,
            Ok(Message::SubmitTransaction(tx)) => {
                let response = match runtime.submit_transaction(&tx) {
                    Ok(_) => Message::Ack { ok: true, error: None },
                    Err(e) => Message::Ack { ok: false, error: Some(e) },
                };
                write_message(&mut stream, &response).await?;
            }
            Ok(Message::Hello { .. }) | Ok(Message::Pong) | Ok(Message::Ack { .. }) => {}
            Err(_) => break,
        }
    }
    Ok(())
}

pub async fn run_server(
    listen_addr: &str,
    node_id: String,
    chain_id: String,
    runtime: Arc<ChainRuntime>,
    max_peers: usize,
) -> Result<(), String> {
    let listener = TcpListener::bind(listen_addr).await.map_err(|e| e.to_string())?;
    let slots = Arc::new(Semaphore::new(max_peers.max(1)));
    tracing::info!(%listen_addr, %node_id, "MYRIX P2P transport listening");

    loop {
        let (mut stream, peer) = listener.accept().await.map_err(|e| e.to_string())?;
        let permit = slots.clone().acquire_owned().await.map_err(|e| e.to_string())?;
        let chain = chain_id.clone();
        let runtime = runtime.clone();
        let node = node_id.clone();
        tokio::spawn(async move {
            let result = async {
                let _ = &node;
                handle_connection(stream, chain, runtime).await
            }.await;
            if let Err(e) = result { tracing::debug!(%peer, error=%e, "P2P connection closed"); }
            drop(permit);
        });
    }
}

pub async fn connect_and_submit(
    address: &str,
    node_id: String,
    chain_id: String,
    tx: Transaction,
) -> Result<(), String> {
    let mut stream = TcpStream::connect(address).await.map_err(|e| e.to_string())?;
    write_message(&mut stream, &Message::Hello { node_id, chain_id }).await?;
    match read_message(&mut stream).await? {
        Message::Pong => {}
        _ => return Err("handshake rejected".to_string()),
    }
    write_message(&mut stream, &Message::SubmitTransaction(tx)).await?;
    match read_message(&mut stream).await? {
        Message::Ack { ok: true, .. } => Ok(()),
        Message::Ack { error: Some(e), .. } => Err(e),
        _ => Err("unexpected P2P response".to_string()),
    }
}
