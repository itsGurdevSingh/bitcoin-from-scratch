use std::collections::HashSet;
type Fee = u64;

use crate::{
    block::BlockReward,
    blockchain::overlay::Overlay,
    crypto::sha256d,
    ledger::Ledger,
    merkle::MerkleTree,
    transaction::{OutPoint, Transaction},
    validator::{ValidationError, constant::COINBASE_MATURITY},
    virtual_machine::{ExecutionContext, ScriptType, ScriptVerifier, VirtualMachine},
};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate(
        tx: &Transaction,
        ledger: &Ledger,
        overlay: &Overlay,
        parent_height: u32,
    ) -> Result<Fee, ValidationError> {
        // check inputs exist output exist
        if tx.inputs.is_empty() {
            return Err(ValidationError::NoInputs);
        }
        if tx.outputs.is_empty() {
            return Err(ValidationError::NoOutputs);
        }

        let mut seen_inputs: HashSet<OutPoint> = HashSet::new();
        let mut total_input_value: u64 = 0;

        // no duplicate inputs are input has valid utxo from utxo set and total input value
        for (idx, input) in tx.inputs.iter().enumerate() {
            // is duplicate
            if !seen_inputs.insert(input.previous_output.clone()) {
                return Err(ValidationError::DuplicateInput);
            };

            // get utxo for input
            let res = overlay.lookup(ledger, &input.previous_output);

            let utxo = match res {
                Some(utxo) => {
                    total_input_value += utxo.value;
                    utxo
                }
                None => return Err(ValidationError::MissingUtxo),
            };

            if utxo.is_coinbase {
                let confirmations = (parent_height + 1) as i64 - utxo.block_height as i64;

                if confirmations < COINBASE_MATURITY as i64 {
                    return Err(ValidationError::PrematureCoinbaseSpend);
                }
            }

            // validate script
            match ScriptVerifier::verify(tx, idx, utxo) {
                Err(e) => return Err(ValidationError::ScriptVerificationFailed(e)),
                Ok(_) => {}
            }
        }

        // output and total value of outputs

        let mut vm = VirtualMachine::new(ExecutionContext::new()); //placeholder context
        let mut total_output_value: u64 = 0;
        for output in tx.outputs.iter() {
            if output.value == 0 {
                return Err(ValidationError::InvalidOutputValue);
            }
            total_output_value += output.value;

            if !vm.is_valid_script_pub_key(&output.script_pub_key) {
                return Err(ValidationError::InvalidCoinbaseTransaction);
            }
        }

        // is input values enough for  output
        if total_input_value < total_output_value {
            return Err(ValidationError::InsufficientInputValue);
        }

        let fee: Fee = total_input_value - total_output_value;

        Ok(fee)
    }

    pub fn validate_coinbase(
        transaction: &[Transaction],
        total_fees: u64,
        parent_height: u32,
    ) -> Result<Fee, ValidationError> {
        let coinbase_tx = &transaction[0];
        if !coinbase_tx.is_coinbase() {
            return Err(ValidationError::InvalidCoinbaseTransaction);
        };

        // check value is proper reward + total fees.
        if !(coinbase_tx.outputs[0].value
            == BlockReward::total_reward(parent_height + 1, total_fees))
        {
            return Err(ValidationError::InvalidCoinbaseTransaction);
        };

        if !coinbase_tx.inputs[0].witness.stack.is_empty(){

            let mut witness_merkle_root = MerkleTree::compute_root_witness_v0(&transaction[1..])
                .map_err(|_| ValidationError::InvalidCoinbaseTransaction)?
                .into_bytes()
                .to_vec();

            witness_merkle_root.extend([0u8; 32]);

            let witness_commitment = sha256d(&witness_merkle_root).to_vec();

            if !ScriptType::is_witness_commitment_script(
                &coinbase_tx.outputs[1].script_pub_key,
                &witness_commitment,
            ) {
                Err(ValidationError::InvalidCoinbaseTransaction)?;
            }
        }

        let mut vm = VirtualMachine::new(ExecutionContext::new());
        if !vm.is_valid_script_pub_key(&coinbase_tx.outputs[0].script_pub_key) {
            return Err(ValidationError::InvalidCoinbaseTransaction);
        }

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        crypto::{generate_keypair_dummy, hash::hash160, sign_tx},
        script::{OpCode, Script, ScriptItem},
        tests::dummy_tx::get_valid_tx,
        transaction::{TransactionSigHash, TxInput, TxOutput, Witness},
        types::TxId,
        utxo::Utxo,
        virtual_machine::SigHashType,
    };

    use super::*;

    #[test]
    fn valid_transaction() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let transaction = get_valid_tx(&mut ledger, 10, 0, 8);

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionValidator::validate(&transaction, &ledger, &overlay, 2000);

        // input is 10 and output is 8 fee should be
        // input - output = fee
        //   10  -   8    = 2

        assert_eq!(res, Ok(2));
    }
    #[test]
    fn missing_utxo() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(2);

        // add utxo to ledger to replicate they are valid and already their
        let ledger = Ledger::new();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionValidator::validate(&transaction, &ledger, &overlay, 10000);

        assert_eq!(res, Err(ValidationError::MissingUtxo));
    }
    #[test]
    fn duplicate_input() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input.clone(), tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();

        for idx in 0..transaction.inputs.len() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            let message = transaction.signing_hash(idx, &utxo.script_pub_key, SigHashType::All);

            let mut sig = sign_tx(&message, &sk).serialize_der().to_vec();
            sig.extend((SigHashType::All as u32).to_le_bytes());

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            transaction.inputs[idx].script_sig = script;

            // add valid utxo
            // we have to ignore error because on second duplicate utxo addition ledger thoug error.
            ledger
                .add_utxo(transaction.inputs[idx].previous_output.clone(), utxo)
                .unwrap_or_else(|_| return);
        }

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionValidator::validate(&transaction, &ledger, &overlay, 20000);

        assert_eq!(res, Err(ValidationError::DuplicateInput));
    }
    #[test]
    fn insufficient_input_value() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        for idx in 0..transaction.inputs.len() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let utxo = create_dummy_utxo(1, hash160(&pk.serialize().to_vec()).to_vec());

            let message = transaction.signing_hash(idx, &utxo.script_pub_key, SigHashType::All);

            let mut sig = sign_tx(&message, &sk).serialize_der().to_vec();
            sig.extend((SigHashType::All as u32).to_le_bytes());

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            transaction.inputs[idx].script_sig = script;

            // add valid utxo
            ledger
                .add_utxo(transaction.inputs[idx].previous_output.clone(), utxo)
                .unwrap();
        }

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionValidator::validate(&transaction, &ledger, &overlay, 2000);

        assert_eq!(res, Err(ValidationError::InsufficientInputValue));
    }
    #[test]
    fn no_inputs() {
        let tx_output = create_dummy_tx_output(20);

        // add utxo to ledger to replicate they are valid and already their
        let ledger = Ledger::new();

        let transaction = Transaction {
            version: 10,
            inputs: vec![],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionValidator::validate(&transaction, &ledger, &overlay, 2000);

        assert_eq!(res, Err(ValidationError::NoInputs));
    }
    #[test]
    fn no_outputs() {
        let tx_input = create_dummy_tx_input();

        // add utxo to ledger to replicate they are valid and already their
        let mut ledger = Ledger::new();

        let utxo = create_dummy_utxo(10, vec![1, 22, 2]);

        ledger
            .add_utxo(tx_input.clone().previous_output, utxo)
            .unwrap();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![],
            lock_time: 1000,
        };

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionValidator::validate(&transaction, &ledger, &overlay, 2000);

        assert_eq!(res, Err(ValidationError::NoOutputs));
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
            witness: Witness::new(),
            sequence: 5,
        }
    }

    fn create_dummy_tx_output(val: u64) -> TxOutput {
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
            value: val,
            script_pub_key: script,
        }
    }

    fn create_dummy_utxo(val: u64, pkh: Vec<u8>) -> Utxo {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(pkh),
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        Utxo {
            value: val,
            script_pub_key: Script {
                items: p2pkh_script,
            },
            is_coinbase: false,
            block_height: 1000,
        }
    }
}
