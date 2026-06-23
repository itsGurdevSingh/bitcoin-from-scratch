#[derive(Debug, PartialEq, Eq)]
pub enum VmError {
    EmptyStack,
    InvalidOpcode,
    VerifyFailed,
    InvalidData,
    EmptyScript,

    //configration limits
    StackOverflow,
    ScriptTooLarge,
    ScriptElementTooLarge,
}