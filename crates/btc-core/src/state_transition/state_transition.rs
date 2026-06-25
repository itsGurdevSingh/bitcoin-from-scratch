use crate::{transaction::OutPoint, utxo::Utxo};

pub struct SpentUtxo {
    pub outpoint: OutPoint,
    pub utxo: Utxo,
}

pub struct CreatedUtxo {
    pub outpoint: OutPoint,
    pub utxo: Utxo,
}

pub struct StateTransition {
    pub spent_utxos: Vec<SpentUtxo>,
    pub created_utxos: Vec<CreatedUtxo>,
    pub fee: u64,
}