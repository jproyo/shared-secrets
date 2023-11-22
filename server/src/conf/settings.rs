//! Settings based on [`config-rs`] crate which follows 12-factor configuration model.
//! Configuration file by default is under `config` folder.
//!
use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

/// Struct for storing settings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    raft_addr: String,
    peer_addr: Option<String>,
    http_port: u16,
    node_id: u8,
    api_key: String,
    interval_refresh_secs: u64,
}

impl Settings {
    /// Creates a new instance of `Settings`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be created or deserialized.
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(config::Environment::default())
            .build()?;

        let settings: Settings = s.try_deserialize()?;

        Ok(settings)
    }

    /// Returns the Raft address.
    pub fn raft_addr(&self) -> &str {
        &self.raft_addr
    }

    /// Returns the peer address, if available.
    pub fn peer_addr(&self) -> Option<&str> {
        self.peer_addr.as_deref()
    }

    /// Returns the web server address.
    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    /// Returns the node ID.
    pub fn node_id(&self) -> u8 {
        self.node_id
    }

    /// Returns the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the interval refresh seconds.
    pub fn interval_refresh_secs(&self) -> u64 {
        self.interval_refresh_secs
    }
}
