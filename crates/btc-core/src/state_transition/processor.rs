use crate::{
    ledger::Ledger,
    state_transition::{CreatedUtxo, ProcessorError, SpentUtxo, StateTransition},
    transaction::{OutPoint, Transaction},
    utxo::Utxo,
    validator::TransactionValidator,
};

pub struct TransactionProcessor;

impl TransactionProcessor {
    pub fn process(
        tx: &Transaction,
        ledger: &Ledger,
        block_height: u32,
    ) -> Result<StateTransition, ProcessorError> {
        let fee = TransactionValidator::validate(tx, ledger)
            .map_err(|e| ProcessorError::Validation(e))?;

        let mut state: StateTransition = StateTransition {
            spent_utxos: vec![],
            created_utxos: vec![],
            fee,
        };
        for input in tx.inputs.iter() {
            let spent_outpoint = &input.previous_output;

            let spent_utxo = ledger
                .get_utxo(&spent_outpoint)
                .ok_or(ProcessorError::MissingUtxo)?;

            let spent = SpentUtxo {
                outpoint: spent_outpoint.clone(),
                utxo: spent_utxo.clone(),
            };

            state.spent_utxos.push(spent);
        }

        let txid = tx.txid();

        for (index, output) in tx.outputs.iter().enumerate() {
            let created_utxo = CreatedUtxo {
                outpoint: OutPoint {
                    txid,
                    vout: index as u32,
                },
                utxo: Utxo {
                    value: output.value,
                    script_pub_key: output.script_pub_key.clone(),
                    is_coinbase: false,
                    block_height,
                },
            };

            state.created_utxos.push(created_utxo);
        }

        Ok(state)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        crypto::{generate_keypair_dummy, hash::hash160, sign_tx}, script::{OpCode, Script, ScriptItem}, transaction::{TxInput, TxOutput}, types::TxId, validator::ValidationError,
    };

    use super::*;

    #[test]
    fn valid_transaction_creates_state_transition() {
        let (tx, ledger) = get_valid_tx();

        let res = TransactionProcessor::process(&tx, &ledger, 0);

        assert!(res.is_ok())
    }

    #[test]
    fn collects_spent_utxos() {
        let (tx, ledger) = get_valid_tx();

        let res = TransactionProcessor::process(&tx, &ledger, 0).unwrap();

        assert!(res.spent_utxos.len() == tx.inputs.len());
    }

    #[test]
    fn creates_output_utxos() {
        let (tx, ledger) = get_valid_tx();

        let res = TransactionProcessor::process(&tx, &ledger, 0).unwrap();

        assert!(res.created_utxos.len() == tx.outputs.len());

        for (output, created_utxo) in tx.outputs.iter().zip(res.created_utxos.iter()) {
            assert!(output.value == created_utxo.utxo.value)
        }
    }

    #[test]
    fn assigns_correct_outpoints() {
        let (tx, ledger) = get_valid_tx();

        let res = TransactionProcessor::process(&tx, &ledger, 0).unwrap();

        let txid = tx.txid();

        for (idx , created_utxo)in res.created_utxos.iter().enumerate() {
            assert!(created_utxo.outpoint.txid == txid);
            assert!(created_utxo.outpoint.vout == idx as u32)
        }
    }

    #[test]
    fn preserves_transaction_fee() {
        let (tx, ledger) = get_valid_tx();

        let res = TransactionProcessor::process(&tx, &ledger, 0).unwrap();

        let mut total_input: u64 = 0;
        let mut total_output: u64 = 0;
        for input in tx.inputs.iter() {
            let input_utxo = ledger.get_utxo(&input.previous_output).unwrap();

            total_input += input_utxo.value;
        }

        for output in tx.outputs.iter() {
            total_output += output.value;
        }

        let fee = total_input - total_output;

        assert_eq!(fee, res.fee)
    }

    #[test]
    fn duplicate_input() {
        let (tx, ledger) = get_invalid_tx();

        let res = TransactionProcessor::process(&tx, &ledger, 0);

        assert_eq!(res, Err(ProcessorError::Validation(ValidationError::DuplicateInput)))
    }

    fn get_valid_tx() -> (Transaction, Ledger) {
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
        let message = transaction.signing_hash();

        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            let _ = ledger.add_utxo(input.previous_output.clone(), utxo);
        }

        (transaction, ledger)
    }

    fn get_invalid_tx() -> (Transaction, Ledger) {
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
        let message = transaction.signing_hash();

        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            let _ = ledger.add_utxo(input.previous_output.clone(), utxo);
        }

        (transaction, ledger)
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
