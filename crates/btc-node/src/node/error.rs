use btc_core::{
    blockchain::error::BlockchainError, mempool::MempoolError, validator::ValidationError,
};
use redb::Error;

#[derive(Debug)]
pub enum NodeError {
    Storage(Error),
    LockPoisoned(String),
    Chain(BlockchainError),
    Validation(ValidationError),
    Mempool(MempoolError),
    OverlayNotFound,
}
