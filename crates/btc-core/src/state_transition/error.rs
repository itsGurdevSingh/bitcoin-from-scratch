use crate::validator::ValidationError;

pub enum ProcessorError {
    Validation(ValidationError),
    MissingUtxo,
}