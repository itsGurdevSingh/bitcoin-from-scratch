use crate::{ledger::LedgerError, state_transition::ProcessorError as TxProcessorError};

pub enum BlockProcessorErrors{
    HasNoTransaction,
    TransactionProcessor(TxProcessorError),
}

pub enum BlockchainError {
    ChainIsEmpty,
    WrongPreviousBlock,
    Processor(BlockProcessorErrors),
    Ledger(LedgerError),
    Mempool
}