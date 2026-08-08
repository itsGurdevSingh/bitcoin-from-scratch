use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    blockchain::BlockNode,
    presistaence::{DbPersistence, PersistenceError},
    types::BlockHash,
};

pub struct Nodes<S: DbPersistence> {
    storage: Arc<RwLock<S>>,
    inner: RwLock<HashMap<BlockHash, BlockNode>>,
}

impl<S: DbPersistence> Nodes<S> {
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self {
            storage,
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(
        &mut self,
        block_hash: BlockHash,
        block_node: BlockNode,
        is_active: bool,
    ) -> Option<BlockNode> {
        self.storage_write()
            .ok()?
            .insert_node(&block_hash, &block_node, is_active)
            .ok()?;
        self.inner.write().ok()?.insert(block_hash, block_node)
    }

    pub fn get(&self, block_hash: &BlockHash) -> Option<BlockNode> {
        if let Some(node) = self.inner.read().unwrap().get(block_hash) {
            return Some(node.clone());
        }

        let block_node = self.storage_read().ok()?.get_node(block_hash).ok()??;

        self.inner
            .write()
            .unwrap()
            .insert(block_hash.clone(), block_node.clone())?;

        Some(block_node)
    }

    pub fn get_by_height(&self, height: u32) -> Option<BlockNode> {
        self.storage_read().ok()?.get_node_by_height(height).ok()?
    }

    pub fn remove(&self, block_hash: &BlockHash) -> Option<BlockNode>{
        let _ = self.storage_write().ok()?.remove_node(block_hash).ok()?;
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


