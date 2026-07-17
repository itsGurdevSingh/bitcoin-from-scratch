#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeError {
    UnexpectedEndOfBytes,
    InvalidCompactSize,
    CountOverflow,
    InvalidSegWitFlag(u8),
    UnknownOpcode(u8),
}

impl core::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeserializeError::UnexpectedEndOfBytes => write!(f, "unexpected end of bytes"),
            DeserializeError::InvalidCompactSize => write!(f, "invalid compact size encoding"),
            DeserializeError::CountOverflow => write!(f, "decoded count does not fit in usize"),
            DeserializeError::InvalidSegWitFlag(flag) => {
                write!(f, "invalid segwit flag: {flag:#04x}")
            }
            DeserializeError::UnknownOpcode(opcode) => write!(f, "unknown opcode: {opcode:#04x}"),
        }
    }
}

impl std::error::Error for DeserializeError {}