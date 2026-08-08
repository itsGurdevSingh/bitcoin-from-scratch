use std::{fs, path::Path};

use redb::Database;

use crate::storage::{
    block_node::BlockNodeStore,
    blocks::BlockStore,
    headers::HeaderStore,
    mempool::MempoolStore,
    metadata::MetadataStore,
    tables::{MEMPOOL_ENTRY_TABLE, UTXOS_TABLE},
    transactions::TransactionStore,
    utxos::UtxoStore,
};

pub struct Storage {
    pub db: Database,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let db = if path.exists() {
            redb::Database::open(path)?
        } else {
            redb::Database::create(path)?
        };

        {
            let write = db.begin_write()?;
            {
                let _ = write.open_table(MEMPOOL_ENTRY_TABLE)?;
                let _ = write.open_table(UTXOS_TABLE)?;
                // add all your other tables here
            }
            write.commit()?;
        }

        Ok(Self { db })
    }

    pub fn metadata(&self) -> MetadataStore<'_> {
        MetadataStore { db: &self.db }
    }
    pub fn mempool(&self) -> MempoolStore<'_> {
        MempoolStore { db: &self.db }
    }
    pub fn block(&self) -> BlockStore<'_> {
        BlockStore { db: &self.db }
    }
    pub fn transaction(&self) -> TransactionStore<'_> {
        TransactionStore { db: &self.db }
    }
    pub fn header(&self) -> HeaderStore<'_> {
        HeaderStore { db: &self.db }
    }
    pub fn block_node(&self) -> BlockNodeStore<'_> {
        BlockNodeStore { db: &self.db }
    }
    pub fn ledger(&self) -> UtxoStore<'_> {
        UtxoStore { db: &self.db }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        println,
        sync::{Arc, RwLock},
    };

    use btc_core::{mempool::Mempool, transaction::Transaction};

    use super::*;

    #[test]
    fn op() {
        let _storage = Storage::open("./data/node.redb").unwrap();
    }

    #[test]
    fn get_mempool() {
        let storage = Storage::open("./data/node.redb").unwrap();

        let s = Arc::new(RwLock::new(storage));
        let mut mempool: Mempool<Storage> = Mempool::new(s.clone());
        let _ = mempool.add_transaction(Transaction::new(), 40);

        let a = s.read().unwrap().mempool().get_all().unwrap().unwrap();

        for i in a {
            println!("key is ,{:?}", i.0);
        }
    }
}
