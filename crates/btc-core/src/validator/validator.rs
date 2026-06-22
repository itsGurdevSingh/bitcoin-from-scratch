use std::collections::HashSet;
type Fee = u64;

use crate::{
    ledger::Ledger,
    transaction::{OutPoint, Transaction},
    validator::ValidationError,
};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate(tx: &Transaction, ledger: &Ledger) -> Result<Fee, ValidationError> {
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
        for input in tx.inputs.iter() {
            // is duplicate
            if !seen_inputs.insert(input.previous_output.clone()) {
                return Err(ValidationError::DuplicateInput);
            };

            // get utxo for input
            let res = ledger.get_utxo(&input.previous_output);

            match res {
                Some(utxo) => {
                    total_input_value += utxo.value;
                }
                None => return Err(ValidationError::MissingUtxo),
            }
        }

        // output and total value of outputs

        let mut total_output_value: u64 = 0;
        for output in tx.outputs.iter() {
            if output.value == 0 {
                return Err(ValidationError::InvalidOutputValue);
            }
            total_output_value += output.value;

            // TODO:
            // Validate script structure once Script VM is implemented.
        }

        // is input values enough for  output
        if total_input_value < total_output_value {
            return Err(ValidationError::InsufficientInputValue);
        }

        let fee: Fee = total_input_value - total_output_value;

        Ok(fee)
    }
}

//Todo
// fn valdiate_script(script: &Script) -> bool {
//     true
// }

#[cfg(test)]
mod test {
    use crate::{
        script::{OpCode, Script, ScriptItem},
        transaction::{TxInput, TxOutput},
        types::TxId,
        utxo::Utxo,
    };

    use super::*;

    #[test]
    fn valid_transaction() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // add utxo to ledger to replicate they are valid and already their
        let mut ledger = Ledger::new();

        let utxo = create_dummy_utxo(10);

        ledger
            .add_utxo(tx_input.clone().previous_output, utxo)
            .unwrap();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res = TransactionValidator::validate(&transaction, &ledger);

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

        let res = TransactionValidator::validate(&transaction, &ledger);

        assert_eq!(res, Err(ValidationError::MissingUtxo));
    }
    #[test]
    fn duplicate_input() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(2);

        // add utxo to ledger to replicate they are valid and already their
        let mut ledger = Ledger::new();

        let utxo = create_dummy_utxo(10);

        ledger
            .add_utxo(tx_input.clone().previous_output, utxo)
            .unwrap();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input.clone(), tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res = TransactionValidator::validate(&transaction, &ledger);

        assert_eq!(res, Err(ValidationError::DuplicateInput));
    }
    #[test]
    fn insufficient_input_value() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(20);

        // add utxo to ledger to replicate they are valid and already their
        let mut ledger = Ledger::new();

        let utxo = create_dummy_utxo(10);

        ledger
            .add_utxo(tx_input.clone().previous_output, utxo)
            .unwrap();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res = TransactionValidator::validate(&transaction, &ledger);

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

        let res = TransactionValidator::validate(&transaction, &ledger);

        assert_eq!(res, Err(ValidationError::NoInputs));
    }
    #[test]
    fn no_outputs() {
        let tx_input = create_dummy_tx_input();

        // add utxo to ledger to replicate they are valid and already their
        let mut ledger = Ledger::new();

        let utxo = create_dummy_utxo(10);

        ledger
            .add_utxo(tx_input.clone().previous_output, utxo)
            .unwrap();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![],
            lock_time: 1000,
        };

        let res = TransactionValidator::validate(&transaction, &ledger);

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

    fn create_dummy_utxo(val: u64) -> Utxo {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(vec![0u8; 20]), // 20-byte dummy pubkey hash
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
