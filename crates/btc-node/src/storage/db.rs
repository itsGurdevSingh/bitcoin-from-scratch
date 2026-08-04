use std::{fs, path::Path};

use redb::Database;

use crate::storage::{
    block_node_metadata::BlockMetadataStore, blocks::BlockStore, headers::HeaderStore,
    mempool::MempoolStore, metadata::MetadataStore, transactions::TransactionStore,
    utxos::UtxoStore,
};

pub struct Storage {
    pub db: Database,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, redb::DatabaseError> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let db = if path.exists() {
            redb::Database::open(path)?
        } else {
            redb::Database::create(path)?
        };

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
    pub fn block_node(&self) -> BlockMetadataStore<'_> {
        BlockMetadataStore { db: &self.db }
    }
    pub fn ledger(&self) -> UtxoStore<'_> {
        UtxoStore { db: &self.db }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op() {
        let _storage = Storage::open("./data/node.redb").unwrap();
    }
}
