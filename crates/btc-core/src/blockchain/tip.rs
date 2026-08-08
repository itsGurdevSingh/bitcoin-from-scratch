use std::{
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use crate::{
    presistaence::{DbPersistence, PersistenceError}, types::BlockHash,
};

pub struct Tip<S: DbPersistence> {
    storage: Arc<RwLock<S>>,
    inner: BlockHash,
}

impl<S: DbPersistence> Tip<S> {
    pub fn new(storage: Arc<RwLock<S>>, tip: BlockHash) -> Self {
        Self {
            storage,
            inner: tip,
        }
    }

    pub fn set(
        &mut self,
        block_hash: BlockHash,
    ) -> Option<BlockHash> {
        self.storage_write()
            .ok()?
            .set_tip(&block_hash)
            .ok()?;
        self.inner = block_hash.clone();
        Some(block_hash)
    }

    pub fn get(&self) -> BlockHash {
        self.inner
    }

    fn storage_write(&self) -> Result<RwLockWriteGuard<'_, S>, PersistenceError> {
        self.storage
            .write()
            .map_err(|_| PersistenceError::StoragePoisoned)
    }
}


