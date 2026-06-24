use std::collections::{HashMap, HashSet};

use crate::{
    ledger::Ledger,
    mempool::{MEMPOOL_SIZE, MempoolEntry, MempoolError},
    transaction::{OutPoint, Transaction},
    types::TxId,
    validator::TransactionValidator,
};

pub struct Mempool {
    transactions: HashMap<TxId, MempoolEntry>,
    reserved_outpoints: HashSet<OutPoint>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            reserved_outpoints: HashSet::new(),
        }
    }

    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        ledger: &Ledger,
    ) -> Result<u64, MempoolError> {
        // early exit
        if self.transactions.len() >= MEMPOOL_SIZE {
            return Err(MempoolError::MempoolFull);
        }

        let txid = tx.txid();
        if self.contains(&txid) {
            return Err(MempoolError::TransactionAlreadyExists);
        }

        let fee = TransactionValidator::validate(&tx, ledger)
            .map_err(|_| MempoolError::ValidationFailed)?;

         // test run for error exit not impact data (save for rollback)
        for input in &tx.inputs {
            if self.reserved_outpoints.contains(&input.previous_output) {
                return Err(MempoolError::DoubleSpendDetected);
            }
        }
        for input in &tx.inputs {
            self.reserved_outpoints
                .insert(input.previous_output.clone());
        }

        let entry = MempoolEntry { tx, fee };

        self.transactions.insert(txid, entry);

        return Ok(fee);
    }

    pub fn remove_transaction(&mut self, txid: &TxId) -> Option<MempoolEntry> {
        if let Some(entry) = self.transactions.remove(txid) {
            for input in &entry.tx.inputs {
                self.reserved_outpoints.remove(&input.previous_output);
            }

            return Some(entry);
        }
        return None;
    }

    pub fn contains(&self, txid: &TxId) -> bool {
        self.transactions.contains_key(txid)
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }
}



#[cfg(test)]

mod test {

    use crate::{crypto::{generate_keypair_dummy, hash::hash160, sign_tx}, script::{OpCode, Script, ScriptItem}, transaction::{TxInput, TxOutput}, utxo::Utxo};

use super::*;

    #[test]
    fn valid_transaction_added() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message =transaction.signing_hash();


        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            ledger
                .add_utxo(input.previous_output.clone(), utxo)
                .unwrap();
        }

        let mut mempool = Mempool::new();

        let res = mempool.add_transaction(transaction, &ledger);

        assert_eq!(res, Ok(2));

    }

    #[test]
    fn duplicate_txid_rejected() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message =transaction.signing_hash();


        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            ledger
                .add_utxo(input.previous_output.clone(), utxo)
                .unwrap();
        }

        let mut mempool = Mempool::new();

        let _res = mempool.add_transaction(transaction.clone(), &ledger);
        let res2 = mempool.add_transaction(transaction, &ledger);

        assert_eq!(res2, Err(MempoolError::TransactionAlreadyExists))


    }

    #[test]
    fn double_spend_rejected() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input.clone()],
            outputs: vec![tx_output.clone()],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message =transaction.signing_hash();


        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            ledger
                .add_utxo(input.previous_output.clone(), utxo)
                .unwrap();
        }

        let mut transaction2 = transaction.clone();

        transaction2.version= 12;

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message =transaction2.signing_hash();


        for input in transaction2.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            let _ = ledger.add_utxo(input.previous_output.clone(), utxo);
        }


        let mut mempool = Mempool::new();

        let _res = mempool.add_transaction(transaction, &ledger);
        let res2 = mempool.add_transaction(transaction2, &ledger);

        assert_eq!(res2, Err(MempoolError::DoubleSpendDetected));
        // assert_eq!(res2, Err(MempoolError::ValidationFailed))

    }

    #[test]
    fn remove_transaction_releases_inputs() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message =transaction.signing_hash();


        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            ledger
                .add_utxo(input.previous_output.clone(), utxo)
                .unwrap();
        }

        let mut mempool = Mempool::new();

        mempool.add_transaction(transaction.clone(), &ledger).unwrap();


        let txid = transaction.txid();

       let _res =  mempool.remove_transaction(&txid);

       assert!(!mempool.contains(&txid))



    }

    #[test]
    fn invalid_transaction_does_not_reserve_inputs() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(8);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input.clone(), tx_input], // this make tx invalid because we add same input twice
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message =transaction.signing_hash();


        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            let _ = ledger.add_utxo(input.previous_output.clone(), utxo);
        }

        let mut mempool = Mempool::new();

        let res = mempool.add_transaction(transaction, &ledger);

        assert_eq!(res, Err(MempoolError::ValidationFailed))

    }


    fn create_dummy_tx_input() -> TxInput {
        let sig_script_items: Vec<ScriptItem> = vec![
            ScriptItem::PushData(vec![0u8; 32]),
            ScriptItem::PushData(vec![0u8; 64]),
        ];

        let script_sig = Script {
            items: sig_script_items,
        };

        let previous_output = OutPoint {
            txid: TxId([0u8; 32]),
            vout: 8,
        };

        TxInput {
            previous_output,
            script_sig,
            sequence: 5,
        }
    }

    fn create_dummy_tx_output(val: u64) -> TxOutput {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(vec![0u8; 20]), // 20-byte dummy pubkey hash
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        let script: Script = Script {
            items: p2pkh_script,
        };

        TxOutput {
            value: val,
            script_pub_key: script,
        }
    }

    fn create_dummy_utxo(val: u64, pkh: Vec<u8>) -> Utxo {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(pkh), 
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        Utxo {
            value: val,
            script_pub_key: Script {
                items: p2pkh_script,
            },
            is_coinbase: false,
            block_height: 1000,
        }
    }
}
