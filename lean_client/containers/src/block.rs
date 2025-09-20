use crate::{Bytes32, Slot, Uint64, SignedVote};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockBody {
    pub attestations: Vec<SignedVote>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub slot: Slot,
    pub proposer_index: Uint64,
    pub parent_root: Bytes32,
    pub state_root: Bytes32,
    pub body_root: Bytes32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub slot: Slot,
    pub proposer_index: Uint64,
    pub parent_root: Bytes32,
    pub state_root: Bytes32,
    pub body: BlockBody,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBlock {
    pub message: Block,
    /// Placeholder for real signature type
    pub signature: Bytes32,
}
