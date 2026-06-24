use crate::transaction::Transaction;

pub struct MempoolEntry {
    pub tx: Transaction,
    pub fee: u64,
}