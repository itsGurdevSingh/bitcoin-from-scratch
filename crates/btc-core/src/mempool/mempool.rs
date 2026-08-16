use std::{
    collections::{BTreeSet, HashMap, HashSet}, sync::{Arc, RwLock, RwLockWriteGuard},
};

use crate::{
    mempool::{FeeIndex, MEMPOOL_SIZE, MempoolEntry, MempoolError},
    presistaence::{PersistenceError, db_persistence::DbPersistence},
    transaction::{OutPoint, Transaction},
    types::TxId,
};

pub struct Mempool<S: DbPersistence> {
    storage: Arc<RwLock<S>>,
    pub transactions: HashMap<TxId, MempoolEntry>,
    reserved_outpoints: HashSet<OutPoint>,
    pub by_fee_rate: BTreeSet<FeeIndex>,
}

impl<S: DbPersistence> Mempool<S> {
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self {
            storage,
            transactions: HashMap::new(),
            reserved_outpoints: HashSet::new(),
            by_fee_rate: BTreeSet::new(),
        }
    }

    fn storage_write(&self) -> Result<RwLockWriteGuard<'_, S>, PersistenceError> {
        self
            .storage
            .write()
            .map_err(|_| PersistenceError::StoragePoisoned)
        
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

        let vsize = tx.v_bytes();
        self.by_fee_rate.insert(FeeIndex { fee, vsize, txid });

        let entry = MempoolEntry { tx, fee };

        self.storage_write()
        .map_err(|e| MempoolError::Persistence(e))?
        .insert_entry(&txid, &entry).map_err(|e| MempoolError::Persistence(e))?;
        self.transactions.insert(txid, entry);

        return Ok(fee);
    }

    pub fn remove_transaction(&mut self, txid: &TxId) -> Option<MempoolEntry> {
        let _ = self.storage_write().ok()?.remove_entry(txid);

        if let Some(entry) = self.transactions.remove(txid) {
            for input in &entry.tx.inputs {
                self.reserved_outpoints.remove(&input.previous_output);
            }

            let vsize = entry.tx.v_bytes();
            self.by_fee_rate.remove(&FeeIndex {
                fee: entry.fee,
                vsize,
                txid: *txid,
            });

            return Some(entry);
        }
        return None;
    }

    pub fn get_transaction(&self, txid: &TxId) -> Option<&MempoolEntry> {
        self.transactions.get(&txid)
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

    use crate::{ledger::Ledger, presistaence::Store, tests::dummy_tx::get_valid_tx};

    use super::*;

    #[test]
    fn valid_transaction_added() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

        let transaction = get_valid_tx(&mut ledger, 50, 0, 45);

        let store = Store::new();
        let mut mempool = Mempool::new(store);

        let res = mempool.add_transaction(transaction, 2);

        assert_eq!(res, Ok(2));
    }

    #[test]
    fn duplicate_txid_rejected() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

        let transaction = get_valid_tx(&mut ledger, 50, 0, 40);

        let store = Store::new();
        let mut mempool = Mempool::new(store);

        let _res = mempool.add_transaction(transaction.clone(), 2);
        let res2 = mempool.add_transaction(transaction, 2);

        assert_eq!(res2, Err(MempoolError::TransactionAlreadyExists))
    }

    #[test]
    fn double_spend_rejected() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

        let transaction = get_valid_tx(&mut ledger, 50, 0, 45);
        let mut transaction2 = transaction.clone();
        transaction2.outputs[0].value = 40;

        let store = Store::new();
        let mut mempool = Mempool::new(store);

        let _res = mempool.add_transaction(transaction, 2);
        let res2 = mempool.add_transaction(transaction2, 2);

        assert_eq!(res2, Err(MempoolError::DoubleSpendDetected));
        // assert_eq!(res2, Err(MempoolError::ValidationFailed))
    }

    #[test]
    fn remove_transaction_releases_inputs() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

        let transaction = get_valid_tx(&mut ledger, 50, 0, 45);

        let store = Store::new();
        let mut mempool = Mempool::new(store);

        mempool.add_transaction(transaction.clone(), 2).unwrap();

        let txid = transaction.txid();

        let _res = mempool.remove_transaction(&txid);

        assert!(!mempool.contains(&txid))
    }
}
