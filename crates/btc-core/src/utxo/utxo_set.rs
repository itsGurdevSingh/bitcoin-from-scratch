use std::collections::HashMap;

use crate::transaction::OutPoint;

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
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, outpoint: OutPoint, utxo: Utxo) -> Option<Utxo> {
        self.inner.insert(outpoint, utxo)
    }

    pub fn get(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.inner.get(outpoint)
    }

    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.inner.contains_key(outpoint)
    }

    pub fn spend(&mut self, outpoint: &OutPoint) -> Option<Utxo> {
        self.inner.remove(outpoint)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

}
