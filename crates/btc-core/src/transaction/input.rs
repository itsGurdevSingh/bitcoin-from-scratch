use crate::script::Script;
use crate::transaction::OutPoint;

pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
}