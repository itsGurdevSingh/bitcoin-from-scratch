use std::sync::{Arc, RwLock};

use crate::{
    block::Block, blockchain::BlockNode, mempool::MempoolEntry, presistaence::{DbPersistence, PersistenceError}, transaction::OutPoint, types::{BlockHash, TxId}, utxo::Utxo,
};

pub struct Store {
    db: Vec<u8>,
}

impl Store {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self { db: Vec::new() }))
    }
}
impl DbPersistence for Store {
    fn insert_entry(
        &self,
        txid: &TxId,
        mempool_entry: &MempoolEntry,
    ) -> Result<(), PersistenceError> {
        _ = self.db;
        _ = txid;
        _ = mempool_entry;
        Ok(())
    }

    fn remove_entry(&self, txid: &TxId) -> Result<Option<MempoolEntry>, PersistenceError> {
        let _ = txid;
        Ok(Some(MempoolEntry::new()))
    }

    fn insert_utxo(&self, outpoint: &OutPoint, utxo: &Utxo) -> Result<(), PersistenceError> {
        _ = outpoint;
        _ = utxo;
        Ok(())
    }
    fn remove_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, PersistenceError> {
        _ = outpoint;
        Ok(Some(Utxo::new()))
    }
    fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, PersistenceError> {
        _ = outpoint;
        Ok(None)
    }

    fn insert_node(&self, block_hash: &BlockHash, block_node: &BlockNode, is_active: bool) -> Result<(), PersistenceError> {
        _ = block_hash;
        _ = block_node;
        _ = is_active;
        Ok(())
    }
    fn get_node(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, PersistenceError> {
        _ = block_hash;
        Ok(None)
    }
    fn remove_node(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, PersistenceError> {
        _ = block_hash;
        Ok(None)
    }
    fn get_node_by_height(&self, height: u32) ->  Result<Option<BlockNode>, PersistenceError> {
        _ = height;
        Ok(None)
    }

    fn insert_orphan_block(&self, block_hash: &BlockHash, block: &Block) -> Result<(), PersistenceError> {
        _ = block_hash;
        _ = block;
        Ok(())
    }
    fn get_orphan_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, PersistenceError> {
        _ = block_hash;
        Ok(None)
    }
    fn remove_orphan_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, PersistenceError> {
        _ = block_hash;
        Ok(None)
    }

    fn get_tip(&self) -> Result<Option<BlockHash>, PersistenceError> {
        Ok(None)
    }
    fn set_tip(&self, block_hash: &BlockHash) -> Result<(), PersistenceError> {
        _ = block_hash;
        Ok(())
    }
}
