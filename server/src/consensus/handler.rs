use std::sync::Arc;

use bincode::serialize;
use log::info;
use riteraft::Mailbox;
use sss_wrap::secret::secret::Share;

use crate::domain::error::SecretServerError;
use crate::domain::model::ClientId;

use super::messages::Message;
use super::raft::HashStore;

#[derive(Clone)]
pub struct ConsensusHandler {
    storage: HashStore,
    mailbox: Arc<Mailbox>,
}

impl std::fmt::Debug for ConsensusHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsensusHandler").finish()
    }
}

impl ConsensusHandler {
    pub fn new(storage: HashStore, mailbox: Arc<Mailbox>) -> Self {
        Self { storage, mailbox }
    }

    pub async fn leave(&self) -> Result<(), SecretServerError> {
        self.mailbox.leave().await?;
        Ok(())
    }

    pub fn get(&self, id: ClientId) -> Result<Option<Share>, SecretServerError> {
        self.storage.get(id)
    }

    pub fn insert(&mut self, id: ClientId, share: Share) -> Result<(), SecretServerError> {
        self.storage.insert(id, share)
    }

    pub fn is_begin_refresh(&self) -> bool {
        self.storage.is_begin_refresh()
    }

    pub async fn start_refresh(&self) -> Result<(), SecretServerError> {
        info!("Sending start refresh message to the rest of the participants in the network");
        let message = serialize(&Message::StartRefresh {
            node_id: self.storage.node_id(),
        })?;
        let _ = self.mailbox.send(message).await?;
        Ok(())
    }
}
