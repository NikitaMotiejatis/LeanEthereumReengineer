use ssz_derive::Ssz;

#[derive(Clone, Debug, PartialEq, Eq, Ssz, Default)]
pub struct Config {
    /// total validators in the network
    pub num_validators: u64,
    /// genesis timestamp (seconds since UNIX epoch)
    pub genesis_time: u64,
}