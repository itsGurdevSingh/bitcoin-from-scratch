use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use btc_core::blockchain::Blockchain;

use crate::{node::NodeError, storage::Storage};

pub struct Node {
    pub chain: Blockchain<Storage>,
}

impl Node {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, NodeError> {
        let storage = Arc::new(RwLock::new(
            Storage::open(path).map_err(NodeError::Storage)?,
        ));
        let mut chain = Blockchain::new(storage).map_err(NodeError::Chain)?;

        // persist the initial tip so a later load can restore the chain from disk
        let genesis_tip = chain.tip.get();
        chain.tip.set(genesis_tip);

        Ok(Self { chain })
    }

    pub fn load_chain(path: impl AsRef<Path>) -> Result<Self, NodeError> {
        let path = path.as_ref().to_path_buf();
        let storage = Arc::new(RwLock::new(
            Storage::open(&path).map_err(NodeError::Storage)?,
        ));

        let persisted_tip = {
            let read_guard = storage
                .read()
                .map_err(|_| NodeError::LockPoisoned("metadata lock poisoned".to_string()))?;
            read_guard
                .metadata()
                .get_tip()
                .map_err(NodeError::Storage)?
        };

        if let Some(saved_tip) = persisted_tip {
            let mempool_entries = {
                let read_guard = storage
                    .read()
                    .map_err(|_| NodeError::LockPoisoned("mempool lock poisoned".to_string()))?;
                read_guard.mempool().get_all().map_err(NodeError::Storage)?
            };

            let mut chain = Blockchain::new(storage.clone()).map_err(NodeError::Chain)?;

            if let Some(entries) = mempool_entries {
                for entry in entries.values() {
                    chain
                        .mempool
                        .add_transaction(entry.tx.clone(), entry.fee)
                        .map_err(|_| {
                            NodeError::Chain(btc_core::blockchain::error::BlockchainError::Mempool)
                        })?;
                }
            }

            // preserve the persisted tip before the newly created chain replaces it with genesis
            chain.tip.set(saved_tip);

            Ok(Self { chain })
        } else {
            Self::new(path)
        }
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
