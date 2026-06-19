use crate::script::Script;

/// Represents an Unspent Transaction Output (UTXO).
///
/// A UTXO is the fundamental unit of ownership in Bitcoin.
/// Unlike account-based systems, Bitcoin does not track balances.
/// Instead, it tracks a set of spendable outputs.
///
/// A UTXO contains:
/// - The amount of value it holds (in satoshis)
/// - The spending conditions required to unlock it
/// - Metadata needed for consensus validation
///
/// UTXOs are identified externally by an OutPoint:
/// `(txid, vout)`.
#[derive(Debug, Clone)]
pub struct Utxo {
    /// Amount stored in this UTXO, denominated in satoshis.
    pub value: u64,

    /// Locking script that defines the conditions required
    /// to spend this UTXO.
    pub script_pub_key: Script,

    /// Indicates whether this UTXO was created by a coinbase
    /// transaction (block reward).
    ///
    /// Coinbase outputs require 100 confirmations before
    /// they become spendable.
    pub is_coinbase: bool,

    /// Block height at which this UTXO was created.
    ///
    /// Used for rules such as coinbase maturity and
    /// height-based validation.
    pub block_height: u32,
}