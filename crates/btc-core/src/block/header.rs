use crate::{
    crypto::sha256d,
    difficulty::Difficulty,
    serialization::BitcoinSerialize,
    types::{BlockHash, MerkleRoot},
};

pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: BlockHash,
    pub merkle_root: MerkleRoot,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn hash(&self) -> BlockHash {
        let serialize = self.serialize();

        let hash = sha256d(&serialize);

        BlockHash(hash)
    }

    pub fn verify_pow(&self) -> bool {
        let hash = self.hash().into_bytes();
        let target = Difficulty::target_from_bits(self.bits);

        hash <= target
    }
}

impl BitcoinSerialize for BlockHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes .extend_from_slice(&self.version.to_le_bytes());
        bytes .extend_from_slice(self.previous_block_hash.as_bytes());
        bytes .extend_from_slice(self.merkle_root.as_bytes());
        bytes .extend_from_slice(&self.timestamp.to_le_bytes());
        bytes .extend_from_slice(&self.bits.to_le_bytes());
        bytes .extend_from_slice(&self.nonce.to_le_bytes());

        bytes 
    }
}
