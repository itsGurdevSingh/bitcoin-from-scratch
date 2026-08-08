use crate::{presistaence::PersistenceError, utxo::UtxoError};

#[derive(Debug, PartialEq, Eq)]
pub enum LedgerError {
    Utxo(UtxoError),
    Persistence(PersistenceError),
    MutexError
}