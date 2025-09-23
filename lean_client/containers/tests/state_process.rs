// tests/state_process.rs
use containers::{
    block::{Block, BlockBody, hash_tree_root},
    checkpoint::Checkpoint,
    slot::Slot,
    state::State,
    types::{Bytes32, Uint64, ValidatorIndex},
    vote::{SignedVote, Vote},
};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use ssz::PersistentList as List;
use typenum::U4096;

#[path = "common.rs"]
mod common;
use common::{create_block, sample_config};

#[fixture]
pub fn genesis_state() -> State {
    let config = sample_config();
    State::generate_genesis(Uint64(config.genesis_time), Uint64(config.num_validators))
}

#[test]
fn test_process_slot() {
    let genesis_state = genesis_state();

    assert_eq!(genesis_state.latest_block_header.state_root, Bytes32(ssz::H256::zero()));

    let state_after_slot = genesis_state.process_slot();
    let expected_root = hash_tree_root(&genesis_state);

    assert_eq!(state_after_slot.latest_block_header.state_root, expected_root);

    let state_after_second_slot = state_after_slot.process_slot();
    assert_eq!(state_after_second_slot.latest_block_header.state_root, expected_root);
}

#[test]
fn test_process_slots() {
    let genesis_state = genesis_state();
    let target_slot = Slot(5);

    let new_state = genesis_state.process_slots(target_slot);

    assert_eq!(new_state.slot, target_slot);
    assert_eq!(new_state.latest_block_header.state_root, hash_tree_root(&genesis_state));
}

#[test]
#[should_panic]
fn test_process_slots_backwards() {
    let genesis_state = genesis_state();
    let advanced_state = genesis_state.process_slots(Slot(5));

    let _ = advanced_state.process_slots(Slot(4)); // Should panic
}

#[test]
fn test_process_block_header_valid() {
    let genesis_state = genesis_state();
    let mut state_at_slot_1 = genesis_state.process_slots(Slot(1));
    let genesis_header_root = hash_tree_root(&state_at_slot_1.latest_block_header);

    let block = create_block(1, &mut state_at_slot_1.latest_block_header, None).message;
    let new_state = state_at_slot_1.process_block_header(&block);

    assert_eq!(new_state.latest_finalized.root, genesis_header_root);
    assert_eq!(new_state.latest_justified.root, genesis_header_root);
    assert_eq!(new_state.historical_block_hashes.as_slice(), &[genesis_header_root]);
    assert_eq!(new_state.justified_slots.as_slice(), &[true]);
    assert_eq!(new_state.latest_block_header.slot, Slot(1));
    assert_eq!(new_state.latest_block_header.state_root, Bytes32(ssz::H256::zero()));
}

#[rstest]
#[case(2, 1, None, "Block slot mismatch")]
#[case(1, 2, None, "Incorrect block proposer")]
#[case(1, 1, Some(Bytes32(ssz::H256::from_slice(&[0xde; 32]))), "Block parent root mismatch")]
fn test_process_block_header_invalid(
    #[case] bad_slot: u64,
    #[case] bad_proposer: u64,
    #[case] bad_parent_root: Option<Bytes32>,
    #[case] expected_error: &str,
) {
    let genesis_state = genesis_state();
    let state_at_slot_1 = genesis_state.process_slots(Slot(1));
    let parent_header = &state_at_slot_1.latest_block_header;
    let parent_root = hash_tree_root(parent_header);

    let block = Block {
        slot: Slot(bad_slot),
        proposer_index: ValidatorIndex(bad_proposer),
        parent_root: bad_parent_root.unwrap_or(parent_root),
        state_root: Bytes32(ssz::H256::zero()),
        body: BlockBody { attestations: List::default() },
    };

    let result = std::panic::catch_unwind(|| {
        state_at_slot_1.process_block_header(&block);
    });

    assert!(result.is_err());
    let panic_msg = result.unwrap_err().downcast::<String>().unwrap();
    assert!(panic_msg.contains(expected_error));
}

#[test]
fn test_process_attestations_justification_and_finalization() {
    let mut state = genesis_state();

    // Process slot 1 and block
    let mut state_at_slot_1 = state.process_slots(Slot(1));
    let block1 = create_block(1, &mut state_at_slot_1.latest_block_header, None);
    state = state_at_slot_1.process_block(&block1.message);

    // Process slot 4 and block
    let mut state_at_slot_4 = state.process_slots(Slot(4));
    let block4 = create_block(4, &mut state_at_slot_4.latest_block_header, None);
    state = state_at_slot_4.process_block(&block4.message);

    // Advance to slot 5
    state = state.process_slots(Slot(5));

    let genesis_checkpoint = Checkpoint {
        root: state.historical_block_hashes[0],
        slot: Slot(0),
    };

    let checkpoint4 = Checkpoint {
        root: hash_tree_root(&state.latest_block_header),
        slot: Slot(4),
    };

    let votes_for_4: Vec<SignedVote> = (0..7)
        .map(|i| SignedVote {
            data: Vote {
                validator_id: Uint64(i),
                slot: Slot(4),
                head: checkpoint4.clone(),
                target: checkpoint4.clone(),
                source: genesis_checkpoint.clone(),
            },
            signature: Bytes32(ssz::H256::zero()),
        })
        .collect();

    // Convert Vec to DynamicList with maximum
    let mut votes_list: List<_, U4096> = List::default();
    for v in votes_for_4 { votes_list.push(v).unwrap(); }

    let new_state = state.process_attestations(&votes_list);

    assert_eq!(new_state.latest_justified, checkpoint4);
    assert!(new_state.justified_slots[4]);
    assert_eq!(new_state.latest_finalized, genesis_checkpoint);
    assert!(!new_state.get_justifications().contains_key(&checkpoint4.root));
}