use crate::utxo::UtxoError;

#[derive(Debug, PartialEq, Eq)]
pub enum LedgerError {
    Utxo(UtxoError),
}