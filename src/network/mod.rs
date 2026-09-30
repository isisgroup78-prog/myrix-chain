use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
    pub peers: HashMap<String, Peer>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer_id: String, address: String) {
        self.peers.insert(peer_id.clone(), Peer { id: peer_id, address });
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }
}
