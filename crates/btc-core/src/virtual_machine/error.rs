pub enum VmError {
    EmptyStack,
    InvalidOpcode,
    VerifyFailed,
    InvalidData,
}