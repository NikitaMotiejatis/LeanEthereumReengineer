/// Core consensus parameters and chain presets
/// for the Lean Consensus Experimental Chain.

/// --- Type Wrappers ---

/*
/// Don't really need this wrapper around u64 now, but it might help in the future.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uint64 (pub u64);

impl Uint64 {
    pub const fn new(value: u64) -> Self {
        Uint64 (value)
    }

    #[inline]
    pub fn get(&self) -> u64 {
        self.0
    }
}*/

/// A value in basis points (1/10000).
/// Valid range: 0 <= value <= 10000
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasisPoint(pub u64);

impl BasisPoint {
    pub const MAX: u64 = 10_000;

    /// Create a new BasisPoint, returning `None` if the value is invalid.
    pub const fn new(value: u64) -> Option<Self> {
        if value <= Self::MAX {
            Some(BasisPoint(value))
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// --- Time Parameters ---

/// Number of intervals per slot for forkchoice processing.
pub const INTERVALS_PER_SLOT: u64 = u64(4);

/// The fixed duration of a single slot in milliseconds.
pub const SLOT_DURATION_MS: u64 = u64(4000);

/// The fixed duration of a single slot in seconds.
pub const SECONDS_PER_SLOT: u64 = u64(SLOT_DURATION_MS.0 / 1000);

/// Seconds per forkchoice processing interval.
pub const SECONDS_PER_INTERVAL: u64 = u64(SECONDS_PER_SLOT.0 / INTERVALS_PER_SLOT.0);

/// The number of slots to look back for justification.
pub const JUSTIFICATION_LOOKBACK_SLOTS: u64 = u64(3);

/// Deadlines (validated BasisPoint constants).
pub const PROPOSER_REORG_CUTOFF_BPS: BasisPoint =
    match BasisPoint::new(2500) {
        Some(bps) => bps,
        None => panic!("Invalid proposer cutoff basis points"),
    };

pub const VOTE_DUE_BPS: BasisPoint =
    match BasisPoint::new(5000) {
        Some(bps) => bps,
        None => panic!("Invalid vote due basis points"),
    };

pub const FAST_CONFIRM_DUE_BPS: BasisPoint =
    match BasisPoint::new(7500) {
        Some(bps) => bps,
        None => panic!("Invalid fast confirm basis points"),
    };

pub const VIEW_FREEZE_CUTOFF_BPS: BasisPoint =
    match BasisPoint::new(7500) {
        Some(bps) => bps,
        None => panic!("Invalid view freeze cutoff basis points"),
    };

/// --- State List Length Presets ---

pub const HISTORICAL_ROOTS_LIMIT: u64 = u64(1 << 18); // 2^18
pub const VALIDATOR_REGISTRY_LIMIT: u64 = u64(1 << 12); // 2^12

/// --- Chain Configuration Struct ---

#[derive(Clone, Debug)]
pub struct ChainConfig {
    pub slot_duration_ms: u64,
    pub second_per_slot: u64,
    pub justification_lookback_slots: u64,
    pub proposer_reorg_cutoff_bps: BasisPoint,
    pub vote_due_bps: BasisPoint,
    pub fast_confirm_due_bps: BasisPoint,
    pub view_freeze_cutoff_bps: BasisPoint,
    pub historical_roots_limit: u64,
    pub validator_registry_limit: u64,
}

/// The Devnet Chain Configuration.
pub const DEVNET_CONFIG: ChainConfig = ChainConfig {
    slot_duration_ms: SLOT_DURATION_MS,
    second_per_slot: SECONDS_PER_SLOT,
    justification_lookback_slots: JUSTIFICATION_LOOKBACK_SLOTS,
    proposer_reorg_cutoff_bps: PROPOSER_REORG_CUTOFF_BPS,
    vote_due_bps: VOTE_DUE_BPS,
    fast_confirm_due_bps: FAST_CONFIRM_DUE_BPS,
    view_freeze_cutoff_bps: VIEW_FREEZE_CUTOFF_BPS,
    historical_roots_limit: HISTORICAL_ROOTS_LIMIT,
    validator_registry_limit: VALIDATOR_REGISTRY_LIMIT,
};
