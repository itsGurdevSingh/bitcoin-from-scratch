#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaprootError {
    InvalidTweak,
    SigningOutputNotExist
}