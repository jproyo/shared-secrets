use serde::{Deserialize, Serialize};
use sss_wrap::secret::secret::Share;

use crate::domain::model::{ClientId, NodeId};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
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
