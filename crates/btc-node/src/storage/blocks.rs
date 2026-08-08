use btc_core::{
    block::Block,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::BlockHash,
};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::BLOCKS_TABLE;

pub struct BlockStore<'a> {
    pub db: &'a Database,
}

impl<'a> BlockStore<'a> {
    pub fn insert(&self, block_hash: &BlockHash, block: &Block) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let block_serialized = block.serialize();
        {
            let mut table = write.open_table(BLOCKS_TABLE)?;
            table.insert(block_hash.as_bytes(), block_serialized.as_slice())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get(&self, block_hash: &BlockHash) -> Result<Option<Block>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(BLOCKS_TABLE)?;
        let value = table.get(block_hash.as_bytes())?;

        match value {
            Some(value) => {
                let (block, _) = Block::deserialize(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    pub fn remove(&self, block_hash: &BlockHash) -> Result<Option<Block>, Error> {
        let write = self.db.begin_write()?;

        let result = {
            let mut table = write.open_table(BLOCKS_TABLE)?;

            match table.remove(block_hash.as_bytes())? {
                Some(value) => {
                    let (entry, _) = Block::deserialize(value.value())
                        .map_err(|_| StorageError::Corrupted("deserialization failed".into()))?;
                    Some(entry)
                }
                None => None,
            }
        };
        write.commit()?;
        Ok(result)
    }
}
