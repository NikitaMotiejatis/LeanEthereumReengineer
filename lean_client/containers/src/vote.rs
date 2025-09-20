use crate::{Bytes32, Slot, Uint64, Checkpoint};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub validator_id: Uint64,
    pub slot: Slot,
    pub head: Checkpoint,
    pub target: Checkpoint,
    pub source: Checkpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedVote {
    pub data: Vote,
    /// Placeholder for real signature type (e.g., XMSS later)
    pub signature: Bytes32,
}
