use slog::{info, Logger};
use sss_wrap::secret::secret::{RenewableShare, Share};

use async_trait::async_trait;
use bincode::{deserialize, serialize};
use riteraft::{Mailbox, Raft, Result as RiteResult, Store};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;

use crate::domain::error::SecretServerError;
use crate::domain::model::{ClientId, NodeId};

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    StartRefresh {
        node_id: NodeId,
    },
    Refresh {
        client_id: ClientId,
        new_share: Share,
    },
    FinishRefresh {
        node_id: NodeId,
    },
}

#[derive(Clone)]
pub struct HashStore {
    node_id: NodeId,
    storage: Arc<RwLock<HashMap<ClientId, Share>>>,
    refreshing: Arc<AtomicBool>,
}

impl std::fmt::Debug for HashStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashStore")
            .field("node_id", &self.node_id)
            .field("refreshing", &self.refreshing)
            .finish()
    }
}

impl HashStore {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            node_id,
            refreshing: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn get(&self, id: ClientId) -> Result<Option<Share>, SecretServerError> {
        Ok(self.storage.read()?.get(&id).cloned())
    }

    pub fn insert(&mut self, id: ClientId, share: Share) -> Result<(), SecretServerError> {
        self.storage.write()?.insert(id, share);
        Ok(())
    }
}

#[async_trait]
impl Store for HashStore {
    async fn apply(&mut self, message: &[u8]) -> RiteResult<Vec<u8>> {
        let message: Message = deserialize(message)?;
        let message: Vec<u8> = match message {
            Message::StartRefresh { node_id } => {
                if node_id != self.node_id {
                    self.refreshing
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                serialize(&Message::StartRefresh { node_id })?
            }
            Message::Refresh {
                client_id,
                new_share,
            } => {
                if new_share.id() == *self.node_id.deref() {
                    let db_read = self
                        .storage
                        .read()
                        .map_err(|e| -> SecretServerError { e.into() })?;
                    let old_share = db_read.get(&client_id).ok_or(SecretServerError::NotFound)?;
                    let new_share_to_store =
                        RenewableShare::renew_with_share(&new_share, &old_share);
                    let mut db = self
                        .storage
                        .write()
                        .map_err(|e| -> SecretServerError { e.into() })?;
                    db.insert(client_id, new_share_to_store);
                }
                serialize(&Message::Refresh {
                    client_id,
                    new_share: new_share.clone(),
                })?
            }
            Message::FinishRefresh { node_id } => {
                if node_id != self.node_id {
                    self.refreshing
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                }
                serialize(&Message::FinishRefresh { node_id })?
            }
        };
        Ok(message)
    }

    async fn snapshot(&self) -> RiteResult<Vec<u8>> {
        Ok(serialize(
            &self
                .storage
                .read()
                .map_err(|e| -> SecretServerError { e.into() })?
                .clone(),
        )?)
    }

    async fn restore(&mut self, snapshot: &[u8]) -> RiteResult<()> {
        let new: HashMap<ClientId, Share> = deserialize(snapshot)?;
        let mut db = self
            .storage
            .write()
            .map_err(|e| -> SecretServerError { e.into() })?;
        let _ = std::mem::replace(&mut *db, new);
        Ok(())
    }
}

pub async fn init_consensus(
    raft_addr: String,
    peer_addr: Option<String>,
    store: HashStore,
    logger: Logger,
) -> Result<(JoinHandle<Result<(), riteraft::Error>>, Arc<Mailbox>), SecretServerError> {
    let raft = Raft::new(raft_addr.to_owned(), store.clone(), logger.clone());
    let mailbox = Arc::new(raft.mailbox());
    let (raft_handle, mailbox) = if let Some(addr) = peer_addr {
        info!(logger, "running in follower mode");
        let handle = tokio::spawn(raft.join(addr.to_owned()));
        (handle, mailbox)
    } else {
        info!(logger, "running in leader mode");
        let handle = tokio::spawn(raft.lead());
        (handle, mailbox)
    };
    Ok((raft_handle, mailbox))
}
