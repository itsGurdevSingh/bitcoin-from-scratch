#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaprootError {
    InvalidTweak,
    InvalidXonlyBytes,
    SigningOutputNotExist, 

    TargetScriptNotExist
}