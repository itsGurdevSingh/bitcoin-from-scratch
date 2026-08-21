use btc_core::{
    crypto::sha256d,
    serialization::{BitcoinDeserialize, BitcoinSerialize},
};

use crate::network::{Command, NetworkDeserializeError, config::MAGIC};

const MESSAGE_HEADER_LEN: usize = 24;
const NETWORK_MAGIC_LEN: usize = 4;
const COMMAND_FIELD_LEN: usize = 12;
const PAYLOAD_LENGTH_LEN: usize = 4;
const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024; //4mb
const CHECKSUM_LEN: usize = 4;

// ┌────────────────────────────┐
// │ magic        4 bytes       │
// │ command     12 bytes       │
// │ length       4 bytes       │
// │ checksum     4 bytes       │
// ├────────────────────────────┤
// │ payload      N bytes       │
// └────────────────────────────┘

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkMessageHeader {
    pub command: Command,
    pub payload_len: usize,
    pub checksum: [u8; 4]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkMessage {
    pub command: Command,
    pub payload: Vec<u8>,
}

impl BitcoinSerialize for NetworkMessage {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(MAGIC); //[1,2,3,4]

        let mut command = [0u8; COMMAND_FIELD_LEN];
        let name = self.command.as_wire_name();
        command[..name.len()].copy_from_slice(name);
        bytes.extend(command);

        bytes.extend((self.payload.len() as u32).to_le_bytes());

        let checksum = Self::checksum(&self.payload);
        bytes.extend(checksum);

        bytes.extend(&self.payload);

        bytes
    }
}

impl BitcoinDeserialize for NetworkMessageHeader {
    type Error = NetworkDeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.len() < MESSAGE_HEADER_LEN {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: MESSAGE_HEADER_LEN,
                actual: bytes.len(),
            });
        }

        let mut offset = 0;

        let magic: [u8; NETWORK_MAGIC_LEN] =
            bytes[offset..offset + NETWORK_MAGIC_LEN]
                .try_into()
                .map_err(|_| NetworkDeserializeError::UnexpectedEndOfBytes {
                    expected: MESSAGE_HEADER_LEN,
                    actual: bytes.len(),
                })?;
        if magic != MAGIC {
            return Err(NetworkDeserializeError::InvalidMagic);
        }
        offset += NETWORK_MAGIC_LEN;

        let command_bytes = &bytes[offset..offset + COMMAND_FIELD_LEN];
        let command = Command::from_wire_name(command_bytes)
            .ok_or(NetworkDeserializeError::InvalidCommand)?;
        offset += COMMAND_FIELD_LEN;

        let payload_len_bytes: [u8; PAYLOAD_LENGTH_LEN] = bytes
            [offset..offset + PAYLOAD_LENGTH_LEN]
            .try_into()
            .map_err(|_| NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + PAYLOAD_LENGTH_LEN,
                actual: bytes.len(),
            })?;
        let payload_len = u32::from_le_bytes(payload_len_bytes) as usize;
        offset += PAYLOAD_LENGTH_LEN;

        if payload_len as usize > MAX_PAYLOAD_SIZE  {
            Err(NetworkDeserializeError::PayloadTooLarge)?
        }

        let checksum_bytes: [u8; CHECKSUM_LEN] = bytes[offset..offset + CHECKSUM_LEN]
            .try_into()
            .map_err(|_| NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + CHECKSUM_LEN,
                actual: bytes.len(),
            })?;
        offset += CHECKSUM_LEN;

        Ok((Self { command, payload_len, checksum: checksum_bytes }, offset))
    }
}

impl BitcoinDeserialize for NetworkMessage {
    type Error = NetworkDeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let (header, consumed) = NetworkMessageHeader::deserialize(bytes)?;
        offset += consumed;

        let payload = bytes[offset..].to_vec();

        if header.payload_len != payload.len() {
            return Err(NetworkDeserializeError::InvalidPayloadLength {
                expected: header.payload_len,
                actual: payload.len(),
            });
        }

        if header.checksum != Self::checksum(&payload) {
            return Err(NetworkDeserializeError::InvalidChecksum);
        }

        Ok((Self { command:header.command ,payload }, bytes.len()))
    }
}

impl NetworkMessage {
    fn checksum(payload: &[u8]) -> [u8; CHECKSUM_LEN] {
        let hash = sha256d(payload);
        let mut checksum = [0u8; CHECKSUM_LEN];
        checksum.copy_from_slice(&hash[..CHECKSUM_LEN]);
        checksum
    }
}
