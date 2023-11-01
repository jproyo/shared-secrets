//! Settings based on [`config-rs`] crate which follows 12-factor configuration model.
//! Configuration file by default is under `config` folder.
//!
use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    raft_addr: String,
    peer_addr: Option<String>,
    web_server: String,
    node_id: u8,
    api_key: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::Environment::default())
            .add_source(File::with_name("config/default"))
            .build()?;

        let settings: Settings = s.try_deserialize()?;

        Ok(settings)
    }

    pub fn raft_addr(&self) -> &str {
        &self.raft_addr
    }

    pub fn peer_addr(&self) -> Option<&str> {
        self.peer_addr.as_deref()
    }

    pub fn web_server(&self) -> &str {
        &self.web_server
    }

    pub fn node_id(&self) -> u8 {
        self.node_id
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}
