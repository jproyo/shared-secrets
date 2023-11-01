use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
pub struct ClientId(pub u64);

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
pub struct NodeId(pub u8);

impl Deref for ClientId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for NodeId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
