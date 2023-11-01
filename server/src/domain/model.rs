use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
pub struct ClientId {
    id: u64,
    shares_required: u8,
    shares_to_create: u8,
    sec_len: usize,
}

impl ClientId {
    pub fn new(id: u64, shares_required: u8, shares_to_create: u8, sec_len: usize) -> Self {
        Self {
            id,
            shares_required,
            shares_to_create,
            sec_len,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn shares_required(&self) -> u8 {
        self.shares_required
    }

    pub fn shares_to_create(&self) -> u8 {
        self.shares_to_create
    }

    pub fn sec_len(&self) -> usize {
        self.sec_len
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
pub struct NodeId(pub u8);

impl Deref for NodeId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
