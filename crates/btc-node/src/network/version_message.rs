use btc_core::serialization::{
    BitcoinDeserialize, BitcoinSerialize,
    compact_size::{get_compact_size, read_compact_size},
};

use crate::network::NetworkDeserializeError;

pub struct NetworkAddress {
    pub services: u64,
    pub ip: [u8; 16],
    pub port: u16,
}

pub struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: u64,
    pub receiver: NetworkAddress,
    pub sender: NetworkAddress,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: u32,
    pub relay: bool,
}

impl BitcoinSerialize for NetworkAddress {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.services.to_le_bytes());
        bytes.extend(self.ip);
        bytes.extend(self.port.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for NetworkAddress {
    type Error = NetworkDeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        const ADDRESS_SIZE: usize = 8 + 16 + 2; // services + ip + port

        if bytes.len() < ADDRESS_SIZE {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: ADDRESS_SIZE,
                actual: bytes.len(),
            });
        }

        let mut offset = 0;

        let services = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let mut ip = [0u8; 16];
        ip.copy_from_slice(&bytes[offset..offset + 16]);
        offset += 16;

        let port = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        Ok((Self { services, ip, port }, offset))
    }
}

impl BitcoinSerialize for VersionMessage {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.version.to_le_bytes());
        bytes.extend(self.services.to_le_bytes());
        bytes.extend(self.timestamp.to_le_bytes());

        bytes.extend(self.receiver.serialize());
        bytes.extend(self.sender.serialize());

        bytes.extend(self.nonce.to_le_bytes());

        let user_agent_bytes = self.user_agent.as_bytes();
        bytes.extend(get_compact_size(user_agent_bytes.len()));
        bytes.extend(user_agent_bytes);

        bytes.extend(self.start_height.to_le_bytes());
        bytes.push(if self.relay { 1 } else { 0 });

        bytes
    }
}

impl BitcoinDeserialize for VersionMessage {
    type Error = NetworkDeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        // version (4 bytes)
        if bytes.len() < offset + 4 {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 4,
                actual: bytes.len(),
            });
        }
        let version = i32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // services (8 bytes)
        if bytes.len() < offset + 8 {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 8,
                actual: bytes.len(),
            });
        }
        let services = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // timestamp (8 bytes)
        if bytes.len() < offset + 8 {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 8,
                actual: bytes.len(),
            });
        }
        let timestamp = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // receiver (26 bytes)
        let (receiver, consumed) = NetworkAddress::deserialize(&bytes[offset..]).map_err(|_| {
            NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 26,
                actual: bytes.len(),
            }
        })?;
        offset += consumed;

        // sender (26 bytes)
        let (sender, consumed) = NetworkAddress::deserialize(&bytes[offset..]).map_err(|_| {
            NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 26,
                actual: bytes.len(),
            }
        })?;
        offset += consumed;

        // nonce (8 bytes)
        if bytes.len() < offset + 8 {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 8,
                actual: bytes.len(),
            });
        }
        let nonce = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // user_agent (variable length)
        let (user_agent_len, len_consumed) = read_compact_size(&bytes[offset..]).map_err(|_| {
            NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 1,
                actual: bytes.len(),
            }
        })?;
        offset += len_consumed;

        if bytes.len() < offset + user_agent_len {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + user_agent_len,
                actual: bytes.len(),
            });
        }

        let user_agent = String::from_utf8(bytes[offset..offset + user_agent_len].to_vec())
            .map_err(|_| NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + user_agent_len,
                actual: bytes.len(),
            })?;
        offset += user_agent_len;

        // start_height (4 bytes)
        if bytes.len() < offset + 4 {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 4,
                actual: bytes.len(),
            });
        }
        let start_height = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // relay (1 byte)
        if bytes.len() < offset + 1 {
            return Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: offset + 1,
                actual: bytes.len(),
            });
        }
        let relay = bytes[offset] != 0;
        offset += 1;

        Ok((
            Self {
                version,
                services,
                timestamp,
                receiver,
                sender,
                nonce,
                user_agent,
                start_height,
                relay,
            },
            offset,
        ))
    }
}
