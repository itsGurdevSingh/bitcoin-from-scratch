use crate::{difficulty::DifficultyErrors, ledger::LedgerError, state_transition::ProcessorError as TxProcessorError};

#[derive(Debug, PartialEq, Eq)]
pub enum BlockProcessorErrors{
    HasNoTransaction,
    TransactionProcessor(TxProcessorError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockchainError {
    ChainIsEmpty,
    WrongPreviousBlock,
    Processor(BlockProcessorErrors),
    Ledger(LedgerError),
    Mempool,
    Difficulty(DifficultyErrors)
}