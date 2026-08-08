use std::cmp::Ordering;

use crate::types::TxId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeIndex {
    pub fee: u64,
    pub vsize: usize,
    pub txid: TxId,
}

impl PartialOrd for FeeIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FeeIndex {
    fn cmp(&self, other: &Self) -> Ordering {

        let a_fee = self.fee as u128;
        let a_vsize = self.vsize as u128;
        let b_fee = other.fee as u128;
        let b_vsize = other.vsize as u128;

        let cmp = (b_fee * a_vsize).cmp(&(a_fee * b_vsize));

        match cmp {
            Ordering::Equal => self.txid.cmp(&other.txid),
            other => other,
        }
    }
}