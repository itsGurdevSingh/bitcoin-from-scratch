use std::collections::HashMap;

use crate::{transaction::OutPoint, utxo::UtxoError};

use super::Utxo;

/// Represents the current UTXO set maintained by a node.
///
/// The UTXO set is the active state of the Bitcoin ledger.
/// Every spendable coin in the system exists as an entry
/// in this collection.
///
/// Keys are OutPoints:
/// `(txid, vout)`
///
/// Values are the corresponding UTXOs.
///
/// Example:
///
/// (tx100, 0) -> Utxo { ... }
/// (tx100, 1) -> Utxo { ... }
///
/// When a transaction spends an output, the corresponding
/// entry is removed from the UTXO set. New outputs created
/// by the transaction are then inserted.
pub struct UtxoSet {
    inner: HashMap<OutPoint, Utxo>,
}

impl UtxoSet {
    /// Creates a new empty UTXO set.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Adds a UTXO into the set if the OutPoint does not already exist.
    ///
    /// Returns `Ok(())` on success, or `Err(UtxoError::AlreadyExists)` if
    /// an entry for the provided `OutPoint` is already present.
    pub fn add_utxo(&mut self, outpoint: OutPoint, utxo: Utxo) -> Result<(), UtxoError> {
        if self.contains_utxo(&outpoint) {
            return Err(UtxoError::AlreadyExists);
        }

        self.inner.insert(outpoint, utxo);

        Ok(())
    }

    /// Retrieves a reference to the UTXO for the given `OutPoint`.
    ///
    /// Returns `Some(&Utxo)` if found, or `None` if the OutPoint is not present.
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.inner.get(outpoint)
    }

    /// Checks if a UTXO with the given OutPoint exists in the set.
    pub fn contains_utxo(&self, outpoint: &OutPoint) -> bool {
        self.inner.contains_key(outpoint)
    }

    ///Marks a UTXO as spent by removing it
    ///from the active UTXO set.
    ///
    /// Returns `Ok(Utxo)` containing the removed UTXO if present, or
    /// `Err(UtxoError::NotFound)` if no entry existed for the provided `OutPoint`.
    pub fn spend_utxo(&mut self, outpoint: &OutPoint) -> Result<Utxo, UtxoError> {
        self.inner.remove(outpoint).ok_or(UtxoError::NotFound)
    }

    /// Returns the number of UTXOs in the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Checks if the UTXO set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        script::{OpCode, Script, ScriptItem},
        types::TxId,
    };

    use super::*;

    #[test]
    fn insert_and_get_utxo() {
        let mut utxo_set = UtxoSet::new();

        let (outpoint, utxo) = create_dummy_data();

        utxo_set.add_utxo(outpoint.clone(), utxo.clone()).unwrap();

        assert_eq!(utxo_set.get_utxo(&outpoint), Some(&utxo));
    }

    #[test]
    fn contains_existing_utxo() {
        let mut utxo_set = UtxoSet::new();

        let (outpoint, utxo) = create_dummy_data();

        utxo_set.add_utxo(outpoint.clone(), utxo).unwrap();

        assert!(utxo_set.contains_utxo(&outpoint));
    }

    #[test]
    fn spend_returns_removed_utxo() {
        let mut utxo_set = UtxoSet::new();

        let (outpoint, utxo) = create_dummy_data();

        utxo_set.add_utxo(outpoint.clone(), utxo.clone()).unwrap();
        assert!(utxo_set.contains_utxo(&outpoint));

        let spent = utxo_set.spend_utxo(&outpoint).unwrap();
        assert_eq!(spent, utxo);
    }

    #[test]
    fn double_spend_returns_error() {
        let mut utxo_set = UtxoSet::new();

        let (outpoint, utxo) = create_dummy_data();

        utxo_set.add_utxo(outpoint.clone(), utxo).unwrap();
        assert!(utxo_set.contains_utxo(&outpoint));

        utxo_set.spend_utxo(&outpoint).unwrap();

        // spent same utxo again 
        let res = utxo_set.spend_utxo(&outpoint);

        assert_eq!(res,Err(UtxoError::NotFound));
    }

    #[test]
    fn adding_duplicate_utxo_returns_error() {
        let mut utxo_set = UtxoSet::new();

        let (outpoint, utxo) = create_dummy_data();

        utxo_set.add_utxo(outpoint.clone(), utxo.clone()).unwrap();

        // insert for same key again should though error .
        let res = utxo_set.add_utxo(outpoint.clone(), utxo.clone());

        assert_eq!(res, Err(UtxoError::AlreadyExists));
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
            script_pub_key: Script { items: p2pkh_script },
            is_coinbase: false,
            block_height: 1000,
        };

        (outpoint, utxo)
    }
}
