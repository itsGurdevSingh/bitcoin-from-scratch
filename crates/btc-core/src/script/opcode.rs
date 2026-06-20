#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    Dup,
    Hash160,
    EqualVerify,
    CheckSig
}