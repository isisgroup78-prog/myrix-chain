use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct CrossChainBridge {
    pub routes: HashMap<String, String>,
}

impl CrossChainBridge {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, chain: String, addr: String) {
        self.routes.insert(chain, addr);
    }
}
