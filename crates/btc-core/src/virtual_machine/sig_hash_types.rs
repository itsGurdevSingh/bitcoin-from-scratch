#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum SigHashType {
    All = 0x01,
    None = 0x02,
    Single = 0x03,
    AllAnyoneCanPay = 0x81,
    NoneAnyoneCanPay = 0x82,
    SingleAnyoneCanPay = 0x83,
}

impl TryFrom<u32> for SigHashType {
    type Error = String; // or a custom error type

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
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