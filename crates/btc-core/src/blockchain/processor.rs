use crate::{
    block::Block, blockchain::{BlockProcessorErrors, overlay::Overlay}, ledger::Ledger, state_transition::{StateTransition, TransactionProcessor},
};

pub struct BlockProcessor;

impl BlockProcessor {
    pub fn process(
        block: &Block,
        ledger: &Ledger,
        overlay: &Overlay
    ) -> Result<Vec<StateTransition>, BlockProcessorErrors> {
        let (coinbase_tx, tx) = block
            .transactions
            .split_first()
            .ok_or(BlockProcessorErrors::HasNoTransaction)?;

        let mut states: Vec<StateTransition> = Vec::new();
        let mut total_fees = 0;

        // collect states of all transactions and also assure that all transeaction are valid
        for tx in tx {
            let tx_state = TransactionProcessor::process(tx, ledger, overlay, 10)
                .map_err(|e| BlockProcessorErrors::TransactionProcessor(e))?;

            total_fees += tx_state.fee;
            states.push(tx_state);
        }

        // here we have to validate our coinbase transeaction is it use valid reward + total fee utxo as output.
        // for that we need coinbase implementation first .
        // create coinbase transaction ,  process (validate and return state)
        let coinbase_state = TransactionProcessor::process_coinbase_tx(coinbase_tx, total_fees, 10)
            .map_err(|e| BlockProcessorErrors::TransactionProcessor(e))?;
        states.push(coinbase_state);

        Ok(states)
    }
}
