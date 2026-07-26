use crate::blockchain::error::BlockchainError;

#[derive(Debug, PartialEq, Eq)]
pub enum BuilderErrors {
    Chain(BlockchainError),
    InvalidMerkleRoot,
    InvalidTxs
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockErrors {
    InvalidBlockSize,
    InvalidBlockWeight,
    InvalidPoW,
    InvalidMerkleRoot,
    InvalidTxFormat,
    DoubleSpentDetected
}