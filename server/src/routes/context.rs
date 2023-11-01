use std::sync::Arc;

use riteraft::Mailbox;

use crate::consensus::raft::HashStore;

pub struct AppContext {
    mailbox: Arc<Mailbox>,
    store: HashStore,
    api_key: String,
}

impl AppContext {
    pub fn new(mailbox: Arc<Mailbox>, store: HashStore, api_key: &str) -> Self {
        Self {
            mailbox,
            store,
            api_key: api_key.to_string(),
        }
    }

    pub fn store(&self) -> HashStore {
        self.store.clone()
    }

    pub fn mailbox(&self) -> Arc<Mailbox> {
        self.mailbox.clone()
    }

    pub fn validate_key(&self, key: &str) -> bool {
        self.api_key == key
    }
}
