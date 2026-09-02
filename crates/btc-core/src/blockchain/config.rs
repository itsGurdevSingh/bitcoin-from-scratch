// 1788279306

use crate::types::MerkleRoot;

pub struct GenesisConfig {
    pub timestamp: u64,
    pub nonce: u32,
    pub bits: u32,
    pub merkle_root: MerkleRoot,
}