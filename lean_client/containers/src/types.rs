use ssz::H256;
use ssz_derive::Ssz;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Ssz, Default)]
#[ssz(transparent)]
pub struct Bytes32(pub H256);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Ssz, Default)]
#[ssz(transparent)]
pub struct Uint64(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ssz, Default)]
#[ssz(transparent)]
pub struct ValidatorIndex(pub u64);