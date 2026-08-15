use btc_core::blockchain::error::BlockchainError;
use redb::Error;

#[derive(Debug)]
pub enum NodeError {
    Storage(Error),
    LockPoisoned(String),
    Chain(BlockchainError),
}
