use crate::{Bytes32, Slot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub root: Bytes32,
    pub slot: Slot,
}
