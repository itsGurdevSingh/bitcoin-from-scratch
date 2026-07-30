use crate::{taproot::TaprootError, transaction::SigHashError};

#[derive(Debug, PartialEq, Eq)]
pub enum VmError {
    EmptyStack,
    InvalidOpcode,
    VerifyFailed,
    InvalidData,
    EmptyScript,
    InvalidScriptFormat,
    NonPushOnlyScriptSig,
    ReturnOp,

    //configration limits
    StackOverflow,
    StackUnderflow,
    ScriptTooLarge,
    ScriptElementTooLarge,
    SigHash(SigHashError),
    Taproot(TaprootError)
}