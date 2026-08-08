use crate::{
    block::Block, blockchain::{BlockProcessorErrors, overlay::Overlay}, ledger::Ledger, presistaence::DbPersistence, state_transition::{StateTransition, TransactionProcessor},
};

pub struct BlockProcessor;

impl BlockProcessor {
    pub fn process<S: DbPersistence>(
        block: &Block,
        ledger: &Ledger<S>,
        overlay: &Overlay,
        parent_block_height: u32,
    ) -> Result<Vec<StateTransition>, BlockProcessorErrors> {
        let (_coinbase_tx, tx) = block
            .transactions
            .split_first()
            .ok_or(BlockProcessorErrors::HasNoTransaction)?;

        let mut states: Vec<StateTransition> = Vec::new();
        let mut total_fees = 0;

        // collect states of all transactions and also assure that all transeaction are valid
        for tx in tx {
            let tx_state = TransactionProcessor::process(tx, ledger, overlay, parent_block_height)
                .map_err(|e| BlockProcessorErrors::TransactionProcessor(e))?;

            total_fees += tx_state.fee;
            states.push(tx_state);
        }

        // here we have to validate our coinbase transeaction is it use valid reward + total fee utxo as output.
        // for that we need coinbase implementation first .
        // create coinbase transaction ,  process (validate and return state)
        let coinbase_state = TransactionProcessor::process_coinbase_tx(
            &block.transactions,
            total_fees,
            parent_block_height,
        )
        .map_err(|e| BlockProcessorErrors::TransactionProcessor(e))?;
        states.push(coinbase_state);

        Ok(states)
    }
}
