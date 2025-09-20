// NOTE: This `State` holds `containers::config::Config` to mirror the spec exactly.
// Your runtime can convert from `chain::config::Config` (see chain/src/config.rs).
use crate::{
    Bytes32, Checkpoint, ContainerConfig, Slot, Uint64, ValidatorIndex,
    block::{Block, BlockBody, BlockHeader, SignedBlock},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    // --- configuration (spec-local) ---
    pub config: ContainerConfig,

    // --- slot / header tracking ---
    pub slot: Slot,
    pub latest_block_header: BlockHeader,

    // --- fork-choice checkpoints ---
    pub latest_justified: Checkpoint,
    pub latest_finalized: Checkpoint,

    // --- historical data ---
    pub historical_block_hashes: Vec<Bytes32>,

    // --- flattened justification tracking ---
    pub justified_slots: Vec<bool>,
    pub justifications_roots: Vec<Bytes32>,
    pub justifications_validators: Vec<bool>, // concatenated chunks per-root
}

impl State {
    pub fn generate_genesis(genesis_time: Uint64, num_validators: Uint64) -> Self {
        let body = BlockBody { attestations: vec![] };
        let header = BlockHeader {
            slot: Slot(0),
            proposer_index: Uint64(0),
            parent_root: Bytes32([0; 32]),
            state_root: Bytes32([0; 32]),
            body_root: ssz_merkle_root_of_body(&body),
        };
        Self {
            config: ContainerConfig { genesis_time: genesis_time.0, num_validators: num_validators.0 },
            slot: Slot(0),
            latest_block_header: header,
            latest_justified: Checkpoint { root: Bytes32([0; 32]), slot: Slot(0) },
            latest_finalized: Checkpoint { root: Bytes32([0; 32]), slot: Slot(0) },
            historical_block_hashes: vec![],
            justified_slots: vec![],
            justifications_roots: vec![],
            justifications_validators: vec![],
        }
    }

    /// Simple RR proposer rule (round-robin).
    pub fn is_proposer(&self, index: ValidatorIndex) -> bool {
        (self.slot.0 % self.config.num_validators) == (index.0 % self.config.num_validators)
    }

    pub fn get_justifications(&self) -> BTreeMap<Bytes32, Vec<bool>> {
        let limit = self.config.num_validators as usize;
        self.justifications_roots
            .iter()
            .enumerate()
            .map(|(i, root)| {
                let start = i * limit;
                let end = start + limit;
                (*root, self.justifications_validators[start..end].to_vec())
            })
            .collect()
    }

    pub fn with_justifications(mut self, mut map: BTreeMap<Bytes32, Vec<bool>>) -> Self {
        let limit = self.config.num_validators as usize;
        let mut roots: Vec<_> = map.keys().cloned().collect();
        roots.sort();
        let mut flat = Vec::with_capacity(roots.len() * limit);
        for r in &roots {
            let v = map.remove(r).expect("root present");
            assert_eq!(v.len(), limit, "vote vector must match validator limit");
            flat.extend_from_slice(&v);
        }
        self.justifications_roots = roots;
        self.justifications_validators = flat;
        self
    }
}

// Stub for now; swap with real SSZ tree-hash when ready.
fn ssz_merkle_root_of_body(_body: &BlockBody) -> Bytes32 { Bytes32([0; 32]) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proposer_round_robin() {
        let st = State::generate_genesis(Uint64(0), Uint64(4));
        assert!(State { config: st.config.clone(), ..st.clone() }.is_proposer(ValidatorIndex(0)));
    }

    #[test]
    fn slot_justifiability_rules() {
        use crate::slot::Slot;
        assert!(Slot(1).is_justifiable_after(Slot(0)));
        assert!(Slot(9).is_justifiable_after(Slot(0))); // perfect square
        assert!(Slot(6).is_justifiable_after(Slot(0))); // pronic (2*3)
    }
}
