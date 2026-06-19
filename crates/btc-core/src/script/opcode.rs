#[derive(Debug, Clone)]
pub enum OpCode {
    Dup,
    Hash160,
    EqualVerify,
    CheckSig
}