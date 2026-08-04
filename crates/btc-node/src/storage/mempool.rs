use btc_core::types::TxId;
use redb::{Database, Error, ReadableDatabase};

use crate::storage::tables::TRANSACTIONS_FEES_TABLE;
pub struct MempoolStore<'a> {
    pub db: &'a Database,
}

impl<'a> MempoolStore<'a> {
    pub fn insert_tx_fees(&self, txid: &TxId, fees: u64) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        {
            let mut table = write.open_table(TRANSACTIONS_FEES_TABLE)?;
            table.insert(txid.as_bytes(), fees)?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_tx_fees(&self, txid: &TxId) -> Result<Option<u64>, Error> {
        let read = self.db.begin_read()?;
        let table = read.open_table(TRANSACTIONS_FEES_TABLE)?;
        let value = table.get(txid.as_bytes())?;

        match value {
            Some(value) => Ok(Some(value.value())),
            None => Ok(None),
        }
    }
}
