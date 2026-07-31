use crate::{script::Script, transaction::{PrecomputedData, Transaction}, utxo::Utxo, virtual_machine::SigVersion};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExecutionContext {
    pub transaction: Transaction,
    pub input_index: usize,
    pub prevout_value: u64,
    pub script_code: Script,
    pub sig_version: SigVersion,
    pub precompute: PrecomputedData,
    pub current_spending_utxo: Utxo
}

impl ExecutionContext {
    pub fn new() -> Self {
        let tx = Transaction::new();
        Self {
            precompute: PrecomputedData::new(&tx, &[]),
            transaction: tx,
            input_index: 0,
            prevout_value: 1,
            script_code: Script::new(),
            sig_version: SigVersion::Legacy,
            current_spending_utxo: Utxo::new()
        }
    }
}
