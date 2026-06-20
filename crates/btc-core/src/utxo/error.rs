#[derive(Debug, PartialEq, Eq)]
pub enum UtxoError {
    AlreadyExists,
    NotFound,
}