use std::collections::HashMap;

use crate::{
    crypto::{hash::hash160, schnorr::verify_signature_tr, sha256},
    script::{OpCode, Script, ScriptItem},
    serialization::BitcoinDeserialize,
    taproot::sighash::taproot_sighash,
    transaction::{
        OutPoint, SpendType, Transaction, pre_compute_tx_data::PrecomputedData,
        sighash::TransactionTaprootSigHash,
    },
    utxo::Utxo,
    virtual_machine::{
        ExecutionContext, ScriptType, SigHashType, SigVersion, VirtualMachine, VmError,
    },
};

pub struct ScriptVerifier;

impl ScriptVerifier {
    pub fn verify_transaction_scripts(
        transaction: &Transaction,
        utxo_set: &HashMap<&OutPoint, &Utxo>,
    ) -> Result<(), VmError> {
        let mut spending_utxo = Vec::new();

        for (_outpoint, utxo) in utxo_set.iter() {
            spending_utxo.push(*utxo);
        }

        let precompute_data = PrecomputedData::new(transaction, &spending_utxo);

        for (idx, input) in transaction.inputs.iter().enumerate() {
            let utxo = *utxo_set
                .get(&input.previous_output)
                .ok_or(VmError::InvalidScriptFormat)?;

            Self::verify(transaction, idx, &precompute_data, utxo)?;
        }

        Ok(())
    }

    pub fn verify(
        transaction: &Transaction,
        input_index: usize,
        precompute_data: &PrecomputedData,
        utxo: &Utxo,
    ) -> Result<(), VmError> {
        let script_pub = utxo.script_pub_key.clone();
        let script_sig = transaction.inputs[input_index].script_sig.clone();

        match ScriptType::is_type_of(&script_pub, &script_sig) {
            ScriptType::P2PKH => ScriptVerifier::execute_p2pkh(transaction, input_index, utxo),
            ScriptType::P2SH => ScriptVerifier::execute_p2sh(transaction, input_index, utxo),
            ScriptType::P2WPKH => {
                ScriptVerifier::execute_p2wpkh_script(transaction, input_index, utxo)
            }
            ScriptType::P2WSH => {
                ScriptVerifier::execute_p2wsh_script(transaction, input_index, utxo)
            }
            ScriptType::P2TR => {
                ScriptVerifier::execute_p2tr_script(transaction, input_index, precompute_data, utxo)
            }
            ScriptType::None => Err(VmError::InvalidScriptFormat),
        }
    }

    fn execute_p2pkh(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
    ) -> Result<(), VmError> {
        let script_pub = utxo.script_pub_key.clone();
        let script_sig = transaction.inputs[input_index].script_sig.clone();

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code: script_pub, // main execuatble script
            sig_version: SigVersion::Legacy,
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_script_sig(&script_sig)?;
        vm.execute_script()
    }

    fn execute_p2sh(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
    ) -> Result<(), VmError> {
        let mut pub_script = utxo.script_pub_key.clone();
        let mut script_sig = transaction.inputs[input_index].script_sig.clone();

        let redeem_script_bytes = script_sig
            .items
            .pop()
            .ok_or(VmError::InvalidScriptFormat)?
            .get_bytes()
            .ok_or(VmError::InvalidScriptFormat)?
            .to_vec();
        let redeem_script_hash = pub_script
            .items
            .pop()
            .ok_or(VmError::InvalidScriptFormat)?
            .get_bytes()
            .ok_or(VmError::InvalidScriptFormat)?
            .to_vec();

        if hash160(&redeem_script_bytes).as_slice() != redeem_script_hash {
            Err(VmError::VerifyFailed)?;
        }

        let (script_code, consumed) =
            Script::deserialize(&redeem_script_bytes).map_err(|_| VmError::InvalidScriptFormat)?;

        if redeem_script_bytes.len() != consumed {
            Err(VmError::InvalidScriptFormat)?;
        }

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code: script_code.clone(), // main execuatble script
            sig_version: SigVersion::Legacy,
        };
        let mut vm = VirtualMachine::new(execution_context);

        match ScriptType::is_type_of(&script_code, &script_sig) {
            ScriptType::P2WPKH | ScriptType::P2WSH => {
                let witness = transaction.inputs[input_index].witness.clone();
                vm.load_witness(&witness)?;
            }
            ScriptType::P2PKH => {
                vm.load_script_sig(&script_sig)?;
            }
            _ => Err(VmError::InvalidScriptFormat)?,
        }

