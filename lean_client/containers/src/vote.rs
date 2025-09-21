use crate::{Bytes32, Slot, Uint64, Checkpoint};
use ssz_rs::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct Vote {
    pub validator_id: Uint64,
    pub slot: Slot,
    pub head: Checkpoint,
    pub target: Checkpoint,
    pub source: Checkpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct SignedVote {
    pub data: Vote,
    pub signature: Bytes32, //placeholder
}