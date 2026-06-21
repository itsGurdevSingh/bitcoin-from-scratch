use std::collections::HashSet;
type Fee = u64;

use crate::{
    ledger::Ledger,
    script::Script,
    transaction::{OutPoint, Transaction},
    utxo,
    validator::ValidationError,
};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate(tx: &Transaction, ledger: &Ledger) -> Result<Fee, ValidationError> {
        // check inputs exist output exist
        if tx.inputs.len() == 0 {
            return Err(ValidationError::NoInputs);
        }
        if tx.outputs.len() == 0 {
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

fn valdiate_script(script: &Script) -> bool {
    true
}
