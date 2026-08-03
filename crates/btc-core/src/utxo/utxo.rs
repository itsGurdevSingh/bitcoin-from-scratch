use crate::{
    script::Script,
    serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
};

/// Represents an Unspent Transaction Output (UTXO).
///
/// A UTXO is the fundamental unit of ownership in Bitcoin.
/// Unlike account-based systems, Bitcoin does not track balances.
/// Instead, it tracks a set of spendable outputs.
///
/// A UTXO contains:
/// - The amount of value it holds (in satoshis)
/// - The spending conditions required to unlock it
/// - Metadata needed for consensus validation
///
/// UTXOs are identified externally by an OutPoint:
/// `(txid, vout)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    /// Amount stored in this UTXO, denominated in satoshis.
    pub value: u64,

    /// Locking script that defines the conditions required
    /// to spend this UTXO.
    pub script_pub_key: Script,

    /// Indicates whether this UTXO was created by a coinbase
    /// transaction (block reward).
    ///
    /// Coinbase outputs require 100 confirmations before
    /// they become spendable.
    pub is_coinbase: bool,

    /// Block height at which this UTXO was created.
    ///
    /// Used for rules such as coinbase maturity and
    /// height-based validation.
    pub block_height: u32,
}

impl Utxo {
    pub fn new() -> Self {
        Self {
            value: 0,
            script_pub_key: Script::new(),
            is_coinbase: false,
            block_height: 0,
        }
    }
}

impl BitcoinSerialize for Utxo {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.value.to_le_bytes());
        bytes.extend(self.script_pub_key.serialize());
        bytes.push(if self.is_coinbase { 1 } else { 0 });
        bytes.extend_from_slice(&self.block_height.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for Utxo {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        if bytes.len() < offset + 8 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let value = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 8;

        let (script_pub_key, consumed) = Script::deserialize(&bytes[offset..])?;
        offset += consumed;

        if bytes.len() < offset + 4 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let is_coinbase = bytes[offset] != 0;
        offset += 1;

        let block_height = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        Ok((
            Self {
                value,
                script_pub_key,
                is_coinbase,
                block_height,
            },
            offset,
        ))
    }
}
