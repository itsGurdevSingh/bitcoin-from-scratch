#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PersistenceError {
    OprationFaild,
    StoragePoisoned
}