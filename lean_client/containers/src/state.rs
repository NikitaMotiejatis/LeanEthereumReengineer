use crate::{Bytes32, Checkpoint, ContainerConfig, Slot, Uint64, ValidatorIndex, block::{Block, BlockBody, BlockHeader, SignedBlock, hash_tree_root}, SignedVote};
use ssz_rs::prelude::*;
use std::collections::BTreeMap;

// Limits used across justification tracking
pub const VALIDATOR_REGISTRY_LIMIT: usize = 65_536; // 65536
pub const JUSTIFICATION_ROOTS_LIMIT: usize = 8_192;  // 8192
pub const JUSTIFICATIONS_VALIDATORS_MAX: usize = VALIDATOR_REGISTRY_LIMIT * JUSTIFICATION_ROOTS_LIMIT;

#[derive(Clone, Debug, PartialEq, Eq, SimpleSerialize, Default)]
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
    pub historical_block_hashes: List<Bytes32, 8192>,

    // --- flattened justification tracking ---
    pub justified_slots: List<bool, 8192>,
    pub justifications_roots: List<Bytes32, 8192>,
    // Flattened votes: up to 8192 roots × 65536 validators
    pub justifications_validators: List<bool, JUSTIFICATIONS_VALIDATORS_MAX>,
}

impl State {
    pub fn generate_genesis(genesis_time: Uint64, num_validators: Uint64) -> Self {
        let mut body_for_root = BlockBody { attestations: List::default() };
        let header = BlockHeader {
            slot: Slot(0),
            proposer_index: ValidatorIndex(0),
            parent_root: Bytes32([0; 32]),
            state_root: Bytes32([0; 32]),
            body_root: hash_tree_root(&mut body_for_root),
        };
        Self {
            config: ContainerConfig { genesis_time: genesis_time.0, num_validators: num_validators.0 },
            slot: Slot(0),
            latest_block_header: header,
            latest_justified: Checkpoint { root: Bytes32([0; 32]), slot: Slot(0) },
            latest_finalized: Checkpoint { root: Bytes32([0; 32]), slot: Slot(0) },
            historical_block_hashes: List::default(),
            justified_slots: List::default(),
            justifications_roots: List::default(),
            justifications_validators: List::default(),
        }
    }

    /// Simple RR proposer rule (round-robin).
    pub fn is_proposer(&self, index: ValidatorIndex) -> bool {
        (self.slot.0 % self.config.num_validators) == (index.0 % self.config.num_validators)
    }

    pub fn get_justifications(&self) -> BTreeMap<Bytes32, Vec<bool>> {
        // Chunk validator votes per root using the fixed registry limit
        let limit = VALIDATOR_REGISTRY_LIMIT;
        self.justifications_roots
            .iter()
            .enumerate()
            .map(|(i, root)| {
                let start = i * limit;
                let end = start + limit;
                (*root, self.justifications_validators.as_ref()[start..end].to_vec())
            })
            .collect()
    }

    pub fn with_justifications(mut self, mut map: BTreeMap<Bytes32, Vec<bool>>) -> Self {
        // Expect each root to have exactly `VALIDATOR_REGISTRY_LIMIT` votes
        let limit = VALIDATOR_REGISTRY_LIMIT;
        let mut roots: Vec<_> = map.keys().cloned().collect();
        roots.sort();

        let mut flat = Vec::with_capacity(roots.len() * limit);
        for r in &roots {
            let v = map.remove(r).expect("root present");
            assert_eq!(v.len(), limit, "vote vector must match validator limit");
            flat.extend_from_slice(&v);
        }

    self.justifications_roots = List::try_from(roots).unwrap_or_default();
    self.justifications_validators = List::try_from(flat).unwrap_or_default();
        self
    }

    pub fn with_historical_hashes(mut self, hashes: Vec<Bytes32>) -> Self {
        self.historical_block_hashes = List::try_from(hashes).unwrap_or_default();
        self
    }

