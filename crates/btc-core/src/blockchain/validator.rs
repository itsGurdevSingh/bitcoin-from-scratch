use crate::{
    block::{Block, BlockErrors, constants::MAX_BLOCK_SIG_OP_COST}, blockchain::{Blockchain, error::BlockchainError, overlay::Overlay}, ledger::LedgerError, presistaence::DbPersistence, script::Script, serialization::BitcoinDeserialize, transaction::{SpendType, TxInput}, utils::time::Time, utxo::UtxoError, virtual_machine::ScriptType,
};

pub struct ChainValidator;

impl ChainValidator {
    pub fn validate<S: DbPersistence>(
        chain: &Blockchain<S>,
        block: &Block,
        overlay: &Overlay,
    ) -> Result<(), BlockchainError> {
        // validate header
        // is valid previos block hash
        if !(block.header.previous_block_hash
            == chain
                .get_node_by_hash(block.header.previous_block_hash)
                .ok_or(BlockchainError::InvalidHeader)?
                .hash
            && block.header.bits == chain.expected_bits()?
            && block.header.timestamp < Time::unix_timestamp() + 7200
            && chain.median_timestamp()? < block.header.timestamp)
        {
            return Err(BlockchainError::InvalidHeader);
        }

        block
            .validate_block()
            .map_err(|e| BlockchainError::Block(e))?;

        let block_sig_op_cost = Self::sig_op_cost(chain, block, overlay)?;

        if MAX_BLOCK_SIG_OP_COST < block_sig_op_cost {
            return Err(BlockchainError::Block(BlockErrors::SigOpCostExceeded));
        }

        Ok(())
    }

    fn sig_op_cost<S: DbPersistence>(
        chain: &Blockchain<S>,
        block: &Block,
        overlay: &Overlay,
    ) -> Result<u32, BlockchainError> {
        let mut cost: u32 = 0;

        for tx in block.transactions[1..].iter() {
            for input in tx.inputs.iter() {
                let utxo = overlay
                    .lookup(&chain.ledger, &input.previous_output)
                    .ok_or(BlockchainError::Ledger(LedgerError::Utxo(
                        UtxoError::NotFound,
                    )))?;

                let script_pub = &utxo.script_pub_key;
                let script_type = ScriptType::is_type_of(script_pub, &input.script_sig);

                match script_type {
                    ScriptType::P2PKH => cost += 1,
                    ScriptType::P2SH => {
                        let redeem_script_bytes = Self::redeem_script_from_script_sig(input)?;
                        cost += Self::script_sig_op_cost(redeem_script_bytes)?;
                    }
                    ScriptType::P2WPKH => cost += 1,
                    ScriptType::P2WSH => {
                        let redeem_script_bytes = Self::redeem_script_from_witness(input)?;
                        cost += Self::script_sig_op_cost(redeem_script_bytes)?;
                    }
                    ScriptType::P2TR => match SpendType::get_spent_type(&input.witness) {
                        Some(SpendType::KeyPath) => cost += 1,
                        Some(SpendType::ScriptPath) => {
                            let redeem_script_bytes =
                                Self::redeem_script_from_taproot_witness(input)?;
                            cost += Self::script_sig_op_cost(redeem_script_bytes)?;
                        }
                        Some(_) | None => return Err(BlockchainError::InvalidScriptFormat),
                    },
                    ScriptType::None => {}
                };
            }
        }

        Ok(cost)
    }

    fn redeem_script_from_script_sig(input: &TxInput) -> Result<&[u8], BlockchainError> {
        input
            .script_sig
            .items
            .last()
            .and_then(|item| item.get_bytes())
            .ok_or(BlockchainError::InvalidScriptFormat)
    }

    fn redeem_script_from_witness(input: &TxInput) -> Result<&[u8], BlockchainError> {
        input
            .witness
            .stack
            .last()
            .map(|bytes| bytes.as_slice())
            .ok_or(BlockchainError::InvalidScriptFormat)
    }

    fn redeem_script_from_taproot_witness(input: &TxInput) -> Result<&[u8], BlockchainError> {
        input
            .witness
            .stack
            .get(input.witness.stack.len().saturating_sub(2))
            .map(|bytes| bytes.as_slice())
            .ok_or(BlockchainError::InvalidScriptFormat)
    }

    fn script_sig_op_cost(redeem_script_bytes: &[u8]) -> Result<u32, BlockchainError> {
        let (script, _) = Script::deserialize(redeem_script_bytes)
            .map_err(|_| BlockchainError::InvalidScriptFormat)?;

        Ok(script.sig_op_cost())
    }
}
