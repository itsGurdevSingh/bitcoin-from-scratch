#[derive(Debug, PartialEq, Eq)]
pub enum VmError {
    EmptyStack,
    InvalidOpcode,
    VerifyFailed,
    InvalidData,
    EmptyScript,
    InvalidSriptFormat,
    ReturnOp,

    //configration limits
    StackOverflow,
    StackUnderflow,
    ScriptTooLarge,
    ScriptElementTooLarge,
}