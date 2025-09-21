use ssz_rs::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, SimpleSerialize, Default)]
pub struct Bytes32(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, SimpleSerialize, Default)]
pub struct Uint64(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, SimpleSerialize, Default)]
pub struct ValidatorIndex(pub u64);