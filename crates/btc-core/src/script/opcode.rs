use crate::serialization::BitcoinSerialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    Dup,
    Hash160,
    EqualVerify,
    CheckSig
}

impl BitcoinSerialize for OpCode {
    fn serialize(&self) -> Vec<u8> {
        match self {
            OpCode::Dup => vec![0x76],
            OpCode::Hash160 => vec![0xa9],
            OpCode::EqualVerify => vec![0x88],
            OpCode::CheckSig => vec![0xac],
        }
    }
}
