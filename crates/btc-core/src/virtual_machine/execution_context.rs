use crate::{script::Script, transaction::Transaction, virtual_machine::SigVersion};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExecutionContext {
    pub transaction: Transaction,
    pub input_index: usize,
    pub prevout_value: u64,
    pub script_code: Script,
    pub sig_version: SigVersion,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            transaction: Transaction::new(),
            input_index: 0,
            prevout_value: 1,
            script_code: Script::new(),
            sig_version: SigVersion::Legacy,
        }
    }
}
