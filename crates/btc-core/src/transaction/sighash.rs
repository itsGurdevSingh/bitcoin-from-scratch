use crate::{
    crypto::{sha256, sha256d},
    script::Script,
    serialization::{BitcoinSerialize, compact_size::get_compact_size},
    taproot::TaprootError,
    transaction::{SigHashError, SpendType, TaprootPrecomputed, WitnessPrecomputed},
    utxo::Utxo,
    virtual_machine::SigHashType,
};

use super::{Transaction, TxInput, Witness};

fn clear_signing_data(input: &mut TxInput) {
    input.script_sig.items.clear();
    input.witness = Witness::new();
}

fn clear_all_signing_data(transaction: &mut Transaction) {
    for input in transaction.inputs.iter_mut() {
        clear_signing_data(input);
    }
}

fn apply_script_code(transaction: &mut Transaction, input_index: usize, script_code: &Script) {
    transaction.inputs[input_index].script_sig = script_code.clone();
}

fn finalize_sig_hash(transaction: &Transaction, hash_type: SigHashType) -> [u8; 32] {
    let mut serial = transaction.serialize();
    serial.extend((hash_type as u32).to_le_bytes());
    sha256d(&serial)
}

fn single_sighash_placeholder() -> [u8; 32] {
    let mut arr = [0u8; 32];
    arr[31] = 1;
    arr
}

pub trait TransactionSigHash {
    fn signing_hash(
        &self,
        input_index: usize,
        script_code: &Script,
        hash_type: SigHashType,
    ) -> [u8; 32];
}

pub trait TransactionWitnessSigHash {
    fn signing_hash_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &WitnessPrecomputed,
        hash_type: SigHashType,
    ) -> Result<[u8; 32], SigHashError>;
}

pub trait TransactionTaprootSigHash {
    fn signing_hash_taproot(
        &self,
        input_index: usize,
        precompute: &TaprootPrecomputed,
        current_spending_utxo: &Utxo,
        hash_type: SigHashType,
        spend_type: SpendType,
    ) -> Result<[u8; 32], TaprootError>;
}

impl TransactionSigHash for Transaction {
    fn signing_hash(
        &self,
        input_index: usize,
        script_code: &Script,
        hash_type: SigHashType,
    ) -> [u8; 32] {
        // SIGHASH_SINGLE special case
        if hash_type.is_single() && input_index >= self.outputs.len() {
            return single_sighash_placeholder();
        }

        let mut tx = self.clone();

        // Clear signing data from every input.
        clear_all_signing_data(&mut tx);

        // Put scriptCode only into the input being signed.
        apply_script_code(&mut tx, input_index, script_code);

        //
        // INPUT MODIFICATIONS
        //

        if hash_type.is_anyone_can_pay() {
            tx.inputs = vec![tx.inputs[input_index].clone()];
        } else {
            for (idx, input) in tx.inputs.iter_mut().enumerate() {
                if idx != input_index && (hash_type.is_none() || hash_type.is_single()) {
                    input.sequence = 0;
                }
            }
        }

        //
        // OUTPUT MODIFICATIONS
        //

        if hash_type.is_none() {
            tx.outputs.clear();
        }

        if hash_type.is_single() {
            for (idx, output) in tx.outputs.iter_mut().enumerate() {
                if idx != input_index {
                    output.value = u64::MAX;
                    output.script_pub_key = Script::new();
                }
            }
        }

        finalize_sig_hash(&tx, hash_type)
    }
}
impl TransactionWitnessSigHash for Transaction {
    fn signing_hash_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &WitnessPrecomputed,
        hash_type: SigHashType,
    ) -> Result<[u8; 32], SigHashError> {
        let mut bytes = Vec::new();

        let hash_prevouts = if hash_type.is_anyone_can_pay() {
            [0u8; 32]
        } else {
            precompute.hash_prevouts
        };

        let hash_sequences =
            if hash_type.is_anyone_can_pay() || hash_type.is_none() || hash_type.is_single() {
                [0u8; 32]
            } else {
                precompute.hash_sequences
            };

        let hash_outputs = match hash_type {
            SigHashType::Default | SigHashType::All | SigHashType::AllAnyoneCanPay => {
                precompute.hash_outputs
            }

            SigHashType::None | SigHashType::NoneAnyoneCanPay => [0u8; 32],

            SigHashType::Single | SigHashType::SingleAnyoneCanPay => {
                let output = self
                    .outputs
                    .get(input_index)
                    .ok_or(SigHashError::SigningOutputNotExist)?;

                sha256d(&output.serialize())
            }
        };

        bytes.extend(self.version.to_le_bytes());
        bytes.extend(hash_prevouts);
        bytes.extend(hash_sequences);
        bytes.extend(self.inputs[input_index].previous_output.serialize());
        bytes.extend(script_code.serialize());
        bytes.extend(amount.to_le_bytes());
        bytes.extend(self.inputs[input_index].sequence.to_le_bytes());
        bytes.extend(hash_outputs);
        bytes.extend(self.lock_time.to_le_bytes());
        bytes.extend((hash_type as u32).to_le_bytes());

        Ok(sha256d(&bytes))
    }
}

impl TransactionTaprootSigHash for Transaction {
    fn signing_hash_taproot(
        &self,
        input_index: usize,
        precompute: &TaprootPrecomputed,
        current_spending_utxo: &Utxo,
        hash_type: SigHashType,
        spend_type: SpendType,
    ) -> Result<[u8; 32], TaprootError> {
        let mut bytes = Vec::new();
        bytes.push(hash_type.clone() as u8);
        bytes.extend(self.version.to_le_bytes());
        bytes.extend(self.lock_time.to_le_bytes());

        // non ACP
        if !hash_type.is_anyone_can_pay() {
            bytes.extend(precompute.hash_prevouts);
            bytes.extend(precompute.hash_amounts);
            bytes.extend(precompute.hash_scriptpubkeys);
            bytes.extend(precompute.hash_sequences);
        }

        match hash_type {
            SigHashType::Default | SigHashType::All | SigHashType::AllAnyoneCanPay => {
                bytes.extend(precompute.hash_outputs);
            }
            _ => {}
        }

        bytes.push(spend_type.as_u8());

        // ACP
        if hash_type.is_anyone_can_pay() {
            bytes.extend(self.inputs[input_index].previous_output.serialize());
            bytes.extend(current_spending_utxo.value.to_le_bytes());
            bytes.extend(current_spending_utxo.script_pub_key.serialize());
            bytes.extend(self.inputs[input_index].sequence.to_le_bytes());
        }

        if !hash_type.is_anyone_can_pay() {
            bytes.extend((input_index as u32).to_le_bytes());
        }

        if spend_type.has_annex() {
            if let Some(annex_bytes) = spend_type.get_annex_bytes() {
                let mut data = get_compact_size(annex_bytes.len());
                data.extend(annex_bytes);
                bytes.extend(sha256(&data));
            }
        }
        match hash_type {
            SigHashType::Single | SigHashType::SingleAnyoneCanPay => {
                if self.outputs.len() <= input_index {
                    Err(TaprootError::SigningOutputNotExist)?;
                }
                bytes.extend_from_slice(&sha256(&self.outputs[input_index].serialize()));
            }
            _ => {}
        }

        Ok(sha256(&bytes))
    }
}
