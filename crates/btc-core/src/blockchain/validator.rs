use crate::{
    block::Block,
    blockchain::{Blockchain, error::BlockchainError},
    utils::time::Time,
};

pub struct ChainValidator;

impl ChainValidator {
    pub fn validate(chain: &Blockchain, block: &Block) -> Result<(), BlockchainError> {
        // validate header
        // is valid previos block hash
        if !(block.header.previous_block_hash
            == chain
                .get_node_by_hash(block.header.previous_block_hash)
                .ok_or(BlockchainError::InvalidHeader)?
                .hash
            && block.header.bits == chain.expected_bits()?
            && block.header.timestamp < Time::unix_timestamp() + 7200
            && chain.median_timestamp()? < block.header.timestamp)
        {
            return Err(BlockchainError::InvalidHeader);
        }

        block
            .validate_block()
            .map_err(|e| BlockchainError::Block(e))?;

        Ok(())
    }
}
