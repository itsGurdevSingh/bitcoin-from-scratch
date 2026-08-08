use btc_core::types::BlockHash;
use redb::{Database, Error, ReadableDatabase};

use crate::storage::tables::METADATA;

pub struct MetadataStore<'a> {
    pub db: &'a Database,
}

impl<'a> MetadataStore<'a> {
    pub fn set_tip(&self, block_hash: &BlockHash) -> Result<(), Error> {
        self.set_metadata("tip", block_hash.as_bytes())
    }
    pub fn get_tip(&self) -> Result<Option<BlockHash>, Error> {
        if let Some(tip_bytes) = self.get_metadata("tip")? {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&tip_bytes);
            let a = BlockHash(bytes);
        }
        Ok(None)
    }

    pub fn set_metadata(&self, key: &str, value: &[u8]) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        {
            let mut table = write.open_table(METADATA)?;
            table.insert(key, value)?;
        }
        write.commit()?;

        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        let read = self.db.begin_read()?;

        let table = read.open_table(METADATA)?;

        if let Some(value) = table.get(key)? {
            Ok(Some(value.value().to_vec()))
        } else {
            Ok(None)
        }
    }
}
