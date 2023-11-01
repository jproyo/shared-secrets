use serde::{Deserialize, Serialize};
use sss_wrap::secret::secret::Share;

use crate::domain::model::{ClientId, NodeId};

/// Enum representing different types of messages for Raft consensus protocol.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Message to start refreshing with the given `node_id`.
    StartRefresh { node_id: NodeId },
    /// Message to refresh with the given `client_id` and `new_share`.
    Refresh {
        client_id: ClientId,
        new_share: Share,
    },
    /// Message to finish refreshing with the given `node_id`.
    FinishRefresh { node_id: NodeId },
}
