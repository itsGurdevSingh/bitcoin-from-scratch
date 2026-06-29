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
    pub timestamp: u32,
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

        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(self.previous_block_hash.as_bytes());
        bytes.extend_from_slice(self.merkle_root.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.bits.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());

        bytes
    }
}

#[cfg(test)]
mod test {

    use crate::{block::Block, miner::Miner};

    use super::*;

    #[test]
    fn modified_header_invalidates_pow() {
        let mut block = Block {
            header: BlockHeader {
                version: 10,
                previous_block_hash: BlockHash([1u8; 32]),
                merkle_root: MerkleRoot([2u8; 32]),
                timestamp: 10000,
                bits: 0x1f00ffff,
                nonce: 0,
            },
            transactions: vec![],
        };

        let _res = Miner::mine(&mut block);

        assert!(block.header.verify_pow());

        block.header.version = 11;

        assert!(!block.header.verify_pow())
    }
}
