use crate::{
    crypto::sha256d, script::Script, serialization::BitcoinSerialize,
    transaction::PrecomputedTransactionData, virtual_machine::SigHashType,
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

fn finalize_witness_v0_sig_hash(
    transaction: &Transaction,
    input_index: usize,
    amount: u64,
    script_code: &Script,
    hash_prevouts: [u8; 32],
    hash_sequence: [u8; 32],
    hash_outputs: [u8; 32],
    hash_type: SigHashType,
) -> [u8; 32] {
    let mut bytes = Vec::new();

    bytes.extend(transaction.version.to_le_bytes());
    bytes.extend(hash_prevouts);
    bytes.extend(hash_sequence);
    bytes.extend(transaction.inputs[input_index].previous_output.serialize());
    bytes.extend(script_code.serialize());
    bytes.extend(amount.to_le_bytes());
    bytes.extend(transaction.inputs[input_index].sequence.to_le_bytes());
    bytes.extend(hash_outputs);
    bytes.extend(transaction.lock_time.to_le_bytes());
    bytes.extend((hash_type as u32).to_le_bytes());

    sha256d(&bytes)
}

pub trait TransactionSigHash {
    fn signing_hash(
        &self,
        input_index: usize,
        script_code: &Script,
        hash_type: SigHashType,
    ) -> [u8; 32];

    fn sig_hash_all(&self, input_index: usize, script_code: &Script) -> [u8; 32];

    fn sig_hash_none(&self, input_index: usize, script_code: &Script) -> [u8; 32];

    fn sig_hash_single(&self, input_index: usize, script_code: &Script) -> [u8; 32];

    fn sig_hash_anyone_can_pay_all(&self, input_index: usize, script_code: &Script) -> [u8; 32];

    fn sig_hash_anyone_can_pay_none(&self, input_index: usize, script_code: &Script) -> [u8; 32];

    fn sig_hash_anyone_can_pay_single(&self, input_index: usize, script_code: &Script) -> [u8; 32];
}

pub trait TransactionWitnessSigHash {
    fn signing_hash_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
        hash_type: SigHashType,
    ) -> [u8; 32];

    fn sig_hash_all_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32];

    fn sig_hash_none_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32];

    fn sig_hash_single_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32];

    fn sig_hash_anyone_can_pay_all_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32];

    fn sig_hash_anyone_can_pay_none_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32];

    fn sig_hash_anyone_can_pay_single_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32];
}

impl TransactionSigHash for Transaction {
    fn signing_hash(
        &self,
        input_index: usize,
        script_code: &Script,
        hash_type: SigHashType,
    ) -> [u8; 32] {
        match hash_type {
            SigHashType::All => self.sig_hash_all(input_index, script_code),
            SigHashType::None => self.sig_hash_none(input_index, script_code),
            SigHashType::Single => self.sig_hash_single(input_index, script_code),
            SigHashType::AllAnyoneCanPay => {
                self.sig_hash_anyone_can_pay_all(input_index, script_code)
            }
            SigHashType::NoneAnyoneCanPay => {
                self.sig_hash_anyone_can_pay_none(input_index, script_code)
            }
            SigHashType::SingleAnyoneCanPay => {
                self.sig_hash_anyone_can_pay_single(input_index, script_code)
            }
        }
    }

    fn sig_hash_all(&self, input_index: usize, script_code: &Script) -> [u8; 32] {
        let mut clone = self.clone();
        clear_all_signing_data(&mut clone);
        apply_script_code(&mut clone, input_index, script_code);
        finalize_sig_hash(&clone, SigHashType::All)
    }

    fn sig_hash_none(&self, input_index: usize, script_code: &Script) -> [u8; 32] {
        let mut clone = self.clone();

        for (idx, input) in clone.inputs.iter_mut().enumerate() {
            clear_signing_data(input);

            if idx != input_index {
                input.sequence = 0;
            }
        }

        apply_script_code(&mut clone, input_index, script_code);
        clone.outputs = Vec::new();

        finalize_sig_hash(&clone, SigHashType::None)
    }

    fn sig_hash_single(&self, input_index: usize, script_code: &Script) -> [u8; 32] {
        if input_index >= self.outputs.len() {
            return single_sighash_placeholder();
        }

        let mut clone = self.clone();

        for (idx, input) in clone.inputs.iter_mut().enumerate() {
            if idx != input_index {
                input.sequence = 0;
                continue;
            }

            clear_signing_data(input);
            input.script_sig = script_code.clone();
        }

        for (idx, output) in clone.outputs.iter_mut().enumerate() {
            if idx == input_index {
                continue;
            }

            output.value = 0xffffffffffffffff;
            output.script_pub_key = Script::new();
        }

        finalize_sig_hash(&clone, SigHashType::Single)
    }

