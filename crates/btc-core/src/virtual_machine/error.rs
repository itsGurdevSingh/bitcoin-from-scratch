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
    CleanStack,

    // Script / witness / taproot specific errors
    MissingUtxo,
    MissingRedeemScript,
    InvalidRedeemScript,
    InvalidRedeemScriptLength,
    RedeemScriptHashMismatch,
    P2wshScriptSigNotAllowed,
    MissingWitnessScript,
    InvalidWitnessStackSize,
    WitnessScriptHashMismatch,
    P2wpkhScriptSigNotAllowed,
    InvalidTaprootSpendType,
    MissingTaprootSignature,
    MissingTaprootControlBlock,
    MissingTaprootScript,
    TaprootCommitmentMismatch,
    TaprootSignatureVerificationFailed,

    //configration limits
    StackOverflow,
    StackUnderflow,
    ScriptTooLarge,
    ScriptElementTooLarge,
    SigHash(SigHashError),
    Taproot(TaprootError),
}
