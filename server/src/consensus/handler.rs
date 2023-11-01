use std::sync::Arc;

use bincode::serialize;
use log::info;
use riteraft::Mailbox;
use sss_wrap::secret::secret::ShareMeta;

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

    pub fn get(&self, id: ClientId) -> Result<Option<ShareMeta>, SecretServerError> {
        self.storage.get(id)
    }

    pub fn insert(&mut self, id: ClientId, share: ShareMeta) -> Result<(), SecretServerError> {
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
        self.mailbox
            .send(message)
            .await
            .map_err(|e| e.into())
            .map(|_| ())
    }

    pub async fn refresh_secrets(&self) -> Result<(), SecretServerError> {
        let messages = self
            .storage
            .storage()
            .read()?
            .iter()
            .map(|(id, share)| {
                let poly = share.share.renew_poly(&share.meta);
                let r = (0..share.meta.shares_to_create)
                    .map(move |i| {
                        let share = poly.get_share(i + 1, share.share.ys_len());
                        info!("Refreshing message with share {:?}", share);
                        Message::Refresh {
                            client_id: id.clone(),
                            new_share: share,
                        }
                    })
                    .collect::<Vec<_>>();
                r
            })
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        info!(
            "Sending refresh messages to the rest of the participants in the network {:?}",
            messages.len()
        );
        for message in messages {
            let message = serialize(&message)?;
            let _ = self.mailbox.send(message).await?;
        }
        Ok(())
    }

    pub async fn finish_refresh(&self) -> Result<(), SecretServerError> {
        info!("Sending finish refresh message to the rest of the participants in the network");
        let message = serialize(&Message::FinishRefresh {
            node_id: self.storage.node_id(),
        })?;
        let _ = self.mailbox.send(message).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use riteraft::Raft;
    use slog::o;
    use sss_wrap::from_secrets;
    use sss_wrap::secret::secret::Metadata;

    use super::*;

    #[tokio::test]
    async fn test_refresh_secrets_no_secrets() -> Result<(), SecretServerError> {
        let storage = HashStore::new(crate::domain::model::NodeId(1)); // Initialize the storage
        let raft = Raft::new(
            "localhost:8080".to_string(),
            storage.clone(),
            slog::Logger::root(slog::Discard, o!()),
        );
        let secret_server = ConsensusHandler::new(storage.clone(), Arc::new(raft.mailbox()));
        secret_server.refresh_secrets().await?;
        assert_eq!(storage.storage().read()?.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_refresh_secrets_with_secrets() -> Result<(), SecretServerError> {
        let storage = HashStore::new(crate::domain::model::NodeId(1)); // Initialize the storage
        let raft = Raft::new(
            "localhost:8080".to_string(),
            storage.clone(),
            slog::Logger::root(slog::Discard, o!()),
        );
        let mail = raft.mailbox();
        tokio::spawn(raft.lead());
        let secret_server = ConsensusHandler::new(storage.clone(), Arc::new(mail));
        let secret_vec = "test-secret".to_string().into_bytes();
        let secrets = from_secrets(secret_vec.clone(), 9, 10, None).unwrap();
        for (i, x) in secrets.clone().into_iter().enumerate() {
            secret_server
                .clone()
                .insert(
                    ClientId(i as u64),
                    ShareMeta::new(x.into(), Metadata::new(9, 10, secret_vec.len())),
                )
                .unwrap();
        }
        secret_server.refresh_secrets().await?;
        assert_eq!(storage.storage().read()?.len(), 10);
        for (i, x) in storage.storage().read()?.iter() {
            assert_ne!(x.share, secrets[i.0 as usize].clone().into());
        }

        Ok(())
    }
}
