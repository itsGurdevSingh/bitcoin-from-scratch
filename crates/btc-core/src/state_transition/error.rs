use crate::validator::ValidationError;

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessorError {
    Validation(ValidationError),
    MissingUtxo,
}