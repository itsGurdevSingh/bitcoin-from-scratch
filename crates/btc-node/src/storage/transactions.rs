use btc_core::{transaction::Transaction, types::TxId};
use redb::{Database, Error, ReadableDatabase, StorageError};

use crate::storage::tables::TRANSACTIONS_TABLE;
pub struct TransactionStore<'a> {
    pub db: &'a Database,
}

impl<'a> TransactionStore<'a> {
    pub fn insert_transaction(&self, txid: &TxId, transaction: &Transaction) -> Result<(), Error> {
        let write = self.db.begin_write()?;
        let tx_bytes = transaction.serialize_witness();
        {
            let mut table = write.open_table(TRANSACTIONS_TABLE)?;
            table.insert(txid.as_bytes(), tx_bytes.as_slice())?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn get_transaction(&self, txid: &TxId) -> Result<Option<Transaction>, Error> {
        let read = self.db.begin_read()?;

        let table = read.open_table(TRANSACTIONS_TABLE)?;

        let value = table.get(txid.as_bytes())?;

        match value {
            Some(value) => {
                let (tx, _) = Transaction::deserialize_witness(value.value())
                    .map_err(|_| StorageError::Corrupted(String::from("deserialization failed")))?;
                Ok(Some(tx))
            }
            None => Ok(None),
        }
    }
}
