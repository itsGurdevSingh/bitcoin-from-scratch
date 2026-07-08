use crate::{serialization::BitcoinSerialize, virtual_machine::VmError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    Dup,
    Hash160,
    Equal,
    Verify,
    EqualVerify,
    CheckSig,

    Op0,
    Op1Negate,
    Op1,
    Op2,
    Op3,
    Op4,
    Op5,
    Op6,
    Op7,
    Op8,
    Op9,
    Op10,
    Op11,
    Op12,
    Op13,
    Op14,
    Op15,
    Op16,

    // PushData1,
    // PushData2,
    // PushData4,
    Drop,
    Swap,
    Over,
    Depth,
    Nip,
    Tuck,
    Drop2,
    Dup2,
    IfDup,
    NumEqual,

    Sha1,
    Sha256,
    Hash256,

    CheckSigVerify,

    Add,
    Sub,
    Negate,
    Abs,
    Not,
    NotEqual0,
    BoolAnd,
    BoolOr,

    LessThan,
    GreaterThan,
    Min,
    Max,
    WithIn,

    If,
    NotIf,
    Else,
    EndIf,
    Return,
}

// Fixed the trait name capitalization to match your code block definition
impl BitcoinSerialize for OpCode {
    fn serialize(&self) -> Vec<u8> {
        match self {
            // Push values & Data
            OpCode::Op0 => vec![0x00],
            // OpCode::PushData1 => vec![0x4c],
            // OpCode::PushData2 => vec![0x4d],
            // OpCode::PushData4 => vec![0x4e],
            OpCode::Op1Negate => vec![0x4f],
            OpCode::Op1 => vec![0x51],
            OpCode::Op2 => vec![0x52],
            OpCode::Op3 => vec![0x53],
            OpCode::Op4 => vec![0x54],
            OpCode::Op5 => vec![0x55],
            OpCode::Op6 => vec![0x56],
            OpCode::Op7 => vec![0x57],
            OpCode::Op8 => vec![0x58],
            OpCode::Op9 => vec![0x59],
            OpCode::Op10 => vec![0x5a],
            OpCode::Op11 => vec![0x5b],
            OpCode::Op12 => vec![0x5c],
            OpCode::Op13 => vec![0x5d],
            OpCode::Op14 => vec![0x5e],
            OpCode::Op15 => vec![0x5f],
            OpCode::Op16 => vec![0x60],

            // Flow control
            OpCode::If => vec![0x63],
            OpCode::NotIf => vec![0x64],
            OpCode::Else => vec![0x67],
            OpCode::EndIf => vec![0x68],
            OpCode::Verify => vec![0x69],
            OpCode::Return => vec![0x6a],

            // Stack operations
            OpCode::Drop => vec![0x75],
            OpCode::Dup => vec![0x76],
            OpCode::Nip => vec![0x77],
            OpCode::Over => vec![0x78],
            OpCode::Swap => vec![0x7c],
            OpCode::Tuck => vec![0x7d],
            OpCode::Drop2 => vec![0x6d],
            OpCode::Dup2 => vec![0x6e],
            OpCode::IfDup => vec![0x73],
            OpCode::Depth => vec![0x74],

            // Splice / String / Logic / Comparison
            OpCode::Equal => vec![0x87],
            OpCode::EqualVerify => vec![0x88],
            OpCode::NumEqual => vec![0x9c],

            // Arithmetic
            OpCode::Add => vec![0x93],
            OpCode::Sub => vec![0x94],
            OpCode::Negate => vec![0x7f],
            OpCode::Abs => vec![0x90],
            OpCode::Not => vec![0x91],
            OpCode::NotEqual0 => vec![0x92],
            OpCode::BoolAnd => vec![0x9a],
            OpCode::BoolOr => vec![0x9b],
            OpCode::LessThan => vec![0x9d],
            OpCode::GreaterThan => vec![0x9e],
            OpCode::Min => vec![0xa3],
            OpCode::Max => vec![0xa4],
            OpCode::WithIn => vec![0xa5],

            // Cryptography
            OpCode::Sha1 => vec![0xa7],
            OpCode::Sha256 => vec![0xa8],
            OpCode::Hash160 => vec![0xa9],
            OpCode::Hash256 => vec![0xaa],
            OpCode::CheckSig => vec![0xac],
            OpCode::CheckSigVerify => vec![0xad],
        }
    }
}

