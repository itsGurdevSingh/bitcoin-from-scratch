use btc_core::{
    block::Block,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::BlockHash,
};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::{BLOCKS_TABLE, HEIGHT_INDEX_TABLE};

pub struct BlockStore<'a> {
    pub db: &'a Database,
}

impl<'a> BlockStore<'a> {
    pub fn insert_block(&self, block_hash: &BlockHash, block: &Block) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let block_serialized = block.serialize();
        {
            let mut table = write.open_table(BLOCKS_TABLE)?;
            table.insert(block_hash.as_bytes(), block_serialized.as_slice())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_block(&self, block_hash: &BlockHash) -> Result<Option<Block>, Error> {
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

    pub fn insert_block_height(&self, height: u32, block_hash: &BlockHash) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        {
            let mut table = write.open_table(HEIGHT_INDEX_TABLE)?;
            table.insert(height, block_hash.as_bytes())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_block_by_height(&self, height: u32) -> Result<Option<Block>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(HEIGHT_INDEX_TABLE)?;
        let value = table.get(height)?;

        match value {
            Some(value) => {
                let block_hash = BlockHash(*value.value());
                self.get_block(&block_hash)
            }
            None => Ok(None),
        }
    }
}
