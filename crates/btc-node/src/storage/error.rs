use redb::Error;

#[derive(Debug)]
pub enum StorageError {
    Database(Error),
    InvalidBlock,
    InvalidHeader,
    InvalidTransaction,
}