use crate::{
    crypto::sha256, serialization::BitcoinSerialize, transaction::Transaction, utxo::Utxo,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrecomputedData {
    pub witness_precompute: WitnessPrecomputed,
    pub taproot_precompute: TaprootPrecomputed,
}

impl PrecomputedData {
    pub fn new(tx: &Transaction, spent_utxo: &[&Utxo]) -> Self {
        let hash_prevouts = tx.hash_prevouts();
        let hash_sequences = tx.hash_sequences();
        let hash_outputs = tx.hash_outputs();

        Self {
            witness_precompute: WitnessPrecomputed {
                hash_prevouts: sha256(&hash_prevouts),
                hash_sequences: sha256(&hash_sequences),
                hash_outputs: sha256(&hash_outputs),
            },

            taproot_precompute: TaprootPrecomputed {
                hash_prevouts,
                hash_amounts: tx.hash_amounts(spent_utxo),
                hash_scriptpubkeys: tx.hash_scriptpubkeys(spent_utxo),
                hash_sequences,
                hash_outputs,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WitnessPrecomputed {
    pub hash_prevouts: [u8; 32],
    pub hash_sequences: [u8; 32],
    pub hash_outputs: [u8; 32],
}

impl WitnessPrecomputed {
    pub fn new(tx: &Transaction) -> Self {
        Self {
            hash_prevouts: sha256(&tx.hash_prevouts()),
            hash_sequences: sha256(&tx.hash_sequences()),
            hash_outputs: sha256(&tx.hash_outputs()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct TaprootPrecomputed {
    pub hash_prevouts: [u8; 32],
    pub hash_amounts: [u8; 32],
    pub hash_scriptpubkeys: [u8; 32],
    pub hash_sequences: [u8; 32],
    pub hash_outputs: [u8; 32],
}

impl TaprootPrecomputed {
    pub fn new(tx: &Transaction, spent_utxo: &[&Utxo]) -> Self {
        Self {
            hash_prevouts: tx.hash_prevouts(),
            hash_amounts: tx.hash_amounts(spent_utxo),
            hash_scriptpubkeys: tx.hash_scriptpubkeys(spent_utxo),
            hash_sequences: tx.hash_sequences(),
            hash_outputs: tx.hash_outputs(),
        }
    }
}

impl Transaction {
    pub fn hash_prevouts(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        for input in self.inputs.iter() {
            bytes.extend(input.previous_output.serialize());
        }
        sha256(&bytes)
    }
    pub fn hash_sequences(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        for input in self.inputs.iter() {
            bytes.extend(input.sequence.to_le_bytes());
        }
        sha256(&bytes)
    }
    pub fn hash_outputs(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        for output in self.outputs.iter() {
            bytes.extend(output.serialize());
        }
        sha256(&bytes)
    }

    pub fn hash_amounts(&self, spent_utxo: &[&Utxo]) -> [u8; 32] {
        let mut bytes: Vec<u8> = Vec::new();

        for utxo in spent_utxo {
            bytes.extend(utxo.value.to_le_bytes());
        }
        sha256(&bytes)
    }
    pub fn hash_scriptpubkeys(&self, spent_utxo: &[&Utxo]) -> [u8; 32] {
        let mut bytes: Vec<u8> = Vec::new();

        for utxo in spent_utxo {
            bytes.extend(utxo.script_pub_key.serialize());
        }
        sha256(&bytes)
    }
}
