#[derive(Debug, PartialEq, Eq)]
pub enum VmError {
    EmptyStack,
    InvalidOpcode,
    VerifyFailed,
    InvalidData,
}