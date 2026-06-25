use crate::{block::BlockHeader, transaction::Transaction};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}