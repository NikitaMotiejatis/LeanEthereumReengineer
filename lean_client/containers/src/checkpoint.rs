use crate::{Bytes32, Slot};
use ssz_rs::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct Checkpoint {
    pub root: Bytes32,
    pub slot: Slot,
}