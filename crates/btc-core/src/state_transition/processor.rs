use crate::{
    blockchain::overlay::Overlay,
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
        overlay: &Overlay,
        parent_height: u32,
    ) -> Result<StateTransition, ProcessorError> {
        let fee = TransactionValidator::validate(tx, ledger, overlay, parent_height)
            .map_err(|e| ProcessorError::Validation(e))?;

        let mut state: StateTransition = StateTransition {
            spent_utxos: vec![],
            created_utxos: vec![],
            fee,
        };
        for input in tx.inputs.iter() {
            let spent_outpoint = &input.previous_output;

            let spent_utxo = overlay
                .lookup(ledger, &spent_outpoint)
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
                    block_height: parent_height + 1,
                },
            };

            state.created_utxos.push(created_utxo);
        }

        Ok(state)
    }

    pub fn process_coinbase_tx(
        transactions: &[Transaction],
        total_fees: u64,
        parent_height: u32,
    ) -> Result<StateTransition, ProcessorError> {
        TransactionValidator::validate_coinbase(transactions, total_fees, parent_height)
            .map_err(|e| ProcessorError::Validation(e))?;
        
        let tx = &transactions[0];
        let txid = tx.txid();
        let state = StateTransition {
            spent_utxos: vec![],
            created_utxos: vec![CreatedUtxo {
                outpoint: OutPoint { txid, vout: 0 },
                utxo: Utxo {
                    value: tx.outputs[0].value,
                    script_pub_key: tx.outputs[0].script_pub_key.clone(),
                    is_coinbase: true,
                    block_height: parent_height + 1,
                },
            }],
            fee: 0,
        };
        Ok(state)
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use crate::tests::dummy_tx::get_valid_tx;

    use super::*;

    #[test]
    fn valid_transaction_creates_state_transition() {
        let mut ledger = Ledger::new();
        let tx = get_valid_tx(&mut ledger, 50, 0, 48);

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionProcessor::process(&tx, &ledger, &overlay, 0);

        assert!(res.is_ok())
    }

    #[test]
    fn collects_spent_utxos() {
        let mut ledger = Ledger::new();
        let tx = get_valid_tx(&mut ledger, 50, 0, 48);

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionProcessor::process(&tx, &ledger, &overlay, 0).unwrap();

        assert!(res.spent_utxos.len() == tx.inputs.len());
    }

    #[test]
    fn creates_output_utxos() {
        let mut ledger = Ledger::new();
        let tx = get_valid_tx(&mut ledger, 50, 0, 48);

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionProcessor::process(&tx, &ledger, &overlay, 0).unwrap();

        assert!(res.created_utxos.len() == tx.outputs.len());

        for (output, created_utxo) in tx.outputs.iter().zip(res.created_utxos.iter()) {
            assert!(output.value == created_utxo.utxo.value)
        }
    }

    #[test]
    fn assigns_correct_outpoints() {
        let mut ledger = Ledger::new();
        let tx = get_valid_tx(&mut ledger, 50, 0, 48);

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionProcessor::process(&tx, &ledger, &overlay, 0).unwrap();

        let txid = tx.txid();

        for (idx, created_utxo) in res.created_utxos.iter().enumerate() {
            assert!(created_utxo.outpoint.txid == txid);
            assert!(created_utxo.outpoint.vout == idx as u32)
        }
    }

    #[test]
    fn preserves_transaction_fee() {
        let mut ledger = Ledger::new();
        let tx = get_valid_tx(&mut ledger, 50, 0, 48);

        let overlay = Overlay {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        let res = TransactionProcessor::process(&tx, &ledger, &overlay, 0).unwrap();

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
}
