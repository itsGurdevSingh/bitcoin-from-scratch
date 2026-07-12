use crate::{transaction::OutPoint, utxo::Utxo};

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct SpentUtxo {
    pub outpoint: OutPoint,
    pub utxo: Utxo,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct CreatedUtxo {
    pub outpoint: OutPoint,
    pub utxo: Utxo,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StateTransition {
    pub spent_utxos: Vec<SpentUtxo>,
    pub created_utxos: Vec<CreatedUtxo>,
    pub fee: u64,
}