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

pub fn read_compact_size(bytes: &[u8]) -> (Value, BytesConsumed) {
    match bytes[0] {
        0x00..=0xFC => (bytes[0] as usize, 1 as usize),

        0xFD => {
            let value = u16::from_le_bytes([bytes[1], bytes[2]]);
            (value as usize, 3 as usize)
        }

        0xFE => {
            let value = u32::from_le_bytes([
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
            ]);
            (value as usize, 5 as usize)
        }

        0xFF => {
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
            (value as usize, 9 as usize)
        }
    }
}