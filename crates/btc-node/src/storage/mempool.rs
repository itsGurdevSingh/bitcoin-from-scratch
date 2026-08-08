use std::collections::HashMap;

use btc_core::{
    mempool::MempoolEntry,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::TxId,
};
use redb::{Database, Error, ReadableDatabase, ReadableTable, StorageError};

use crate::storage::tables::MEMPOOL_ENTRY_TABLE;
pub struct MempoolStore<'a> {
    pub db: &'a Database,
}

impl<'a> MempoolStore<'a> {
    pub fn insert(&self, txid: &TxId, mempool_entry: &MempoolEntry) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let entry_bytes = mempool_entry.serialize();
        {
            let mut table = write.open_table(MEMPOOL_ENTRY_TABLE)?;
            table.insert(txid.as_bytes(), entry_bytes.as_slice())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn remove(&self, txid: &TxId) -> Result<Option<MempoolEntry>, Error> {
        let write = self.db.begin_write()?;
        let result: Option<MempoolEntry>;
        {
            let mut table = write.open_table(MEMPOOL_ENTRY_TABLE)?;
            let value = table.remove(txid.as_bytes())?;

            result = match value {
                Some(value) => {
                    let (entry, _) = MempoolEntry::deserialize(value.value())
                        .map_err(|_| StorageError::Corrupted("deserialization failed".into()))?;
                    Some(entry)
                }
                None => None,
            };
        }
        // Commit the transaction before returning
        write.commit()?;

        Ok(result)
    }

    pub fn get(&self, txid: &TxId) -> Result<Option<MempoolEntry>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(MEMPOOL_ENTRY_TABLE)?;
        let value = table.get(txid.as_bytes())?;

        match value {
            Some(value) => {
                let (entry, _) = MempoolEntry::deserialize(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    pub fn get_all(&self) -> Result<Option<HashMap<TxId, MempoolEntry>>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(MEMPOOL_ENTRY_TABLE)?;
        let mut fees_map: HashMap<TxId, MempoolEntry> = HashMap::new();

        for entry in table.iter()? {
            let (key, value) = entry?;
            let txid = TxId(*key.value());
            let (entry, _) = MempoolEntry::deserialize(value.value())
                .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
            fees_map.insert(txid, entry);
        }
        Ok(Some(fees_map))
    }
}
