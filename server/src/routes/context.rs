use crate::consensus::handler::ConsensusHandler;

pub struct AppContext {
    consensus_handler: ConsensusHandler,
    api_key: String,
}

impl AppContext {
    pub fn new(consensus_handler: ConsensusHandler, api_key: &str) -> Self {
        Self {
            consensus_handler,
            api_key: api_key.to_string(),
        }
    }

    pub fn consensus_handler(&self) -> ConsensusHandler {
        self.consensus_handler.clone()
    }

    pub fn validate_key(&self, key: &str) -> bool {
        self.api_key == key
    }
}
