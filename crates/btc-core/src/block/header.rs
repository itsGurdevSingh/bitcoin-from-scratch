use crate::types::{BlockHash, MerkleRoot};

pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: BlockHash,
    pub merkle_root: MerkleRoot,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u32,
}