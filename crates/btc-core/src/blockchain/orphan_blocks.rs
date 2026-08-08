use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    block::Block, presistaence::{DbPersistence, PersistenceError}, types::BlockHash,
};

pub struct OrphanBlocks<S: DbPersistence> {
    storage: Arc<RwLock<S>>,
    inner: RwLock<HashMap<BlockHash, Block>>,
}

impl<S: DbPersistence> OrphanBlocks<S> {
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self {
            storage,
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(
        &mut self,
        block_hash: BlockHash,
        block: Block,
    ) -> Option<Block> {
        self.storage_write()
            .ok()?
            .insert_orphan_block(&block_hash, &block)
            .ok()?;
        self.inner.write().ok()?.insert(block_hash, block)
    }

    pub fn get(&self, block_hash: &BlockHash) -> Option<Block> {
        if let Some(block) = self.inner.read().unwrap().get(block_hash) {
            return Some(block.clone());
        }

        let block = self.storage_read().ok()?.get_orphan_block(block_hash).ok()??;

        self.inner
            .write()
            .unwrap()
            .insert(block_hash.clone(), block.clone())?;

        Some(block)
    }

    pub fn remove(&self, block_hash: &BlockHash) -> Option<Block>{
        let _ = self.storage_write().ok()?.remove_orphan_block(block_hash).ok()?;
        self.inner.write().ok()?.remove(block_hash)
    }

    fn storage_write(&self) -> Result<RwLockWriteGuard<'_, S>, PersistenceError> {
        self.storage
            .write()
            .map_err(|_| PersistenceError::StoragePoisoned)
    }
    fn storage_read(&self) -> Result<RwLockReadGuard<'_, S>, PersistenceError> {
        self.storage
            .read()
            .map_err(|_| PersistenceError::StoragePoisoned)
    }
}


