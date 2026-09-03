use crate::{
    block::BlockHeader,
    blockchain::{BlockNode, Blockchain, error::BlockchainError},
    presistaence::DbPersistence,
    utils::time::Time,
};

pub struct HeaderValidator;

impl HeaderValidator {
    pub fn validate<S: DbPersistence>(
        chain: &Blockchain<S>,
        header: &BlockHeader,
        parent: &BlockNode,
    ) -> Result<(), BlockchainError> {
        if header.previous_block_hash != parent.hash {
            return Err(BlockchainError::InvalidHeader);
        }

        let expected_bits = chain.expected_bits(parent)?;

        if header.bits != expected_bits {
            return Err(BlockchainError::InvalidHeader);
        }

        if header.timestamp > Time::unix_timestamp() + 7200 {
            return Err(BlockchainError::InvalidHeader);
        }

        if chain.median_timestamp(parent.clone())? >= header.timestamp {
            return Err(BlockchainError::InvalidHeader);
        }

        // PoW validation should also belong here.
        // header.validate_proof_of_work() ...
        if !header.verify_pow() {
            return Err(BlockchainError::InvalidHeader);
        }

        Ok(())
    }
}
