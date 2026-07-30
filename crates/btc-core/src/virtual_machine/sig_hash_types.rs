#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SigHashType {
    Default = 0x00,
    All = 0x01,
    None = 0x02,
    Single = 0x03,
    AllAnyoneCanPay = 0x81,
    NoneAnyoneCanPay = 0x82,
    SingleAnyoneCanPay = 0x83,
}

impl SigHashType {
    pub fn is_anyone_can_pay(&self) -> bool {
        match self {
            Self::AllAnyoneCanPay | Self::SingleAnyoneCanPay | Self::NoneAnyoneCanPay => true,
            _ => false
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None | Self::NoneAnyoneCanPay => true,
            _ => false
        }
    }

    pub fn is_single(&self) -> bool {
        match self {
            Self::Single | Self::SingleAnyoneCanPay => true,
            _ => false
        }
    }

    pub fn is_all(&self) -> bool {
        match self {
            Self::Default | Self::All | Self::AllAnyoneCanPay => true,
            _ => false
        }
    }
}

impl TryFrom<u32> for SigHashType {
    type Error = String; // or a custom error type

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(SigHashType::Default),
            0x01 => Ok(SigHashType::All),
            0x02 => Ok(SigHashType::None),
            0x03 => Ok(SigHashType::Single),
            0x81 => Ok(SigHashType::AllAnyoneCanPay),
            0x82 => Ok(SigHashType::NoneAnyoneCanPay),
            0x83 => Ok(SigHashType::SingleAnyoneCanPay),
            _ => Err(format!("Invalid SigHashType value: {:#X}", value)),
        }
    }
}
