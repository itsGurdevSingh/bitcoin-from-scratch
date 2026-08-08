use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    presistaence::{DbPersistence, PersistenceError}, state_transition::StateTransition, transaction::OutPoint, utxo::{Utxo, UtxoSet},
};

use super::LedgerError;

/// Represents the current spendable state of the blockchain.
///
/// The Ledger owns the UTXO set and is responsible for
/// applying state transitions as transactions are processed.
#[derive(Debug)]
pub struct Ledger<S: DbPersistence> {
    pub storage: Arc<RwLock<S>>,
    pub utxo_set: RwLock<UtxoSet>,
}

impl<S: DbPersistence> Ledger<S> {
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self {
            storage,
            utxo_set: RwLock::new(UtxoSet::new()),
        }
    }

    pub fn add_utxo(&mut self, outpoint: OutPoint, utxo: Utxo) -> Result<(), LedgerError> {
        self.storage_write().map_err(|e| LedgerError::Persistence(e))?
        .insert_utxo(&outpoint, &utxo).map_err(|e| LedgerError::Persistence(e))?;
        self.utxo_set.write().map_err(|_| LedgerError::MutexError)?
        .add_utxo(outpoint, utxo)
        .map_err(LedgerError::Utxo)
    }

    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<Utxo> {

    if let Some(utxo) = self.utxo_set.read().unwrap().get_utxo(outpoint) {
        return Some(utxo.clone());
    }

    let utxo = self.storage_read()
    .ok()?.get_utxo(outpoint).ok()??;

    self.utxo_set
        .write()
        .unwrap()
        .add_utxo(outpoint.clone(), utxo.clone())
        .ok()?;

    Some(utxo)
}

    pub fn spend_utxo(&mut self, outpoint: &OutPoint) -> Result<Utxo, LedgerError> {
        self.storage_write()
        .map_err(|e| LedgerError::Persistence(e))?
        .remove_utxo(outpoint).map_err(|e| LedgerError::Persistence(e))?;
        self.utxo_set.write().map_err(|_| LedgerError::MutexError)?
            .spend_utxo(outpoint)
            .map_err(LedgerError::Utxo)
    }
    pub fn contains_utxo(&self, outpoint: &OutPoint) -> bool {
        let utxos = self.utxo_set.read().unwrap();
        utxos.contains_utxo(outpoint)
    }

    pub fn commit_state(&mut self, state: &StateTransition) -> Result<(), LedgerError> {
        for spent_utxo in state.spent_utxos.iter() {
            self.spend_utxo(&spent_utxo.outpoint)?;
        }

        for created_utxo in state.created_utxos.iter() {
            self.add_utxo(created_utxo.outpoint.clone(), created_utxo.utxo.clone())?;
        }

        Ok(())
    }
    pub fn rollback_state(&mut self, state: &StateTransition) -> Result<(), LedgerError> {
        for spent_utxo in state.spent_utxos.iter() {
            self.add_utxo(spent_utxo.outpoint.clone(), spent_utxo.utxo.clone())?;
        }

        for created_utxo in state.created_utxos.iter() {
            self.spend_utxo(&created_utxo.outpoint)?;
        }

        Ok(())
    }


    
    fn storage_write(&self) -> Result<RwLockWriteGuard<'_, S>, PersistenceError> {
        self
            .storage
            .write()
            .map_err(|_| PersistenceError::StoragePoisoned)
        
    }
    fn storage_read(&self) -> Result<RwLockReadGuard<'_, S>, PersistenceError> {
        self
            .storage
            .read()
            .map_err(|_| PersistenceError::StoragePoisoned)
        
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        presistaence::Store,
        script::{OpCode, Script, ScriptItem},
        types::TxId,
        utxo::UtxoError,
    };

    use super::*;

    #[test]
    fn add_utxo_via_ledger() {
        let mut ledger = Ledger::new(Store::new());

        let (outpoint, utxo) = create_dummy_data();

        let res = ledger.add_utxo(outpoint, utxo);

        assert_eq!(res, Ok(()))
    }

    #[test]
    fn get_utxo_via_ledger() {
        let mut ledger = Ledger::new(Store::new());

        let (outpoint, utxo) = create_dummy_data();

        ledger.add_utxo(outpoint.clone(), utxo.clone()).unwrap();

        let res = ledger.get_utxo(&outpoint);

        assert_eq!(res, Some(utxo))
    }

    #[test]
    fn spend_utxo_via_ledger() {
        let mut ledger = Ledger::new(Store::new());

        let (outpoint, utxo) = create_dummy_data();

        ledger.add_utxo(outpoint.clone(), utxo.clone()).unwrap();

        let res = ledger.spend_utxo(&outpoint);

        assert_eq!(res, Ok(utxo))
    }

    #[test]
    fn double_spend_returns_error() {
        let mut ledger = Ledger::new(Store::new());

        let (outpoint, utxo) = create_dummy_data();

        ledger.add_utxo(outpoint.clone(), utxo.clone()).unwrap();

        ledger.spend_utxo(&outpoint).unwrap();

        // double spent
        let res = ledger.spend_utxo(&outpoint);

        assert_eq!(res, Err(LedgerError::Utxo(UtxoError::NotFound)));
    }

    // helper function
    fn create_dummy_data() -> (OutPoint, Utxo) {
        let outpoint: OutPoint = OutPoint {
            txid: TxId([1u8; 32]),
            vout: 0,
        };

        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(vec![0u8; 20]), // 20-byte dummy pubkey hash
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        let utxo: Utxo = Utxo {
            value: 10,
            script_pub_key: Script {
                items: p2pkh_script,
            },
            is_coinbase: false,
            block_height: 1000,
        };

        (outpoint, utxo)
    }
}
