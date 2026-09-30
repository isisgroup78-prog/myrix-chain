use crate::{
    consensus::{CommitCertificate, Consensus, Vote},
    core::{Block, Transaction},
    runtime::ChainRuntime,
};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::Semaphore};

const MAX_FRAME: usize = 1024 * 1024;
const PROTOCOL: u16 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello { node_id: String, chain_id: String, protocol: u16 },
    Ping,
    Pong,
    SubmitTransaction(Transaction),
    Proposal(Block),
    Vote(Vote),
    Certificate(CommitCertificate),
    RequestBlocks { from: u64, to: u64 },
    Blocks(Vec<BlockBundle>),
    Ack { ok: bool, error: Option<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockBundle {
    pub block: Block,
    pub certificate: CommitCertificate,
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

async fn send(address: &str, node_id: &str, chain_id: &str, message: Message) -> Result<(), String> {
    let mut stream = TcpStream::connect(address).await.map_err(|e| e.to_string())?;
    write_message(&mut stream, &Message::Hello { node_id: node_id.to_string(), chain_id: chain_id.to_string(), protocol: PROTOCOL }).await?;
    match read_message(&mut stream).await? {
        Message::Pong => {}
        _ => return Err("handshake rejected".to_string()),
    }
    write_message(&mut stream, &message).await
}

async fn broadcast(peers: &[String], node_id: &str, chain_id: &str, message: Message) {
    for peer in peers {
        if let Err(e) = send(peer, node_id, chain_id, message.clone()).await {
            tracing::debug!(peer=%peer, error=%e, "P2P send failed");
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    expected_chain: String,
    node_id: String,
    runtime: Arc<ChainRuntime>,
    validator: Option<ValidatorContext>,
) -> Result<(), String> {
    let remote_id = match read_message(&mut stream).await? {
        Message::Hello { node_id, chain_id, protocol } if chain_id == expected_chain && protocol == PROTOCOL => node_id,
        _ => return Err("invalid or incompatible handshake".to_string()),
    };
    write_message(&mut stream, &Message::Pong).await?;
    let consensus = Consensus::new();

    loop {
        let message = match read_message(&mut stream).await {
            Ok(m) => m,
            Err(_) => break,
        };
        match message {
            Message::Ping => write_message(&mut stream, &Message::Pong).await?,
            Message::SubmitTransaction(tx) => {
                let response = match runtime.submit_transaction(&tx) {
                    Ok(_) => Message::Ack { ok: true, error: None },
                    Err(e) => Message::Ack { ok: false, error: Some(e) },
                };
                write_message(&mut stream, &response).await?;
            }
            Message::Proposal(block) => {
                runtime.validate_block(&block, &consensus)?;
                runtime.store_proposal(&block)?;
                if let Some(ctx) = &validator {
                    if ctx.id != block.proposer {
                        let vote = consensus.sign_vote(&ctx.id, &expected_chain, block.index, block.round, &block.hash, &ctx.key);
                        runtime.record_vote(&vote)?;
                        write_message(&mut stream, &Message::Vote(vote)).await?;
                    }
                }
            }
            Message::Vote(vote) => {
                if vote.chain_id != expected_chain { return Err("vote chain id mismatch".to_string()); }
                consensus.verify_vote(&vote, &runtime.validators)?;
                runtime.record_vote(&vote)?;
                let height = vote.height;
                if let Some(block) = runtime.proposal(height)? {
                    let votes = runtime.votes_for(height, vote.round, &vote.block_hash)?;
                    if let Ok(cert) = consensus.build_certificate(&block, &votes, &runtime.validators) {
                        if runtime.commit_block(&block, &cert, &consensus).is_ok() {
                            tracing::info!(height, hash=%block.hash, "block finalized");
                        }
                    }
                }
            }
            Message::Certificate(cert) => {
                if let Some(block) = runtime.proposal(cert.height)? {
                    consensus.verify_certificate(&block, &cert, &runtime.validators)?;
                    let _ = runtime.commit_block(&block, &cert, &consensus);
                }
            }
            Message::RequestBlocks { from, to } => {
                let mut bundles = Vec::new();
                for height in from..=to.min(from.saturating_add(100)) {
                    if let (Some(block), Some(certificate)) = (runtime.block(height)?, runtime.certificate(height)?) {
                        bundles.push(BlockBundle { block, certificate });
                    }
                }
                write_message(&mut stream, &Message::Blocks(bundles)).await?;
            }
            Message::Blocks(bundles) => {
                for bundle in bundles {
                    runtime.store_proposal(&bundle.block)?;
                    let _ = runtime.commit_block(&bundle.block, &bundle.certificate, &consensus);
                }
            }
            Message::Hello { .. } | Message::Pong | Message::Ack { .. } => {}
        }
    }

    tracing::debug!(peer=%remote_id, node=%node_id, "P2P connection closed");
    Ok(())
}

#[derive(Clone)]
pub struct ValidatorContext {
    pub id: String,
    pub key: SigningKey,
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
        let (stream, peer) = listener.accept().await.map_err(|e| e.to_string())?;
        let permit = slots.clone().acquire_owned().await.map_err(|e| e.to_string())?;
        let chain = chain_id.clone();
        let id = node_id.clone();
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, chain, id, runtime, None).await {
                tracing::debug!(%peer, error=%e, "P2P connection closed");
            }
            drop(permit);
        });
    }
}