    pub fn state_transition(&self, signed_block: SignedBlock, valid_signatures: bool) -> Self {
        assert!(valid_signatures, "Block signatures must be valid");

        let block = signed_block.message;
        let mut state = self.process_slots(block.slot);
        state = state.process_block(&block);

        let mut state_for_hash = state.clone();
        let state_root = hash_tree_root(&mut state_for_hash);
        assert!(block.state_root == state_root, "Invalid block state root");

        state
    }

    pub fn process_slots(&self, target_slot: Slot) -> Self {
        assert!(self.slot < target_slot, "Target slot must be in the future");

        let mut state = self.clone();

        while state.slot < target_slot {
            state = state.process_slot();
            state.slot = Slot(state.slot.0 + 1);
        }

        state
    }

    pub fn process_slot(&self) -> Self {
        if self.latest_block_header.state_root == Bytes32([0; 32]) {
            let mut state_for_hash = self.clone();
            let previous_state_root = hash_tree_root(&mut state_for_hash);

            let mut new_header = self.latest_block_header.clone();
            new_header.state_root = previous_state_root;

            let mut new_state = self.clone();
            new_state.latest_block_header = new_header;
            return new_state;
        }

        self.clone()
    }

    pub fn process_block(&self, block: &Block) -> Self {
        let state = self.process_block_header(block);
        state.process_operations(&block.body)
    }

    pub fn process_block_header(&self, block: &Block) -> Self {
        assert!(block.slot == self.slot, "Block slot mismatch");
        assert!(block.slot > self.latest_block_header.slot, "Block is older than latest header");
        assert!(self.is_proposer(block.proposer_index), "Incorrect block proposer");

        // Create a mutable clone for hash computation
        let mut latest_header_for_hash = self.latest_block_header.clone();
        let parent_root = hash_tree_root(&mut latest_header_for_hash);
        assert!(block.parent_root == parent_root, "Block parent root mismatch");

        let mut new_historical_hashes = self.historical_block_hashes.to_vec();
        new_historical_hashes.push(parent_root);

    let mut new_justified_slots = self.justified_slots.to_vec();
        new_justified_slots.push(self.latest_block_header.slot == Slot(0));

        let num_empty_slots = (block.slot.0 - self.latest_block_header.slot.0 - 1) as usize;
        if num_empty_slots > 0 {
            new_historical_hashes.extend(vec![Bytes32([0; 32]); num_empty_slots]);
            new_justified_slots.extend(vec![false; num_empty_slots]);
        }

        let mut body_for_hash = block.body.clone();
        let body_root = hash_tree_root(&mut body_for_hash);

        let new_latest_block_header = BlockHeader {
            slot: block.slot,
            proposer_index: block.proposer_index,
            parent_root: block.parent_root,
            body_root,
            state_root: Bytes32([0; 32]),
        };

        let mut new_latest_justified = self.latest_justified.clone();
        let mut new_latest_finalized = self.latest_finalized.clone();

        if self.latest_block_header.slot == Slot(0) {
            new_latest_justified.root = parent_root;
            new_latest_finalized.root = parent_root;
        }

        Self {
            config: self.config.clone(),
            slot: self.slot,
            latest_block_header: new_latest_block_header,
            latest_justified: new_latest_justified,
            latest_finalized: new_latest_finalized,
            historical_block_hashes: List::try_from(new_historical_hashes).unwrap_or_default(),
            justified_slots: List::try_from(new_justified_slots).unwrap_or_default(),
            justifications_roots: self.justifications_roots.clone(),
            justifications_validators: self.justifications_validators.clone(),
        }
    }

    pub fn process_operations(&self, body: &BlockBody) -> Self {
        self.process_attestations(&body.attestations) // Pass reference to attestations
    }

