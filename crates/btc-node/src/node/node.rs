use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use btc_core::{
    block::constants::{MAX_BLOCK_SIZE, MIN_STANDARD_TX_VBYTES},
    blockchain::{Blockchain, Nodes, OrphanBlocks, Tip},
    ledger::Ledger,
    mempool::{Mempool, MempoolError},
    presistaence::DbPersistence,
    transaction::Transaction,
    types::TxId,
    validator::TransactionValidator,
};

use crate::{node::NodeError, storage::Storage};

pub struct Node {
    pub chain: Blockchain<Storage>,
}

impl Node {
    fn open_storage(path: impl AsRef<Path>) -> Result<Arc<RwLock<Storage>>, NodeError> {
        Storage::open(path)
            .map(|storage| Arc::new(RwLock::new(storage)))
            .map_err(NodeError::Storage)
    }

    fn read_persisted_tip(
        storage: &Arc<RwLock<Storage>>,
    ) -> Result<Option<btc_core::types::BlockHash>, NodeError> {
        let read_guard = storage
            .read()
            .map_err(|_| NodeError::LockPoisoned("metadata lock poisoned".to_string()))?;

        read_guard.metadata().get_tip().map_err(NodeError::Storage)
    }

    fn restore_mempool(
        mempool: &mut Mempool<Storage>,
        storage: &Arc<RwLock<Storage>>,
    ) -> Result<(), NodeError> {
        let mempool_entries = {
            let read_guard = storage
                .read()
                .map_err(|_| NodeError::LockPoisoned("mempool lock poisoned".to_string()))?;
            read_guard.mempool().get_all().map_err(NodeError::Storage)?
        };

        if let Some(entries) = mempool_entries {
            for entry in entries.values() {
                mempool
                    .add_transaction(entry.tx.clone(), entry.fee)
                    .map_err(|_| {
                        NodeError::Chain(btc_core::blockchain::error::BlockchainError::Mempool)
                    })?;
            }
        }

        Ok(())
    }

    pub fn new(path: impl AsRef<Path>) -> Result<Self, NodeError> {
        let storage = Self::open_storage(path)?;
        let mut chain = Blockchain::new(storage).map_err(NodeError::Chain)?;

        // keep the current genesis tip in memory so a future reload can restore it
        let genesis_tip = chain.tip.get();
        chain.tip.set(genesis_tip);

        Ok(Self { chain })
    }

    pub fn load_chain(path: impl AsRef<Path>) -> Result<Self, NodeError> {
        let path = path.as_ref().to_path_buf();
        let storage = Self::open_storage(&path)?;

        let persisted_tip = Self::read_persisted_tip(&storage)?;

        if let Some(saved_tip) = persisted_tip {
            let ledger = Ledger::new(storage.clone());
            let nodes = Nodes::new(storage.clone());

            let mut mempool = Mempool::new(storage.clone());

            // restore the saved mempool entries for rebuilding the chain structure
            Self::restore_mempool(&mut mempool, &storage)?;

            let chain = Blockchain {
                tip: Tip::new(storage.clone(), saved_tip),
                nodes,
                orphan_blocks: OrphanBlocks::new(storage.clone()),
                ledger,
                mempool,
            };

            Ok(Self { chain })
        } else {
            Self::new(path)
        }
    }

    // get best fee rate tx for mining
    pub fn get_mining_txs(&mut self) -> Result<Vec<Transaction>, NodeError> {
        let mut total_bytes = 0;
        let mut txs: Vec<Transaction> = Vec::new();
        let mut invalid_txids: Vec<TxId> = Vec::new();

        for fee_index in self.chain.mempool.by_fee_rate.iter() {
            let entry = self
                .chain
                .mempool
                .transactions
                .get(&fee_index.txid)
                .ok_or(NodeError::Mempool(MempoolError::EntryCrupted))?;
            let vsize = entry.tx.v_bytes();

            if self.validate_transaction(&entry.tx).is_err() {
                // some time due to reorg our mempool entries not belong to our active chain we have to filter that our.
                invalid_txids.push(fee_index.txid);
                continue;
            }

            // header contain 84 and 4 for comapct size total 88 and we keep 12 as grace total 100 bytes;
            if (total_bytes + vsize) > (MAX_BLOCK_SIZE - 100) {
                continue;
            };
            total_bytes += vsize;
            let remaining = MAX_BLOCK_SIZE - total_bytes;

            if remaining < MIN_STANDARD_TX_VBYTES {
                break;
            }

            txs.push(entry.tx.clone());
        }

        // remove invalid tx which not belong to our active chain
        // may added due to reorg.
        for txid in invalid_txids {
            self.chain.mempool.remove_transaction(&txid);
        }

        Ok(txs)
    }

    fn validate_transaction(&self, transaction: &Transaction) -> Result<u64, NodeError> {
        let tip_node = self.chain.tip_node().map_err(NodeError::Chain)?;

        let overlay = self
            .chain
            .create_overlay(tip_node.hash)
            .ok_or(NodeError::OverlayNotFound)?;

        TransactionValidator::validate(&transaction, &self.chain.ledger, &overlay, tip_node.height)
            .map_err(NodeError::Validation)
    }

    pub fn submit_transaction<S: DbPersistence>(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), NodeError> {
        let fee = self.validate_transaction(&transaction)?;
        self.chain
            .mempool
            .add_transaction(transaction, fee)
            .map_err(NodeError::Mempool)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn create_node() {
        let path = env::temp_dir().join(format!(
            "btc-node-genesis-{}-{}.redb",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let _ = fs::remove_file(&path);

        let node = Node::new(&path).expect("node should initialize a blockchain");

        let tip = node.chain.tip_node().expect("genesis tip should exist");
        assert_eq!(
            node.chain.height(),
            0,
            "chain should start at genesis height 0"
        );
        assert!(tip.parent.is_none(), "genesis block should have no parent");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_chain_preserves_persisted_tip() {
        let path = env::temp_dir().join(format!(
            "btc-node-load-{}-{}.redb",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let _ = fs::remove_file(&path);

        let expected_tip = {
            let node = Node::new(&path).expect("node should initialize a blockchain");
            let tip = node.chain.tip.get();
            drop(node);
            tip
        };

        let loaded = Node::load_chain(&path).expect("existing chain should be loaded");
        assert_eq!(
            loaded.chain.tip.get(),
            expected_tip,
            "tip should be preserved on reload"
        );

        let _ = fs::remove_file(&path);
    }
}
