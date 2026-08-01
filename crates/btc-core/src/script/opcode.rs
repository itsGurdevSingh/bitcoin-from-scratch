use crate::{
    serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
    virtual_machine::VmError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Op0 = 0x00,

    Op1Negate = 0x4f,
    Op1 = 0x51,
    Op2 = 0x52,
    Op3 = 0x53,
    Op4 = 0x54,
    Op5 = 0x55,
    Op6 = 0x56,
    Op7 = 0x57,
    Op8 = 0x58,
    Op9 = 0x59,
    Op10 = 0x5a,
    Op11 = 0x5b,
    Op12 = 0x5c,
    Op13 = 0x5d,
    Op14 = 0x5e,
    Op15 = 0x5f,
    Op16 = 0x60,

    If = 0x63,
    NotIf = 0x64,
    Else = 0x67,
    EndIf = 0x68,
    Verify = 0x69,
    Return = 0x6a,

    Drop2 = 0x6d,
    Dup2 = 0x6e,
    IfDup = 0x73,
    Depth = 0x74,
    Drop = 0x75,
    Dup = 0x76,
    Nip = 0x77,
    Over = 0x78,
    Swap = 0x7c,
    Tuck = 0x7d,

    Equal = 0x87,
    EqualVerify = 0x88,

    Abs = 0x90,
    Not = 0x91,
    NotEqual0 = 0x92,
    Add = 0x93,
    Sub = 0x94,
    Negate = 0x7f,
    BoolAnd = 0x9a,
    BoolOr = 0x9b,
    NumEqual = 0x9c,
    LessThan = 0x9d,
    GreaterThan = 0x9e,

    Min = 0xa3,
    Max = 0xa4,
    WithIn = 0xa5,

    Sha1 = 0xa7,
    Sha256 = 0xa8,
    Hash160 = 0xa9,
    Hash256 = 0xaa,

    CheckSig = 0xac,
    CheckSigVerify = 0xad,
    CheckSigAdd = 0xae
}
impl OpCode{
    pub fn is_push_only(&self) -> bool {
        (*self as u8) <= (OpCode::Op16 as u8)
    }
}


// Fixed the trait name capitalization to match your code block definition
impl BitcoinSerialize for OpCode {
    fn serialize(&self) -> Vec<u8> {
        return vec![*self as u8];
    }
}

impl BitcoinDeserialize for OpCode {
    type Error = DeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.is_empty() {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let opcode = match bytes[0] {
            // Push values
            0x00 => OpCode::Op0,
            0x4f => OpCode::Op1Negate,
            0x51 => OpCode::Op1,
            0x52 => OpCode::Op2,
            0x53 => OpCode::Op3,
            0x54 => OpCode::Op4,
            0x55 => OpCode::Op5,
            0x56 => OpCode::Op6,
            0x57 => OpCode::Op7,
            0x58 => OpCode::Op8,
            0x59 => OpCode::Op9,
            0x5a => OpCode::Op10,
            0x5b => OpCode::Op11,
            0x5c => OpCode::Op12,
            0x5d => OpCode::Op13,
            0x5e => OpCode::Op14,
            0x5f => OpCode::Op15,
            0x60 => OpCode::Op16,

            // Flow control
            0x63 => OpCode::If,
            0x64 => OpCode::NotIf,
            0x67 => OpCode::Else,
            0x68 => OpCode::EndIf,
            0x69 => OpCode::Verify,
            0x6a => OpCode::Return,

            // Stack
            0x6d => OpCode::Drop2,
            0x6e => OpCode::Dup2,
            0x73 => OpCode::IfDup,
            0x74 => OpCode::Depth,
            0x75 => OpCode::Drop,
            0x76 => OpCode::Dup,
            0x77 => OpCode::Nip,
            0x78 => OpCode::Over,
            0x7c => OpCode::Swap,
            0x7d => OpCode::Tuck,

            // Comparison
            0x87 => OpCode::Equal,
            0x88 => OpCode::EqualVerify,
            0x9c => OpCode::NumEqual,

            // Arithmetic
            0x7f => OpCode::Negate,
            0x90 => OpCode::Abs,
            0x91 => OpCode::Not,
            0x92 => OpCode::NotEqual0,
            0x93 => OpCode::Add,
            0x94 => OpCode::Sub,
            0x9a => OpCode::BoolAnd,
            0x9b => OpCode::BoolOr,
            0x9d => OpCode::LessThan,
            0x9e => OpCode::GreaterThan,
            0xa3 => OpCode::Min,
            0xa4 => OpCode::Max,
            0xa5 => OpCode::WithIn,

            // Crypto
            0xa7 => OpCode::Sha1,
            0xa8 => OpCode::Sha256,
            0xa9 => OpCode::Hash160,
            0xaa => OpCode::Hash256,
            0xac => OpCode::CheckSig,
            0xad => OpCode::CheckSigVerify,
            0xae => OpCode::CheckSigAdd,

            value=> {
                return Err(DeserializeError::UnknownOpcode(value));
            },
        };

        Ok((opcode, 1))
    }
}

pub trait OpCodeTrait {
    fn dup(&mut self) -> Result<(), VmError>;
    fn hash160(&mut self) -> Result<(), VmError>;
    fn check_sig(&mut self) -> Result<(), VmError>;
    fn check_sig_verify(&mut self) -> Result<(), VmError>;
    fn check_sig_add(&mut self) -> Result<(), VmError>;
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
