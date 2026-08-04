use btc_core::{
    block::BlockHeader,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::BlockHash,
};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::BLOCKS_HEADERS_TABLE;

pub struct HeaderStore<'a> {
    pub db: &'a Database,
}

impl<'a> HeaderStore<'a> {
    pub fn insert_header(
        &self,
        block_hash: &BlockHash,
        block_header: &BlockHeader,
    ) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let block_header_serialized = block_header.serialize();
        {
            let mut table = write.open_table(BLOCKS_HEADERS_TABLE)?;
            table.insert(block_hash.as_bytes(), block_header_serialized.as_slice())?;
        };

        write.commit()?;
        Ok(())
    }

    pub fn get_header(&self, block_hash: &BlockHash) -> Result<Option<BlockHeader>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(BLOCKS_HEADERS_TABLE)?;

        let value = table.get(block_hash.as_bytes())?;

        match value {
            Some(value) => {
                let (block_header, _) = BlockHeader::deserialize(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(block_header))
            }
            None => Ok(None),
        }
    }
}
