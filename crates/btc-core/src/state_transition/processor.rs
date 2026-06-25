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
