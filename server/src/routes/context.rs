use crate::consensus::handler::ConsensusHandler;

/// The application context.
pub struct AppContext {
    consensus_handler: ConsensusHandler,
    api_key: String,
}

impl AppContext {
    /// Creates a new `AppContext` instance.
    ///
    /// # Arguments
    ///
    /// * `consensus_handler` - The consensus handler.
    /// * `api_key` - The API key as a string.
    ///
    /// # Returns
    ///
    /// A new `AppContext` instance.
    pub fn new(consensus_handler: ConsensusHandler, api_key: &str) -> Self {
        Self {
            consensus_handler,
            api_key: api_key.to_string(),
        }
    }

    /// Returns the consensus handler.
    ///
    /// # Returns
    ///
    /// The consensus handler.
    pub fn consensus_handler(&self) -> ConsensusHandler {
        self.consensus_handler.clone()
    }

    /// Validates the provided key against the stored API key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to validate.
    ///
    /// # Returns
    ///
    /// `true` if the provided key is valid, otherwise `false`.
    pub fn validate_key(&self, key: &str) -> bool {
        self.api_key == key
    }
}
