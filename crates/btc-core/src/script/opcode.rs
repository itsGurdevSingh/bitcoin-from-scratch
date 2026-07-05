use crate::{serialization::BitcoinSerialize, virtual_machine::VmError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    Dup,
    Hash160,

    Equal,
    EqualVerify,

    Verify,

    CheckSig,
}

impl BitcoinSerialize for OpCode {
    fn serialize(&self) -> Vec<u8> {
        match self {
            OpCode::Dup => vec![0x76],
            OpCode::Hash160 => vec![0xa9],
            OpCode::CheckSig => vec![0xac],
            OpCode::Equal => vec![0x86],
            OpCode::Verify => vec![0x87],
            OpCode::EqualVerify => vec![0x88],
        }
    }
}

pub trait OpCodeTrait {
    fn dup(&mut self) -> Result<(), VmError>;
    fn hash160(&mut self) -> Result<(), VmError>;
    fn check_sig(&mut self) -> Result<(), VmError>;
    fn equal(&mut self) -> Result<(),VmError>;
    fn verify(&mut self) -> Result<(), VmError>;
    fn equal_verify(&mut self) -> Result<(), VmError>;

}
