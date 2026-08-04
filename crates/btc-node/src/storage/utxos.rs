use btc_core::{
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    transaction::OutPoint,
    utxo::Utxo,
};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::UTXOS_TABLE;

pub struct UtxoStore<'a> {
    pub db: &'a Database,
}

impl<'a> UtxoStore<'a> {
    pub fn insert_utxo(&self, outpoint: &OutPoint, utxo: &Utxo) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let utxo_bytes = utxo.serialize();
        let key = Self::key_from_outpoint(outpoint);

        {
            let mut table = write.open_table(UTXOS_TABLE)?;
            table.insert(key.as_slice(), utxo_bytes.as_slice())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<Utxo>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(UTXOS_TABLE)?;
        let key = Self::key_from_outpoint(outpoint);

        let value = table.get(key.as_slice())?;
        match value {
            Some(value) => {
                let (utxo, _) = Utxo::deserialize(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(utxo))
            }
            None => Ok(None),
        }
    }

    fn key_from_outpoint(outpoint: &OutPoint) -> Vec<u8> {
        let mut key: Vec<u8> = Vec::new();
        key.extend_from_slice(outpoint.txid.as_bytes());
        key.extend(outpoint.vout.to_le_bytes());
        key
    }
}
