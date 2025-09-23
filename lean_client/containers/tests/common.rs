use containers::{
    block::{Block, BlockBody, BlockHeader, SignedBlock, hash_tree_root},
    checkpoint::Checkpoint,
    config::Config,
    slot::Slot,
    state::State,
    types::{Bytes32, ValidatorIndex},
    vote::{SignedVote},
};
use ssz_rs::prelude::*;

pub const DEVNET_CONFIG_VALIDATOR_REGISTRY_LIMIT: usize = 65536;

pub fn create_block(slot: u64, parent_header: &mut BlockHeader, votes: Option<List<SignedVote, 1024>>) -> SignedBlock {
    let body = BlockBody {
        attestations: votes.map(List::from).unwrap_or_default(),
    };

    let block_message = Block {
        slot: Slot(slot),
        proposer_index: ValidatorIndex(slot % 10),
        parent_root: hash_tree_root(parent_header),
        state_root: Bytes32([0; 32]),
        body,
    };

    SignedBlock {
        message: block_message,
        signature: Bytes32([0; 32]),
    }
}

pub fn create_votes(indices: &[usize]) -> Vec<bool> {
    let mut votes = vec![false; DEVNET_CONFIG_VALIDATOR_REGISTRY_LIMIT];
    for &index in indices {
        if index < votes.len() {
            votes[index] = true;
        }
    }
    votes
}

pub fn sample_block_header() -> BlockHeader {
    BlockHeader {
        slot: Slot(0),
        proposer_index: ValidatorIndex(0),
        parent_root: Bytes32([0; 32]),
        state_root: Bytes32([0; 32]),
        body_root: Bytes32([0; 32]),
    }
}

pub fn sample_checkpoint() -> Checkpoint {
    Checkpoint {
        root: Bytes32([0; 32]),
        slot: Slot(0),
    }
}

pub fn base_state(config: Config) -> State {
    State {
        config,
        slot: Slot(0),
        latest_block_header: sample_block_header(),
        latest_justified: sample_checkpoint(),
        latest_finalized: sample_checkpoint(),
        historical_block_hashes: List::default(),
        justified_slots: List::default(),
        justifications_roots: List::default(),
        justifications_validators: List::default(),
    }
}

pub fn sample_config() -> Config {
    Config {
        num_validators: 10,
        genesis_time: 0,
    }
}