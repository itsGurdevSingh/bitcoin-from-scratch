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


#[cfg(test)] 
mod test {

    use crate::{block::BlockHeader, types::{BlockHash, MerkleRoot}};

use super::*;


#[test]
fn mines_valid_block() {
    let mut block = Block {
        header: BlockHeader {
            version: 10,
            previous_block_hash: BlockHash([1u8; 32]),
            merkle_root: MerkleRoot ([2u8; 32]),
            timestamp:10000,
            bits: 0x1f00ffff,
            nonce:0
        },
        transactions: vec![]
    };

    let res = Miner::mine(&mut block);
    assert!(res.is_ok());
    assert!(block.header.verify_pow())
}
}