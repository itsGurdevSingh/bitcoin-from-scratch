use crate::serialization::DeserializeError;

pub fn get_compact_size(value: usize) -> Vec<u8>{
    match value {
            0..=252 => vec![value as u8],

            253..=0xFFFF => {
                let mut bytes = vec![0xFD];
                bytes.extend_from_slice(&(value as u16).to_le_bytes());
                bytes
            }

            0x10000..=0xFFFF_FFFF => {
                let mut bytes = vec![0xFE];
                bytes.extend_from_slice(&(value as u32).to_le_bytes());
                bytes
            }

            _ => {
                let mut bytes = vec![0xFF];
                bytes.extend_from_slice(&value.to_le_bytes());
                bytes
            }
        }
}

type Value = usize;
type BytesConsumed = usize;

pub fn read_compact_size(bytes: &[u8]) -> Result<(Value, BytesConsumed), DeserializeError> {
    if bytes.is_empty() {
        return Err(DeserializeError::UnexpectedEndOfBytes);
    }

    match bytes[0] {
        0x00..=0xFC => Ok((bytes[0] as usize, 1)),

        0xFD => {
            if bytes.len() < 3 {
                return Err(DeserializeError::UnexpectedEndOfBytes);
            }
            let value = u16::from_le_bytes([bytes[1], bytes[2]]);
            Ok((value as usize, 3))
        }

        0xFE => {
            if bytes.len() < 5 {
                return Err(DeserializeError::UnexpectedEndOfBytes);
            }
            let value = u32::from_le_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
            ]);
            Ok((value as usize, 5))
        }

        0xFF => {
            if bytes.len() < 9 {
                return Err(DeserializeError::UnexpectedEndOfBytes);
            }
            let value = u64::from_le_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8],
            ]);
            let value = usize::try_from(value).map_err(|_| DeserializeError::CountOverflow)?;
            Ok((value, 9))
        }
    }
}