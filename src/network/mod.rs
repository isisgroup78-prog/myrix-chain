use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Peer { pub id: String, pub address: String }

#[derive(Clone, Debug)]
pub struct Network {
    pub peers: HashMap<String, Peer>,
    pub max_peers: usize,
}

impl Default for Network {
    fn default() -> Self { Self::new(128) }
}

impl Network {
    pub fn new(max_peers: usize) -> Self { Self { peers: HashMap::new(), max_peers } }

    pub fn add_peer(&mut self, peer_id: String, address: String) -> Result<(), String> {
        if peer_id.is_empty() || address.is_empty() { return Err("peer id and address are required".to_string()); }
        if !self.peers.contains_key(&peer_id) && self.peers.len() >= self.max_peers { return Err("peer limit reached".to_string()); }
        self.peers.insert(peer_id.clone(), Peer { id: peer_id, address });
        Ok(())
    }

    pub fn remove_peer(&mut self, peer_id: &str) { self.peers.remove(peer_id); }

    pub fn peer_count(&self) -> usize { self.peers.len() }
}
