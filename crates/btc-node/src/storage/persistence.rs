use btc_core::{
    block::Block, blockchain::BlockNode, mempool::MempoolEntry, presistaence::{DbPersistence, PersistenceError}, transaction::OutPoint, types::{BlockHash, TxId}, utxo::Utxo,
};

use crate::storage::Storage;

impl DbPersistence for Storage {
    fn insert_entry(
        &self,
        txid: &TxId,
        mempool_entry: &MempoolEntry,
    ) -> Result<(), PersistenceError> {
        self.mempool()
            .insert(txid, mempool_entry)
            .map_err(|_| PersistenceError::OprationFaild)
    }
    fn remove_entry(&self, txid: &TxId) -> Result<Option<MempoolEntry>, PersistenceError> {
        self.mempool()
            .remove(txid)
            .map_err(|_| PersistenceError::OprationFaild)
    }

    fn insert_utxo(&self, outpoint: &OutPoint, utxo: &Utxo) -> Result<(), PersistenceError> {
        self.ledger()
            .insert(outpoint, utxo)
            .map_err(|_| PersistenceError::OprationFaild)
    }
    fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, PersistenceError> {
        self.ledger()
            .get(outpoint)
            .map_err(|_| PersistenceError::OprationFaild)
    }
    fn remove_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, PersistenceError> {
        self.ledger()
            .remove(outpoint)
            .map_err(|_| PersistenceError::OprationFaild)
    }


    // chain
    fn insert_node(&self, block_hash: &BlockHash, block_node: &BlockNode, is_active: bool) -> Result<(), PersistenceError> {
        self.block_node().insert(block_hash, block_node, is_active).map_err(|_| PersistenceError::OprationFaild)
    }
    fn get_node(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, PersistenceError> {
       self.block_node().get(block_hash).map_err(|_| PersistenceError::OprationFaild)
    }
    fn remove_node(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, PersistenceError> {
        self.block_node().remove(block_hash).map_err(|_| PersistenceError::OprationFaild)
    }
    fn get_node_by_height(&self, height: u32) ->  Result<Option<BlockNode>, PersistenceError> {
        self.block_node().get_node_by_height(height).map_err(|_| PersistenceError::OprationFaild)
    }

    fn get_orphan_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, PersistenceError> {
        self.block().get(block_hash).map_err(|_| PersistenceError::OprationFaild)
    }
    fn insert_orphan_block(&self, block_hash: &BlockHash, block: &Block) -> Result<(), PersistenceError> {
        self.block().insert(block_hash, block).map_err(|_| PersistenceError::OprationFaild)
    }
    fn remove_orphan_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, PersistenceError> {
        self.block().remove(block_hash).map_err(|_| PersistenceError::OprationFaild)
    }

    fn get_tip(&self) -> Result<Option<BlockHash>, PersistenceError> {
        self.metadata().get_tip().map_err(|_| PersistenceError::OprationFaild)
    }
    fn set_tip(&self, block_hash: &BlockHash) -> Result<(), PersistenceError> {
        self.metadata().set_tip(block_hash).map_err(|_| PersistenceError::OprationFaild)
    }

}
