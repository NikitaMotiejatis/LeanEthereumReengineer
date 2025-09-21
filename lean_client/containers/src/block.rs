use crate::{Bytes32, Slot,  SignedVote, ValidatorIndex, State};
use ssz_rs::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct BlockBody {
    pub attestations: List<SignedVote, 1024>,
}

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct BlockHeader {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: Bytes32,
    pub state_root: Bytes32,
    pub body_root: Bytes32,
}

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct Block {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: Bytes32,
    pub state_root: Bytes32,
    pub body: BlockBody,
}

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct SignedBlock {
    pub message: Block,
    /// Placeholder for real signature type
    pub signature: Bytes32,
}

// Helper function to compute hash tree root
pub fn hash_tree_root<T: SimpleSerialize>(value: &mut T) -> Bytes32 {
    let mut result = [0; 32];
    if let Ok(root) = value.hash_tree_root() {
        let root_bytes = root.as_ref();
        result.copy_from_slice(root_bytes);
    }
    Bytes32(result)
}