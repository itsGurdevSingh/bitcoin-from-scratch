use btc_core::{
    blockchain::block_node::BlockNodeMetadata,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::BlockHash,
};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::BLOCK_NODE_METADATA_TABLE;
pub struct BlockMetadataStore<'a> {
    pub db: &'a Database,
}

impl<'a> BlockMetadataStore<'a> {
    pub fn insert_block_node_metadata(
        &self,
        block_hash: &BlockHash,
        metadata: &BlockNodeMetadata,
    ) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let metadata_bytes = metadata.serialize();
        {
            let mut table = write.open_table(BLOCK_NODE_METADATA_TABLE)?;
            table.insert(block_hash.as_bytes(), metadata_bytes.as_slice())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_block_node_metadata(
        &self,
        block_hash: &BlockHash,
    ) -> Result<Option<BlockNodeMetadata>, Error> {
        let read = self.db.begin_read()?;

        let table = read.open_table(BLOCK_NODE_METADATA_TABLE)?;

        let value = table.get(block_hash.as_bytes())?;

        match value {
            Some(value) => {
                let (data, _) = BlockNodeMetadata::deserialize(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }
}
