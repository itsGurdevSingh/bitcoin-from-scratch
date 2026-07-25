use std::collections::{HashMap, HashSet};

use crate::{
    mempool::{MEMPOOL_SIZE, MempoolEntry, MempoolError},
    transaction::{OutPoint, Transaction},
    types::TxId,
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

    pub fn add_transaction(&mut self, tx: Transaction, fee: u64) -> Result<u64, MempoolError> {
        // early exit
        if self.transactions.len() >= MEMPOOL_SIZE {
            return Err(MempoolError::MempoolFull);
        }

        let txid = tx.txid();
        if self.contains(&txid) {
            return Err(MempoolError::TransactionAlreadyExists);
        }

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

#[cfg(test)]

mod test {

    use crate::{
        ledger::Ledger,
        tests::dummy_tx::get_valid_tx,
    };

    use super::*;

    #[test]
    fn valid_transaction_added() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let transaction = get_valid_tx(&mut ledger, 50, 0, 45);

        let mut mempool = Mempool::new();

        let res = mempool.add_transaction(transaction, 2);

        assert_eq!(res, Ok(2));
    }

    #[test]
    fn duplicate_txid_rejected() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let transaction = get_valid_tx(&mut ledger, 50, 0, 40);

        let mut mempool = Mempool::new();

        let _res = mempool.add_transaction(transaction.clone(), 2);
        let res2 = mempool.add_transaction(transaction, 2);

        assert_eq!(res2, Err(MempoolError::TransactionAlreadyExists))
    }

    #[test]
    fn double_spend_rejected() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let transaction = get_valid_tx(&mut ledger, 50, 0, 45);
        let mut transaction2 = transaction.clone();
        transaction2.outputs[0].value = 40;

        let mut mempool = Mempool::new();

        let _res = mempool.add_transaction(transaction, 2);
        let res2 = mempool.add_transaction(transaction2, 2);

        assert_eq!(res2, Err(MempoolError::DoubleSpendDetected));
        // assert_eq!(res2, Err(MempoolError::ValidationFailed))
    }

    #[test]
    fn remove_transaction_releases_inputs() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let transaction = get_valid_tx(&mut ledger, 50, 0, 45);

        let mut mempool = Mempool::new();

        mempool.add_transaction(transaction.clone(), 2).unwrap();

        let txid = transaction.txid();

        let _res = mempool.remove_transaction(&txid);

        assert!(!mempool.contains(&txid))
    }

}
