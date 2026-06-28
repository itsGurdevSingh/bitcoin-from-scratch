use crate::{block::Block, miner::MiningError};

pub struct Miner;

impl Miner {
    pub fn mine(block: &mut Block) -> Result<(), MiningError> {
        block.header.nonce = 0;
        loop {
            if block.header.verify_pow() {
                return Ok(());
            }
            if block.header.nonce == u32::MAX {
                return Err(MiningError::NonceExhausted);
            }
            block.header.nonce += 1;
        }
    }
}
