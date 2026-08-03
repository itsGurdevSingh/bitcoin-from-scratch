use crate::{
    crypto::sha256d,
    difficulty::Difficulty,
    serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
    types::{BlockHash, MerkleRoot},
};

#[derive(Clone, Debug, PartialEq, Eq)]
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

        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(self.previous_block_hash.as_bytes());
        bytes.extend_from_slice(self.merkle_root.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.bits.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for BlockHeader {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        if bytes.len() < 84 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let version = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        let previous_block_hash = BlockHash(
            bytes[offset..offset + 32]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 32;

        let merkle_root = MerkleRoot(
            bytes[offset..offset + 32]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 32;

        let timestamp = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 8;

        let bits = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        let nonce = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        Ok((
            Self {
                version,
                previous_block_hash,
                merkle_root,
                timestamp,
                bits,
                nonce,
            },
            offset,
        ))
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
