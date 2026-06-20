use crate::{
    transaction::OutPoint,
    utxo::{Utxo, UtxoSet},
};

use super::LedgerError;

/// Represents the current spendable state of the blockchain.
///
/// The Ledger owns the UTXO set and is responsible for
/// applying state transitions as transactions are processed.
pub struct Ledger {
    utxo_set: UtxoSet,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            utxo_set: UtxoSet::new(),
        }
    }

    pub fn add_utxo(
        &mut self,
        outpoint: OutPoint,
        utxo: Utxo,
    ) -> Result<(), LedgerError> {
        self.utxo_set
            .add_utxo(outpoint, utxo)
            .map_err(LedgerError::Utxo)
    }

    pub fn get_utxo(
        &self,
        outpoint: &OutPoint,
    ) -> Option<&Utxo> {
        self.utxo_set.get_utxo(outpoint)
    }

    pub fn spend_utxo(
        &mut self,
        outpoint: &OutPoint,
    ) -> Result<Utxo, LedgerError> {
        self.utxo_set
            .spend_utxo(outpoint)
            .map_err(LedgerError::Utxo)
    }
}