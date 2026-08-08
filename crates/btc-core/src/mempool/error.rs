use crate::presistaence::PersistenceError;

#[derive(Debug, PartialEq, Eq)]
pub enum MempoolError {
    ValidationFailed,
    TransactionAlreadyExists,
    DoubleSpendDetected,
    MempoolFull,
    FeeTooLow,
    EntryCrupted,

    Persistence(PersistenceError),
}
