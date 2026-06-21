pub enum ValidationError {
    NoInputs,
    NoOutputs,

    MissingUtxo,

    DuplicateInput,

    InsufficientInputValue,

    InvalidOutputValue,
    
    InvalidOutputScript,
}