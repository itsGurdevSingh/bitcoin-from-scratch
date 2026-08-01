use std::collections::HashMap;

use crate::{
    crypto::{hash::hash160, schnorr::verify_signature_tr, sha256},
    script::{OpCode, Script, ScriptItem},
    serialization::BitcoinDeserialize,
    taproot::{ControlBlock, sighash::taproot_sighash},
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
                .ok_or(VmError::MissingUtxo)?;

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
            ScriptType::P2PKH => {
                ScriptVerifier::execute_p2pkh(transaction, input_index, utxo, precompute_data)
            }
            ScriptType::P2SH => {
                ScriptVerifier::execute_p2sh(transaction, input_index, utxo, precompute_data)
            }
            ScriptType::P2WPKH => ScriptVerifier::execute_p2wpkh_script(
                transaction,
                input_index,
                utxo,
                precompute_data,
            ),
            ScriptType::P2WSH => ScriptVerifier::execute_p2wsh_script(
                transaction,
                input_index,
                utxo,
                precompute_data,
            ),
            ScriptType::P2TR => {
                ScriptVerifier::execute_p2tr_script(transaction, input_index, utxo, precompute_data)
            }
            ScriptType::None => Err(VmError::InvalidScriptFormat),
        }
    }

    fn execute_p2pkh(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        let script_pub = utxo.script_pub_key.clone();
        let script_sig = transaction.inputs[input_index].script_sig.clone();

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code: script_pub, // main execuatble script
            sig_version: SigVersion::Legacy,
            precompute: precompute_data.clone(),
            current_spending_utxo: utxo.clone(),
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_script_sig(&script_sig)?;
        vm.execute_script()
    }

    fn execute_p2sh(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        let mut pub_script = utxo.script_pub_key.clone();
        let mut script_sig = transaction.inputs[input_index].script_sig.clone();

        let redeem_script_bytes = script_sig
            .items
            .pop()
            .ok_or(VmError::MissingRedeemScript)?
            .get_bytes()
            .ok_or(VmError::InvalidRedeemScript)?
            .to_vec();
        let redeem_script_hash = pub_script
            .items
            .pop()
            .ok_or(VmError::InvalidRedeemScript)?
            .get_bytes()
            .ok_or(VmError::InvalidRedeemScript)?
            .to_vec();

        if hash160(&redeem_script_bytes).as_slice() != redeem_script_hash {
            return Err(VmError::RedeemScriptHashMismatch);
        }

        let (script_code, consumed) =
            Script::deserialize(&redeem_script_bytes).map_err(|_| VmError::InvalidRedeemScript)?;

        if redeem_script_bytes.len() != consumed {
            return Err(VmError::InvalidRedeemScriptLength);
        }

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code: script_code.clone(), // main execuatble script
            sig_version: SigVersion::Legacy,
            precompute: precompute_data.clone(),
            current_spending_utxo: utxo.clone(),
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
            _ => return Err(VmError::InvalidScriptFormat),
        }

        vm.execute_script()
    }

    pub fn execute_p2wsh_script(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        if !transaction.inputs[input_index].script_sig.items.is_empty() {
            return Err(VmError::P2wshScriptSigNotAllowed);
        }

        let script_pub = utxo.script_pub_key.clone();
        let mut witness = transaction.inputs[input_index].witness.clone();

        if witness.stack.is_empty() {
            return Err(VmError::MissingWitnessScript);
        }

        let redeem_script_bytes = witness.stack.pop().ok_or(VmError::MissingWitnessScript)?;

        let redeem_script_hash = script_pub
            .items
            .last()
            .ok_or(VmError::InvalidScriptFormat)?
            .get_bytes()
            .ok_or(VmError::InvalidScriptFormat)?;

        if sha256(&redeem_script_bytes) != redeem_script_hash {
            return Err(VmError::WitnessScriptHashMismatch);
        }

        let (script_code, consumed) =
            Script::deserialize(&redeem_script_bytes).map_err(|_| VmError::InvalidRedeemScript)?;
        if consumed != redeem_script_bytes.len() {
            return Err(VmError::InvalidRedeemScriptLength);
        }

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code, // main execuatble script
            sig_version: SigVersion::WitnessV0,
            precompute: precompute_data.clone(),
            current_spending_utxo: utxo.clone(),
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_witness(&witness)?;
        vm.execute_script()
    }

    pub fn execute_p2wpkh_script(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        if !transaction.inputs[input_index].script_sig.items.is_empty() {
            return Err(VmError::P2wpkhScriptSigNotAllowed);
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
            return Err(VmError::InvalidWitnessStackSize);
        }

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code, // main execuatble script
            sig_version: SigVersion::WitnessV0,
            precompute: precompute_data.clone(),
            current_spending_utxo: utxo.clone(),
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_witness(&witness)?;
        vm.execute_script()
    }

    pub fn execute_p2tr_script(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        let spend_type = SpendType::get_spent_type(&transaction.inputs[input_index].witness)
            .ok_or(VmError::InvalidTaprootSpendType)?;

        match spend_type {
            SpendType::KeyPath => {
                Self::execute_p2tr_key_path(transaction, input_index, utxo, precompute_data)
            }
            SpendType::ScriptPath => {
                Self::execute_p2tr_script_path(transaction, input_index, utxo, precompute_data)
            }
            SpendType::KeyPathAnnex(_) | SpendType::ScriptPathAnnex(_) => {
                Err(VmError::InvalidTaprootSpendType)
            }
        }
    }

    fn execute_p2tr_key_path(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        let witness = transaction.inputs[input_index].witness.clone();
        let signature = witness
            .stack
            .first()
            .ok_or(VmError::MissingTaprootSignature)?
            .clone();

        let xonly_public_key = utxo.script_pub_key.items[1]
            .get_bytes()
            .ok_or(VmError::InvalidScriptFormat)?;

        let mut signature = signature;
        let hash_type =
            SigHashType::try_from(signature.pop().ok_or(VmError::MissingTaprootSignature)? as u32)
                .map_err(|_| VmError::InvalidScriptFormat)?;

        let message = taproot_sighash(
            &transaction
                .signing_hash_taproot(
                    input_index,
                    &precompute_data.taproot_precompute,
                    utxo,
                    hash_type,
                    SpendType::KeyPath,
                )
                .map_err(VmError::Taproot)?,
        );

        if verify_signature_tr(xonly_public_key, &message, &signature) {
            Ok(())
        } else {
            Err(VmError::TaprootSignatureVerificationFailed)
        }
    }

    fn execute_p2tr_script_path(
        transaction: &Transaction,
        input_index: usize,
        utxo: &Utxo,
        precompute_data: &PrecomputedData,
    ) -> Result<(), VmError> {
        let mut witness = transaction.inputs[input_index].witness.clone();

        let control_block_bytes = witness
            .stack
            .pop()
            .ok_or(VmError::MissingTaprootControlBlock)?;

        let tap_script_bytes = witness.stack.pop().ok_or(VmError::MissingTaprootScript)?;

        let xonly_public_key = utxo.script_pub_key.items[1]
            .get_bytes()
            .ok_or(VmError::InvalidScriptFormat)?;

        let (tap_script, _) =
            Script::deserialize(&tap_script_bytes).map_err(|_| VmError::InvalidRedeemScript)?;
        let (control_block, _) = ControlBlock::deserialize(&control_block_bytes)
            .map_err(|_| VmError::InvalidScriptFormat)?;

        if !ControlBlock::verify_proof(&tap_script, &control_block, &xonly_public_key) {
            return Err(VmError::TaprootCommitmentMismatch);
        }

        let execution_context = ExecutionContext {
            transaction: transaction.clone(),
            input_index,
            prevout_value: utxo.value,
            script_code: tap_script,
            sig_version: SigVersion::Taproot,
            precompute: precompute_data.clone(),
            current_spending_utxo: utxo.clone(),
        };

        let mut vm = VirtualMachine::new(execution_context);
        vm.load_witness(&witness)?;

        vm.execute_script()
    }

    fn create_p2pkh_script(pub_key_bytes: Vec<u8>) -> Script {
        let pub_key_hash = hash160(&pub_key_bytes);
        Script {
            items: vec![
                ScriptItem::Op(OpCode::Dup),
                ScriptItem::Op(OpCode::Hash160),
                ScriptItem::PushData(pub_key_hash.to_vec()),
                ScriptItem::Op(OpCode::EqualVerify),
                ScriptItem::Op(OpCode::CheckSig),
            ],
        }
    }
}
