use btc_core::{
    blockchain::BlockNode,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::BlockHash,
};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::{BLOCK_NODE_TABLE, HEIGHT_INDEX_TABLE};
pub struct BlockNodeStore<'a> {
    pub db: &'a Database,
}

impl<'a> BlockNodeStore<'a> {
    pub fn insert(
        &self,
        block_hash: &BlockHash,
        node: &BlockNode,
        is_active: bool,
    ) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let node_bytes = node.serialize();
        {
            let mut table = write.open_table(BLOCK_NODE_TABLE)?;
            table.insert(block_hash.as_bytes(), node_bytes.as_slice())?;

            if is_active {
                let mut height_index_table = write.open_table(HEIGHT_INDEX_TABLE)?;
                height_index_table.insert(node.height, block_hash.as_bytes())?;
            }
        }
        write.commit()?;
        Ok(())
    }

    pub fn get(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, Error> {
        let read = self.db.begin_read()?;

        let table = read.open_table(BLOCK_NODE_TABLE)?;

        let value = table.get(block_hash.as_bytes())?;

        match value {
            Some(value) => {
                let (data, _) = BlockNode::deserialize(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    pub fn remove(&self, block_hash: &BlockHash) -> Result<Option<BlockNode>, Error> {
        let write = self.db.begin_write()?;

        let result = {
            let mut table = write.open_table(BLOCK_NODE_TABLE)?;

            match table.remove(block_hash.as_bytes())? {
                Some(value) => {
                    let (entry, _) = BlockNode::deserialize(value.value())
                        .map_err(|_| StorageError::Corrupted("deserialization failed".into()))?;

                    let _ = self.remove_height_index(entry.height)?;
                    Some(entry)
                }
                None => None,
            }
        };

        write.commit()?;
        Ok(result)
    }

    pub fn remove_height_index(&self, height: u32) -> Result<Option<BlockHash>, Error> {
        let write = self.db.begin_write()?;

        let result = {
            let mut table = write.open_table(HEIGHT_INDEX_TABLE)?;

            match table.remove(height)? {
                Some(value) => {
                    Some(BlockHash(*value.value()))
                }
                None => None,
            }
        };

        write.commit()?;
        Ok(result)
    }

    pub fn insert_node_height_index(
        &self,
        height: u32,
        block_hash: &BlockHash,
    ) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        {
            let mut table = write.open_table(HEIGHT_INDEX_TABLE)?;
            table.insert(height, block_hash.as_bytes())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_node_by_height(&self, height: u32) -> Result<Option<BlockNode>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(HEIGHT_INDEX_TABLE)?;
        let value = table.get(height)?;

        match value {
            Some(value) => {
                let block_hash = BlockHash(*value.value());
                self.get(&block_hash)
            }
            None => Ok(None),
        }
    }
}
