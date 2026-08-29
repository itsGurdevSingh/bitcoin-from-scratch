use btc_core::{
    block::BlockHeader,
    serialization::{
        BitcoinDeserialize, BitcoinSerialize,
        compact_size::{get_compact_size, read_compact_size},
    },
    types::BlockHash,
};

use crate::network::{NetworkDeserializeError, config::PROTOCOL_VERSION};

pub struct GetHeadersMessage {
    pub version: u32,
    pub locator_hashes: Vec<BlockHash>,
    pub stop_hash: BlockHash,
}

pub struct HeadersMessage {
    pub headers: Vec<BlockHeader>,
}

impl GetHeadersMessage {
    pub fn new(locator_hashes: Vec<BlockHash>, stop_hash: BlockHash) -> Self {
        Self {
            version: PROTOCOL_VERSION as u32,
            locator_hashes,
            stop_hash,
        }
    }
}

impl HeadersMessage {
    pub fn new(headers: Vec<BlockHeader>) -> Self {
        Self { headers }
    }
}

impl BitcoinSerialize for HeadersMessage {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(get_compact_size(self.headers.len()));

        for header in self.headers.iter() {
            bytes.extend(header.serialize());
        }
        bytes
    }
}

impl BitcoinDeserialize for HeadersMessage {
    type Error = NetworkDeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        let (headers_len, consumed) =
            read_compact_size(bytes).map_err(|_| NetworkDeserializeError::InvalidType)?;
        offset += consumed;

        let mut headers: Vec<BlockHeader> = Vec::new();

        for _ in 0..headers_len {
            let (header, consumed) = BlockHeader::deserialize(&bytes[offset..])
                .map_err(|_| NetworkDeserializeError::InvalidType)?;
            headers.push(header);
            offset += consumed;
        }

        Ok((Self { headers }, offset))
    }
}

impl BitcoinSerialize for GetHeadersMessage {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(self.version.to_le_bytes());
        bytes.extend(get_compact_size(self.locator_hashes.len()));

        for locator_hash in self.locator_hashes.iter() {
            bytes.extend(locator_hash.as_bytes());
        }

        bytes.extend(self.stop_hash.as_bytes());
        bytes
    }
}

impl BitcoinDeserialize for GetHeadersMessage {
    type Error = NetworkDeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        let version = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NetworkDeserializeError::InvalidType)?,
        );
        offset += 4;
        let (locators_len, consumed) = read_compact_size(&bytes[offset..])
            .map_err(|_| NetworkDeserializeError::InvalidType)?;
        offset += consumed;

        let mut locator_hashes: Vec<BlockHash> = Vec::new();
        for _ in 0..locators_len {
            locator_hashes.push(BlockHash(
                bytes[offset..offset + 32]
                    .try_into()
                    .map_err(|_| NetworkDeserializeError::InvalidType)?,
            ));
            offset += 32;
        }

        let stop_hash = BlockHash(
            bytes[offset..offset + 32]
                .try_into()
                .map_err(|_| NetworkDeserializeError::InvalidType)?,
        );
        offset += 32;

        Ok((
            Self {
                version,
                locator_hashes,
                stop_hash,
            },
            offset,
        ))
    }
}
