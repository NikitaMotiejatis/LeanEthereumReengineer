use crate::{Bytes32, Slot};
use ssz_derive::Ssz;

#[derive(Clone, Debug, PartialEq, Eq, Ssz, Default)]
pub struct Checkpoint {
    pub root: Bytes32,
    pub slot: Slot,
}