    pub fn process_attestations(&self, attestations: &List<SignedVote, 1024>) -> Self {
        let mut justifications = self.get_justifications();
        let mut latest_justified = self.latest_justified.clone();
        let mut latest_finalized = self.latest_finalized.clone();
    let mut justified_slots = self.justified_slots.to_vec();

        for i in 0..attestations.len() {
            if let Some(signed_vote) = attestations.get(i) {
                let vote = signed_vote.data.clone();
                let target_slot = vote.target.slot;
                let source_slot = vote.source.slot;
                let target_root = vote.target.root;
                let source_root = vote.source.root;

                let target_slot_int = target_slot.0 as usize;
                let source_slot_int = source_slot.0 as usize;

                let source_is_justified = justified_slots.get(source_slot_int).copied().unwrap_or(false);
                let target_already_justified = justified_slots.get(target_slot_int).copied().unwrap_or(false);

                let source_root_matches_history = self.historical_block_hashes.get(source_slot_int)
                    .map(|&root| root == source_root)
                    .unwrap_or(false);

                let target_root_matches_history = self.historical_block_hashes.get(target_slot_int)
                    .map(|&root| root == target_root)
                    .unwrap_or(false);

                let mut latest_header_for_hash = self.latest_block_header.clone();
                let target_matches_latest_header = target_slot == self.latest_block_header.slot &&
                    target_root == hash_tree_root(&mut latest_header_for_hash);

                let target_root_is_valid = target_root_matches_history || target_matches_latest_header;
                let target_is_after_source = target_slot > source_slot;
                let target_is_justifiable = target_slot.is_justifiable_after(latest_finalized.slot);

                let is_valid_vote = source_is_justified &&
                    !target_already_justified &&
                    source_root_matches_history &&
                    target_root_is_valid &&
                    target_is_after_source &&
                    target_is_justifiable;

                if !is_valid_vote {
                    continue;
                }

                if !justifications.contains_key(&target_root) {
                    let limit = VALIDATOR_REGISTRY_LIMIT;
                    justifications.insert(target_root, vec![false; limit]);
                }

                let validator_id = vote.validator_id.0 as usize;
                if let Some(votes) = justifications.get_mut(&target_root) {
                    if validator_id < votes.len() && !votes[validator_id] {
                        votes[validator_id] = true;

                        let count = votes.iter().filter(|&&v| v).count();
                        if 3 * count >= 2 * self.config.num_validators as usize {
                            latest_justified = vote.target;

                            while justified_slots.len() <= target_slot_int {
                                justified_slots.push(false);
                            }
                            justified_slots[target_slot_int] = true;

                            justifications.remove(&target_root);

                            let mut is_finalizable = true;
                            for s in (source_slot_int + 1)..target_slot_int {
                                if Slot(s as u64).is_justifiable_after(latest_finalized.slot) {
                                    is_finalizable = false;
                                    break;
                                }
                            }

                            if is_finalizable {
                                latest_finalized = vote.source;
                            }
                        }
                    }
                }
            }
            }

        let mut new_state = self.clone().with_justifications(justifications);

        new_state.latest_justified = latest_justified;
        new_state.latest_finalized = latest_finalized;
    new_state.justified_slots = List::try_from(justified_slots).unwrap_or_default();

        new_state
    }
}

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

    #[test]
    fn test_hash_tree_root() {
        let body = BlockBody {
            attestations: List::default()
        };
        let mut block = Block {
            slot: Slot(1),
            proposer_index: ValidatorIndex(0),
            parent_root: Bytes32([0; 32]),
            state_root: Bytes32([0; 32]),
            body,
        };

        let root = hash_tree_root(&mut block);
        assert_ne!(root, Bytes32([0; 32])); // Should compute actual hash
    }

    #[test]
    fn test_process_slots() {
        let genesis_state = State::generate_genesis(Uint64(0), Uint64(10));
        let target_slot = Slot(5);

        let new_state = genesis_state.process_slots(target_slot);

        assert_eq!(new_state.slot, target_slot);
        let mut genesis_state_for_hash = genesis_state.clone(); //this is sooooo bad
        assert_eq!(new_state.latest_block_header.state_root, hash_tree_root(&mut genesis_state_for_hash));
    }

}