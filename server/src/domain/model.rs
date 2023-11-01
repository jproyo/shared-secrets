use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
pub struct ClientId(u64);

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
pub struct NodeId(pub u8);

impl Deref for NodeId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
