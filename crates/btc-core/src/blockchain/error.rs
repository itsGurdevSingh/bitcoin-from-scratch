use crate::{
    block::BlockErrors, difficulty::DifficultyErrors, ledger::LedgerError,
    state_transition::ProcessorError as TxProcessorError,
};

#[derive(Debug, PartialEq, Eq)]
pub enum BlockProcessorErrors {
    HasNoTransaction,
    TransactionProcessor(TxProcessorError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockchainError {
    WrongPreviousBlock,
    InvalidHeader,
    ChainIsEmpty,
    Mempool,
    Processor(BlockProcessorErrors),
    Difficulty(DifficultyErrors),
    Ledger(LedgerError),
    Block(BlockErrors),
    InvalidSyntex,
    InvalidScriptFormat,
    OrpanChildfailed,
    FailedOvelayCreation,
}
