#[derive(PartialEq, Eq, Debug)]
pub enum MerkleError {
    EmptyTransactionList,
    TransactionNotFound,
    InvalidProof,
}