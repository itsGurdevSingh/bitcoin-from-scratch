use std::collections::HashSet;

use crate::{
    block::{BlockErrors, BlockHeader, constants::MAX_BLOCK_SIZE}, merkle::MerkleTree, serialization::BitcoinSerialize, transaction::{OutPoint, Transaction},
};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl BitcoinSerialize for Block {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&self.header.serialize());

        for tx in self.transactions.iter() {
            bytes.extend_from_slice(&tx.serialize());
        }
        bytes
    }
}

impl Block {
    pub fn verify_pow(&self) -> Result<(), BlockErrors> {
        if self.header.verify_pow() {
            return Ok(());
        }
        Err(BlockErrors::InvalidPoW)
    }

    pub fn verify_merkle_root(&self) -> Result<(), BlockErrors> {
        if MerkleTree::compute_root(&self.transactions)
            .map_err(|_| BlockErrors::InvalidMerkleRoot)?
            == self.header.merkle_root
        {
            return Ok(());
        };
        Err(BlockErrors::InvalidMerkleRoot)
    }

    pub fn is_double_spent_safe(&self) -> Result<(), BlockErrors> {
        let mut seen_inputs: HashSet<OutPoint> = HashSet::new();

        for tx in self.transactions.iter() {
            for input in tx.inputs.iter() {
                if !seen_inputs.insert(input.previous_output.clone()) {
                    return Err(BlockErrors::DoubleSpentDetected);
                };
            }
        }
        Ok(())
    }

    pub fn verify_coinbase_order(&self) -> Result<(), BlockErrors> {
        if self.transactions[0].is_coinbase() {
            return Ok(());
        }
        Err(BlockErrors::InvalidTxFormat)
    }

    pub fn is_valid_size(&self) -> Result<(), BlockErrors> {
        if self.serialize().len() <= MAX_BLOCK_SIZE {
            return Ok(());
        }
        Err(BlockErrors::InvalidBlockSize)
    }

    pub fn validate_block(&self) -> Result<(), BlockErrors> {
        self.is_valid_size()?;
        self.verify_coinbase_order()?;
        self.verify_pow()?;
        self.verify_merkle_root()?;
        self.is_double_spent_safe()
    }
}
