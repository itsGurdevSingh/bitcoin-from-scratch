use crate::{serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError, compact_size::{get_compact_size, read_compact_size}}, transaction::OutPoint, utxo::Utxo};

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct SpentUtxo {
    pub outpoint: OutPoint,
    pub utxo: Utxo,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct CreatedUtxo {
    pub outpoint: OutPoint,
    pub utxo: Utxo,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StateTransition {
    pub spent_utxos: Vec<SpentUtxo>,
    pub created_utxos: Vec<CreatedUtxo>,
    pub fee: u64,
}

impl BitcoinSerialize for StateTransition {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(get_compact_size(self.spent_utxos.len()));
        for sp in self.spent_utxos.iter() {
            bytes.extend(sp.outpoint.serialize());
            bytes.extend(sp.utxo.serialize());
        };

        bytes.extend(get_compact_size(self.created_utxos.len()));
        for cp in self.created_utxos.iter() {
            bytes.extend(cp.outpoint.serialize());
            bytes.extend(cp.utxo.serialize());
        };

        bytes.extend(self.fee.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for StateTransition {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut spent_utxos: Vec<SpentUtxo> = Vec::new();
        let mut created_utxos: Vec<CreatedUtxo> = Vec::new();

        let mut offset: usize = 0;

        let (spent_len, consumed) = read_compact_size(bytes)?;
        offset += consumed;

        for _ in 0..spent_len {
            let (outpoint, consumed) = OutPoint::deserialize(&bytes[offset..])?;
            offset += consumed;
            let (utxo, consumed) = Utxo::deserialize(&bytes[offset..])?;
            offset += consumed;
            spent_utxos.push(SpentUtxo { outpoint, utxo });
        };

        let (create_len, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        for _ in 0..create_len {
            let (outpoint, consumed) = OutPoint::deserialize(&bytes[offset..])?;
            offset += consumed;
            let (utxo, consumed) = Utxo::deserialize(&bytes[offset..])?;
            offset += consumed;
            created_utxos.push(CreatedUtxo { outpoint, utxo });
        };

        if bytes.len() < offset + 8 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        };

        let fee = u64::from_le_bytes(bytes[offset..offset+8].try_into().map_err(|_| DeserializeError::InvalidCompactSize)?);

        Ok((
            Self {
                created_utxos,
                spent_utxos,
                fee
            },
            offset
        ))

    }
}