pub trait OpCodeTrait {
    fn dup(&mut self) -> Result<(), VmError>;
    fn hash160(&mut self) -> Result<(), VmError>;
    fn check_sig(&mut self) -> Result<(), VmError>;
    fn check_sig_verify(&mut self) -> Result<(), VmError>;
    fn equal(&mut self) -> Result<(), VmError>;
    fn verify(&mut self) -> Result<(), VmError>;
    fn equal_verify(&mut self) -> Result<(), VmError>;

    fn op_1negate(&mut self) -> Result<(), VmError>;
    fn op_0(&mut self) -> Result<(), VmError>;
    fn op_1(&mut self) -> Result<(), VmError>;
    fn op_2(&mut self) -> Result<(), VmError>;
    fn op_3(&mut self) -> Result<(), VmError>;
    fn op_4(&mut self) -> Result<(), VmError>;
    fn op_5(&mut self) -> Result<(), VmError>;
    fn op_6(&mut self) -> Result<(), VmError>;
    fn op_7(&mut self) -> Result<(), VmError>;
    fn op_8(&mut self) -> Result<(), VmError>;
    fn op_9(&mut self) -> Result<(), VmError>;
    fn op_10(&mut self) -> Result<(), VmError>;
    fn op_11(&mut self) -> Result<(), VmError>;
    fn op_12(&mut self) -> Result<(), VmError>;
    fn op_13(&mut self) -> Result<(), VmError>;
    fn op_14(&mut self) -> Result<(), VmError>;
    fn op_15(&mut self) -> Result<(), VmError>;
    fn op_16(&mut self) -> Result<(), VmError>;

    fn drop(&mut self) -> Result<(), VmError>;
    fn swap(&mut self) -> Result<(), VmError>;
    fn over(&mut self) -> Result<(), VmError>;
    fn depth(&mut self) -> Result<(), VmError>;
    fn nip(&mut self) -> Result<(), VmError>;
    fn tuck(&mut self) -> Result<(), VmError>;
    fn dup_2(&mut self) -> Result<(), VmError>;
    fn drop_2(&mut self) -> Result<(), VmError>;
    fn if_dup(&mut self) -> Result<(), VmError>;


    fn num_equal(&mut self) -> Result<(), VmError>;


    fn sha1(&mut self) -> Result<(), VmError>;
    fn sha256(&mut self) -> Result<(), VmError>;
    fn hash256(&mut self) -> Result<(), VmError>;


    fn add(&mut self) -> Result<(), VmError>;
    fn sub(&mut self) -> Result<(), VmError>;
    fn negate(&mut self) -> Result<(), VmError>;
    fn abs(&mut self) -> Result<(), VmError>;
    fn not(&mut self) -> Result<(), VmError>;
    fn not_equal_0(&mut self) -> Result<(), VmError>;
    fn bool_and(&mut self) -> Result<(), VmError>;
    fn bool_or(&mut self) -> Result<(), VmError>;


    fn less_than(&mut self) -> Result<(), VmError>;
    fn grater_than(&mut self) -> Result<(), VmError>;
    fn max(&mut self) -> Result<(), VmError>;
    fn min(&mut self) -> Result<(), VmError>;
    fn within(&mut self) -> Result<(), VmError>;
}

pub trait FlowControl {
    fn op_if(&mut self) -> Result<(), VmError>;
    fn op_not_if(&mut self) -> Result<(), VmError>;
    fn op_else(&mut self) -> Result<(), VmError>;
    fn op_endif(&mut self) -> Result<(), VmError>;
    fn op_return(&self) -> Result<(), VmError>;
}