    fn sig_hash_anyone_can_pay_all(&self, input_index: usize, script_code: &Script) -> [u8; 32] {
        let mut clone = self.clone();
        clear_signing_data(&mut clone.inputs[input_index]);
        apply_script_code(&mut clone, input_index, script_code);
        clone.inputs = vec![clone.inputs[input_index].clone()];

        finalize_sig_hash(&clone, SigHashType::AllAnyoneCanPay)
    }

    fn sig_hash_anyone_can_pay_none(&self, input_index: usize, script_code: &Script) -> [u8; 32] {
        let mut clone = self.clone();
        clear_signing_data(&mut clone.inputs[input_index]);
        apply_script_code(&mut clone, input_index, script_code);
        clone.inputs = vec![clone.inputs[input_index].clone()];
        clone.outputs = Vec::new();

        finalize_sig_hash(&clone, SigHashType::NoneAnyoneCanPay)
    }

    fn sig_hash_anyone_can_pay_single(&self, input_index: usize, script_code: &Script) -> [u8; 32] {
        if input_index >= self.outputs.len() {
            return single_sighash_placeholder();
        }

        let mut clone = self.clone();
        clear_signing_data(&mut clone.inputs[input_index]);
        apply_script_code(&mut clone, input_index, script_code);
        clone.inputs = vec![clone.inputs[input_index].clone()];
        clone.outputs = vec![clone.outputs[input_index].clone()];

        finalize_sig_hash(&clone, SigHashType::SingleAnyoneCanPay)
    }
}

impl TransactionWitnessSigHash for Transaction {
    fn signing_hash_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
        hash_type: SigHashType,
    ) -> [u8; 32] {
        match hash_type {
            SigHashType::All => {
                self.sig_hash_all_witness_v0(input_index, amount, script_code, precompute)
            }
            SigHashType::None => {
                self.sig_hash_none_witness_v0(input_index, amount, script_code, precompute)
            }
            SigHashType::Single => {
                self.sig_hash_single_witness_v0(input_index, amount, script_code, precompute)
            }
            SigHashType::AllAnyoneCanPay => self.sig_hash_anyone_can_pay_all_witness_v0(
                input_index,
                amount,
                script_code,
                precompute,
            ),
            SigHashType::NoneAnyoneCanPay => self.sig_hash_anyone_can_pay_none_witness_v0(
                input_index,
                amount,
                script_code,
                precompute,
            ),
            SigHashType::SingleAnyoneCanPay => self.sig_hash_anyone_can_pay_single_witness_v0(
                input_index,
                amount,
                script_code,
                precompute,
            ),
        }
    }

    fn sig_hash_all_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32] {
        finalize_witness_v0_sig_hash(
            self,
            input_index,
            amount,
            script_code,
            precompute.hash_prevouts,
            precompute.hash_sequence,
            precompute.hash_outputs,
            SigHashType::All,
        )
    }

    fn sig_hash_none_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32] {
        finalize_witness_v0_sig_hash(
            self,
            input_index,
            amount,
            script_code,
            precompute.hash_prevouts,
            precompute.hash_sequence,
            [0u8; 32],
            SigHashType::None,
        )
    }

    fn sig_hash_single_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32] {
        let output_bytes = if input_index >= self.outputs.len() {
            sha256d(&single_sighash_placeholder())
        } else {
            sha256d(&self.outputs[input_index].serialize())
        };

        finalize_witness_v0_sig_hash(
            self,
            input_index,
            amount,
            script_code,
            precompute.hash_prevouts,
            precompute.hash_sequence,
            output_bytes,
            SigHashType::Single,
        )
    }

    fn sig_hash_anyone_can_pay_all_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        precompute: &PrecomputedTransactionData,
    ) -> [u8; 32] {
        finalize_witness_v0_sig_hash(
            self,
            input_index,
            amount,
            script_code,
            [0u8; 32],
            [0u8; 32],
            precompute.hash_outputs,
            SigHashType::All,
        )
    }

    fn sig_hash_anyone_can_pay_none_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        _precompute: &PrecomputedTransactionData,
    ) -> [u8; 32] {
        finalize_witness_v0_sig_hash(
            self,
            input_index,
            amount,
            script_code,
            [0u8; 32],
            [0u8; 32],
            [0u8; 32],
            SigHashType::None,
        )
    }

    fn sig_hash_anyone_can_pay_single_witness_v0(
        &self,
        input_index: usize,
        amount: u64,
        script_code: &Script,
        _precompute: &PrecomputedTransactionData,
    ) -> [u8; 32] {
        let output_bytes = if input_index >= self.outputs.len() {
            sha256d(&single_sighash_placeholder())
        } else {
            sha256d(&self.outputs[input_index].serialize())
        };
        finalize_witness_v0_sig_hash(
            self,
            input_index,
            amount,
            script_code,
            [0u8; 32],
            [0u8; 32],
            output_bytes,
            SigHashType::Single,
        )
    }
}
