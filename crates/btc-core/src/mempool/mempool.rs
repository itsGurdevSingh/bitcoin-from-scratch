use std::collections::{HashMap, HashSet};

use crate::{
    ledger::Ledger,
    mempool::{MEMPOOL_SIZE, MempoolEntry, MempoolError},
    transaction::{OutPoint, Transaction},
    types::TxId,
    validator::TransactionValidator,
};

pub struct Mempool {
    transactions: HashMap<TxId, MempoolEntry>,
    reserved_outpoints: HashSet<OutPoint>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            reserved_outpoints: HashSet::new(),
        }
    }

    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        ledger: &Ledger,
    ) -> Result<u64, MempoolError> {
        // early exit
        if self.transactions.len() >= MEMPOOL_SIZE {
            return Err(MempoolError::MempoolFull);
        }

        let txid = tx.txid();
        if self.contains(&txid) {
            return Err(MempoolError::TransactionAlreadyExists);
        }

        let fee = TransactionValidator::validate(&tx, ledger)
            .map_err(|_| MempoolError::ValidationFailed)?;

         // test run for error exit not impact data (save for rollback)
        for input in &tx.inputs {
            if self.reserved_outpoints.contains(&input.previous_output) {
                return Err(MempoolError::DoubleSpendDetected);
            }
        }
        for input in &tx.inputs {
            self.reserved_outpoints
                .insert(input.previous_output.clone());
        }

        let entry = MempoolEntry { tx, fee };

        self.transactions.insert(txid, entry);

        return Ok(fee);
    }

    pub fn remove_transaction(&mut self, txid: &TxId) -> Option<MempoolEntry> {
        if let Some(entry) = self.transactions.remove(txid) {
            for input in &entry.tx.inputs {
                self.reserved_outpoints.remove(&input.previous_output);
            }

            return Some(entry);
        }
        return None;
    }

    pub fn contains(&self, txid: &TxId) -> bool {
        self.transactions.contains_key(txid)
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }
}