pub async fn run_validator(
    listen_addr: &str,
    node_id: String,
    chain_id: String,
    runtime: Arc<ChainRuntime>,
    max_peers: usize,
    validator_id: String,
    signing_key: SigningKey,
    peers: Vec<String>,
    block_time_ms: u64,
) -> Result<(), String> {
    let listener = TcpListener::bind(listen_addr).await.map_err(|e| e.to_string())?;
    let slots = Arc::new(Semaphore::new(max_peers.max(1)));
    let ctx = ValidatorContext { id: validator_id.clone(), key: signing_key.clone() };
    let server_runtime = runtime.clone();
    let server_chain = chain_id.clone();
    let server_id = node_id.clone();

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let permit = match slots.clone().acquire_owned().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    let chain = server_chain.clone();
                    let id = server_id.clone();
                    let runtime = server_runtime.clone();
                    let validator = Some(ctx.clone());
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, chain, id, runtime, validator).await {
                            tracing::debug!(%peer, error=%e, "validator connection closed");
                        }
                        drop(permit);
                    });
                }
                Err(e) => tracing::warn!(error=%e, "validator accept failed"),
            }
        }
    });

    let consensus = Consensus::new();
    let mut round = 0u64;
    let mut proposed_height = 0u64;
    let interval = Duration::from_millis(block_time_ms.max(250));
    loop {
        let status = runtime.status();
        let height = status.height + 1;
        if let Some(peer) = peers.first() {
            let _ = sync_from_peer(peer, &node_id, &chain_id, runtime.clone(), height, height.saturating_add(100)).await;
        }
        if height != proposed_height {
            round = 0;
            proposed_height = height;
        }
        let leader = consensus.leader_for_round(round, &runtime.validators);
        if leader.as_deref() == Some(validator_id.as_str()) {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs();
            if runtime.proposal(height)?.is_none() {
                let block = runtime.build_block(&chain_id, validator_id.clone(), round, timestamp, 5000, &signing_key)?;
                runtime.store_proposal(&block)?;
                let vote = consensus.sign_vote(&validator_id, &chain_id, block.index, block.round, &block.hash, &signing_key);
                runtime.record_vote(&vote)?;
                broadcast(&peers, &node_id, &chain_id, Message::Proposal(block.clone())).await;
                broadcast(&peers, &node_id, &chain_id, Message::Vote(vote.clone())).await;
            }
        }
        let block = runtime.proposal(height)?;
        if let Some(block) = block {
            let votes = runtime.votes_for(height, block.round, &block.hash)?;
            if let Ok(cert) = consensus.build_certificate(&block, &votes, &runtime.validators) {
                if runtime.commit_block(&block, &cert, &consensus).is_ok() {
                    broadcast(&peers, &node_id, &chain_id, Message::Certificate(cert)).await;
                    continue;
                }
            }
        }
        round = round.saturating_add(1);
        tokio::time::sleep(interval).await;
    }
}

pub async fn sync_from_peer(
    address: &str,
    node_id: &str,
    chain_id: &str,
    runtime: Arc<ChainRuntime>,
    from: u64,
    to: u64,
) -> Result<u64, String> {
    if from > to { return Ok(0); }
    let mut stream = TcpStream::connect(address).await.map_err(|e| e.to_string())?;
    write_message(&mut stream, &Message::Hello {
        node_id: node_id.to_string(),
        chain_id: chain_id.to_string(),
        protocol: PROTOCOL,
    }).await?;
    match read_message(&mut stream).await? {
        Message::Pong => {}
        _ => return Err("handshake rejected".to_string()),
    }
    write_message(&mut stream, &Message::RequestBlocks { from, to }).await?;
    let response = read_message(&mut stream).await?;
    let Message::Blocks(bundles) = response else { return Err("unexpected sync response".to_string()); };
    let consensus = Consensus::new();
    let mut committed = 0;
    for bundle in bundles {
        runtime.store_proposal(&bundle.block)?;
        if runtime.commit_block(&bundle.block, &bundle.certificate, &consensus).is_ok() {
            committed += 1;
        }
    }
    Ok(committed)
}

pub async fn connect_and_submit(address: &str, node_id: String, chain_id: String, tx: Transaction) -> Result<(), String> {
    send(address, &node_id, &chain_id, Message::SubmitTransaction(tx)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn message_round_trip() {
        let message = Message::Hello { node_id: "node-a".into(), chain_id: "myrix-mainnet-1".into(), protocol: PROTOCOL };
        let bytes = serde_json::to_vec(&message).unwrap();
        let decoded: Message = serde_json::from_slice(&bytes).unwrap();
        match decoded {
            Message::Hello { node_id, chain_id, protocol } => {
                assert_eq!(node_id, "node-a");
                assert_eq!(chain_id, "myrix-mainnet-1");
                assert_eq!(protocol, PROTOCOL);
            }
            _ => panic!("wrong message variant"),
        }
    }
}
