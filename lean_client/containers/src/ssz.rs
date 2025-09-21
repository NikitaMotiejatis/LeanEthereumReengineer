use ssz_rs::prelude::*;
use crate::{Block, BlockHeader, BlockBody, Bytes32};

pub trait HashTreeRoot {
    fn hash_tree_root(&self) -> Result<Bytes32, MerkleizationError>;
}

impl HashTreeRoot for BlockHeader {
    fn hash_tree_root(&self) -> Result<Bytes32, MerkleizationError> {
        let root = self.hash_tree_root()?;
        Ok(Bytes32(root))
    }
}

impl HashTreeRoot for BlockBody {
    fn hash_tree_root(&self) -> Result<Bytes32, MerkleizationError> {
        let root = self.hash_tree_root()?;
        Ok(Bytes32(root))
    }
}

impl HashTreeRoot for Block {
    fn hash_tree_root(&self) -> Result<Bytes32, MerkleizationError> {
        let root = self.hash_tree_root()?;
        Ok(Bytes32(root))
    }
}

// Helper function to compute hash tree root
pub fn compute_hash_tree_root<T: SimpleSerialize>(value: &T) -> Result<Bytes32, MerkleizationError> {
    let root = value.hash_tree_root()?;
    Ok(Bytes32(root))
}