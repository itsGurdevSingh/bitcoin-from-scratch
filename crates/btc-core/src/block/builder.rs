use crate::{
    block::{Block, BlockHeader, BuilderErrors},
    blockchain::Blockchain,
    merkle::MerkleTree,
    transaction::Transaction,
    utils::time::Time,
};

pub struct Builder;

impl Builder {
    pub fn build(transactions: &[Transaction], chain: &Blockchain) -> Result<Block, BuilderErrors> {
        Ok(Block {
            header: BlockHeader {
                version: 1,
                previous_block_hash: chain
                    .tip()
                    .map_err(|e| BuilderErrors::Chain(e))?
                    .header
                    .hash(),
                merkle_root: MerkleTree::compute_root(transactions).unwrap(),
                timestamp: Time::unix_timestamp(),
                bits: chain.expected_bits().map_err(|e| BuilderErrors::Chain(e))?,
                nonce: 0,
            },
            transactions: transactions.to_vec(),
        })
    }
}
