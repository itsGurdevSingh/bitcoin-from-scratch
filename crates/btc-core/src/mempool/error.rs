pub enum MempoolError {
    ValidationFailed,
    TransactionAlreadyExists,
    DoubleSpendDetected,
    MempoolFull,
    FeeTooLow,
}
