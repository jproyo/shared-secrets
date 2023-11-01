//! Settings based on [`config-rs`] crate which follows 12-factor configuration model.
//! Configuration file by default is under `config` folder.
//!
use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub addr: String,
    pub id: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub servers: Vec<Server>,
    pub client_id: u8,
    pub api_key: String,
    pub shares_to_create: u8,
    pub shares_required: u8,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(config::Environment::default())
            .build()?;

        let settings: Settings = s.try_deserialize()?;

        Ok(settings)
    }
}