        vm.execute_script()
    }

    pub fn execute_p2wsh_script(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
    ) -> Result<(), VmError> {
        if transaction.inputs[input_index].script_sig.items.is_empty() {
            return Err(VmError::InvalidScriptFormat);
        }

        let script_pub = utxo.script_pub_key.clone();
        let mut witness = transaction.inputs[input_index].witness.clone();

        if witness.stack.is_empty() {
            return Err(VmError::InvalidScriptFormat);
        }

        let redeem_script_bytes = witness.stack.pop().ok_or(VmError::InvalidScriptFormat)?;

        // first verify script hash
        let redeem_script_hash = script_pub
            .items
            .last()
            .ok_or(VmError::InvalidScriptFormat)?
            .get_bytes()
            .ok_or(VmError::InvalidScriptFormat)?;

        if sha256(&redeem_script_bytes) != redeem_script_hash {
            return Err(VmError::InvalidScriptFormat);
        };
        // run actual script
        let (script_code, consumed) =
            Script::deserialize(&redeem_script_bytes).map_err(|_| VmError::InvalidScriptFormat)?;
        if consumed != redeem_script_bytes.len() {
            return Err(VmError::InvalidScriptFormat);
        };

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code, // main execuatble script
            sig_version: SigVersion::WitnessV0,
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_witness(&witness)?;
        vm.execute_script()
    }

    pub fn execute_p2wpkh_script(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
    ) -> Result<(), VmError> {
        if transaction.inputs[input_index].script_sig.items.is_empty() {
            return Err(VmError::InvalidScriptFormat);
        }

        let script_pub = utxo.script_pub_key.clone();
        let witness = transaction.inputs[input_index].witness.clone();

        let pub_key = script_pub
            .items
            .last()
            .ok_or(VmError::InvalidScriptFormat)?;

        let pub_key_bytes = pub_key.get_bytes().ok_or(VmError::InvalidScriptFormat)?;
        let script_code = Self::create_p2pkh_script(pub_key_bytes.to_vec());

        if witness.stack.len() != 2 {
            return Err(VmError::InvalidScriptFormat);
        }

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code, // main execuatble script
            sig_version: SigVersion::WitnessV0,
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_witness(&witness)?;
        vm.execute_script()
    }

    pub fn execute_p2tr_script(
        transaction: &Transaction,
        input_index: usize,
        precompute_data: &PrecomputedData,
        utxo: &Utxo,
    ) -> Result<(), VmError> {
        let spend_type = SpendType::get_spent_type(&transaction.inputs[input_index].witness)
            .ok_or(VmError::InvalidScriptFormat)?;

        if spend_type == SpendType::KeyPath {
            let mut signature = transaction.inputs[input_index].witness.stack[0].clone();
            let hash_type =
                SigHashType::try_from(signature.pop().ok_or(VmError::InvalidScriptFormat)? as u32)
                    .map_err(|_| VmError::InvalidScriptFormat)?;

            let message = taproot_sighash(
                &transaction
                    .signing_hash_taproot(
                        input_index,
                        &precompute_data.taproot_precompute,
                        utxo,
                        hash_type,
                        spend_type,
                    )
                    .map_err(|e| VmError::Taproot(e))?,
            );

            let xonly_public_key = utxo.script_pub_key.items[1]
                .get_bytes()
                .ok_or(VmError::InvalidScriptFormat)?;

            match verify_signature_tr(xonly_public_key, &message, &signature) {
                true => return Ok(()),
                false => Err(VmError::VerifyFailed)?,
            };
        };

        Err(VmError::InvalidData) // we only implement our keypath for taproot its ofr testing purpose
    }

    fn create_p2pkh_script(pub_key_bytes: Vec<u8>) -> Script {
        Script {
            items: vec![
                ScriptItem::Op(OpCode::Dup),
                ScriptItem::Op(OpCode::Hash160),
                ScriptItem::PushData(pub_key_bytes),
                ScriptItem::Op(OpCode::EqualVerify),
                ScriptItem::Op(OpCode::CheckSig),
            ],
        }
    }
}
