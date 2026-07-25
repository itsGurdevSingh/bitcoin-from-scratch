use crate::{
    crypto::sha256d,
    serialization::{
        BitcoinDeserialize, BitcoinSerialize, DeserializeError,
        compact_size::{get_compact_size, read_compact_size},
    },
    transaction::{TxInput, TxOutput, Witness},
    types::{TxId, WTxId},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u64,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            version: 0,
            inputs: vec![TxInput::new()],
            outputs: vec![TxOutput::new()],
            lock_time: 0,
        }
    }
}

impl BitcoinSerialize for Transaction {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.version.to_le_bytes());

        bytes.extend(get_compact_size(self.inputs.len()));

        for input in &self.inputs {
            bytes.extend(input.serialize());
        }

        bytes.extend(get_compact_size(self.outputs.len()));

        for output in &self.outputs {
            bytes.extend(output.serialize());
        }

        bytes.extend_from_slice(&self.lock_time.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for Transaction {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        if bytes.len() < offset + 4 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let version = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        let (inputs_len, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let mut inputs: Vec<TxInput> = Vec::new();

        for _ in 0..inputs_len {
            let (input, consumed) = TxInput::deserialize(&bytes[offset..])?;
            inputs.push(input);
            offset += consumed;
        }

        let (output_len, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let mut outputs: Vec<TxOutput> = Vec::new();

        for _ in 0..output_len {
            let (output, consumed) = TxOutput::deserialize(&bytes[offset..])?;
            outputs.push(output);
            offset += consumed;
        }

        if bytes.len() < offset + 8 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let lock_time = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 8;

        Ok((
            Self {
                version,
                inputs,
                outputs,
                lock_time,
            },
            offset,
        ))
    }
}

impl Transaction {
    pub fn serialize_witness(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.version.to_le_bytes());

        bytes.push(0x00); // marker
        bytes.push(0x01); // flag

        bytes.extend(get_compact_size(self.inputs.len()));

        for input in &self.inputs {
            bytes.extend(input.serialize());
        }

        bytes.extend(get_compact_size(self.outputs.len()));

        for output in &self.outputs {
            bytes.extend(output.serialize());
        }

        for input in &self.inputs {
            bytes.extend(input.witness.serialize());
        }

        bytes.extend_from_slice(&self.lock_time.to_le_bytes());

        bytes
    }

    pub fn deserialize_witness(bytes: &[u8]) -> Result<(Self, usize), DeserializeError> {
        let mut offset: usize = 0;

        if bytes.len() < offset + 4 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let version = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        // lagecy tx (no segwit tx)
        if bytes.len() <= offset {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        if bytes[offset] != 0x00 {
            return Self::deserialize(bytes);
        }

        offset += 1;

        // flag is wrong
        if bytes.len() <= offset {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        if bytes[offset] != 0x01 {
            return Err(DeserializeError::InvalidSegWitFlag(bytes[offset]));
        }
        offset += 1;

        let (inputs_len, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let mut inputs: Vec<TxInput> = Vec::new();

        for _ in 0..inputs_len {
            let (input, consumed) = TxInput::deserialize(&bytes[offset..])?;
            inputs.push(input);
            offset += consumed;
        }

        let (output_len, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let mut outputs: Vec<TxOutput> = Vec::new();

        for _ in 0..output_len {
            let (output, consumed) = TxOutput::deserialize(&bytes[offset..])?;
            outputs.push(output);
            offset += consumed;
        }

        // witness
        for i in 0..inputs_len {
            let (witness, consumed) = Witness::deserialize(&bytes[offset..])?;

            inputs[i].witness = witness;
            offset += consumed;
        }

        if bytes.len() < offset + 8 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let lock_time = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 8;

        Ok((
            Self {
                version,
                inputs,
                outputs,
                lock_time,
            },
            offset,
        ))
    }
}

impl Transaction {
    pub fn txid(&self) -> TxId {
        let bytes = self.serialize();
        TxId::from(sha256d(&bytes))
    }

    pub fn wtid(&self) -> WTxId {
        let bytes = self.serialize_witness();
        WTxId::from(sha256d(&bytes))
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1
            && self.inputs[0].previous_output.txid == TxId([0u8; 32])
            && self.inputs[0].previous_output.vout == 0xffffffff
    }
}

#[cfg(test)]
mod test {

    use crate::{
        ledger::Ledger,
        script::{OpCode, Script, ScriptItem},
        tests::dummy_tx::get_valid_tx,
        transaction::{OutPoint, Witness},
    };

    use super::*;

    #[test]
    fn same_transaction_same_txid() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res = transaction.txid();

        let res2 = transaction.txid();

        assert_eq!(res, res2);
    }

    #[test]
    fn different_transactions_different_txids() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input.clone()],
            outputs: vec![tx_output.clone()],
            lock_time: 1000,
        };
        let transaction2 = Transaction {
            version: 5,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res1 = transaction.txid();

        let res2 = transaction2.txid();

        assert_ne!(res1, res2);
    }

    #[test]
    fn serialize_then_deserilize_return_same_transaction() {
        let tx = get_valid_tx(&mut Ledger::new(), 50, 0, 40);

        let serialize_data = tx.serialize_witness();

        let (tx_res, _) = Transaction::deserialize_witness(&serialize_data).unwrap();

        assert_eq!(tx, tx_res)
    }

    #[test]
    fn deserialize_legacy_rejects_short_bytes() {
        let result = Transaction::deserialize(&[]);
        assert_eq!(result, Err(DeserializeError::UnexpectedEndOfBytes));
    }

    #[test]
    fn deserialize_witness_rejects_invalid_flag() {
        let bytes = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x02];

        let result = Transaction::deserialize_witness(&bytes);
        assert_eq!(result, Err(DeserializeError::InvalidSegWitFlag(0x02)));
    }

    #[test]
    fn deserialize_witness_rejects_truncated_lock_time() {
        let bytes = vec![
            0x01, 0x00, 0x00, 0x00, // version
            0x00, // marker
            0x01, // flag
            0x00, // input count
            0x00, // output count
            0x00, 0x00, 0x00, 0x00, // only 4 bytes of lock_time; expected 8
        ];

        let result = Transaction::deserialize_witness(&bytes);
        assert_eq!(result, Err(DeserializeError::UnexpectedEndOfBytes));
    }

    fn create_dummy_tx_input() -> TxInput {
        let sig_script_items: Vec<ScriptItem> = vec![
            ScriptItem::PushData(vec![0u8; 32]),
            ScriptItem::PushData(vec![0u8; 64]),
        ];

        let script_sig = Script {
            items: sig_script_items,
        };

        let previous_output = OutPoint {
            txid: TxId([0u8; 32]),
            vout: 8,
        };

        TxInput {
            previous_output,
            script_sig,
            witness: Witness { stack: Vec::new() },
            sequence: 5,
        }
    }

    fn create_dummy_tx_output() -> TxOutput {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(vec![0u8; 20]), // 20-byte dummy pubkey hash
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        let script: Script = Script {
            items: p2pkh_script,
        };

        TxOutput {
            value: 2,
            script_pub_key: script,
        }
    }
}
