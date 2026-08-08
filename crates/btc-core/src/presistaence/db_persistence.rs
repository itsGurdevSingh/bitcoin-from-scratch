use crate::{
    block::Block, blockchain::BlockNode, mempool::MempoolEntry, presistaence::PersistenceError, transaction::OutPoint, types::{BlockHash, TxId}, utxo::Utxo,
};

pub trait DbPersistence {
    fn insert_entry(
        &self,
        txid: &TxId,
        mempool_entry: &MempoolEntry,
    ) -> Result<(), PersistenceError>;

    fn remove_entry(&self, txid: &TxId) -> Result<Option<MempoolEntry>, PersistenceError>;

    fn insert_utxo(&self, outpoint: &OutPoint, utxo: &Utxo) -> Result<(), PersistenceError>;
    fn remove_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, PersistenceError>;
    fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, PersistenceError>;

    //chain 
    //orphan block
    fn insert_orphan_block(&self, block_hash: &BlockHash, block: &Block) -> Result<(), PersistenceError>;
    fn get_orphan_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, PersistenceError>;
    fn remove_orphan_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, PersistenceError>;

    // node
    fn insert_node(&self, block_hash: &BlockHash, block_node: &BlockNode, is_active: bool) -> Result<(), PersistenceError>;
    fn get_node(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, PersistenceError>;
    fn remove_node(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, PersistenceError>;
    fn get_node_by_height(&self, height: u32) ->  Result<Option<BlockNode>, PersistenceError>;

    //tip
    fn set_tip(&self, block_hash: &BlockHash) -> Result<(), PersistenceError>;
    fn get_tip(&self) -> Result<Option<BlockHash>, PersistenceError>;

}
