use crate::{
    crypto::sha256d, serialization::BitcoinSerialize, transaction::Transaction
};


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrecomputedTransactionData {
    pub hash_prevouts: [u8; 32],
    pub hash_sequence: [u8; 32],
    pub hash_outputs: [u8; 32],
}

impl PrecomputedTransactionData {
    pub fn new(tx: &Transaction) -> Self {
        Self {
            hash_prevouts: tx.hash_prevouts(),
            hash_sequence: tx.hash_sequence(),
            hash_outputs: tx.hash_outputs(),
        }
    }
}

impl Transaction {
    pub fn hash_prevouts(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        for input in self.inputs.iter() {
            bytes.extend(input.previous_output.serialize());
        }
        sha256d(&bytes)
    }
    pub fn hash_sequence(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        for input in self.inputs.iter() {
            bytes.extend(input.sequence.to_le_bytes());
        }
        sha256d(&bytes)
    }
    pub fn hash_outputs(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        for output in self.outputs.iter() {
            bytes.extend(output.serialize());
        }
        sha256d(&bytes)
    }
}
