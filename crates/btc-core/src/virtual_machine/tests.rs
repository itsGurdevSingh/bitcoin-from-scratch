#[cfg(test)]

mod test {
    use crate::{
        crypto::{generate_keypair_dummy, hash::hash160, sign_tx},
        ledger::Ledger,
        script::{OpCode, Script, ScriptItem},
        tests::dummy_tx::get_valid_tx,
        transaction::{OutPoint, Transaction, TransactionSigHash, TxInput, TxOutput, Witness},
        types::TxId,
        utxo::Utxo,
        virtual_machine::{ScriptVerifier, SigHashType, VmError},
    };

    #[test]
    fn valid_script() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let transaction = get_valid_tx(&mut ledger, 50, 0, 40);

        for (idx, input) in transaction.inputs.iter().enumerate() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();
            let res = ScriptVerifier::verify(&transaction, idx, utxo);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn bad_hash() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(5);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        for i in 0..transaction.inputs.len() {
            let (_sk, pub_key) = generate_keypair_dummy();
            // add valid utxo but public key hash is wrong
            let utxo = create_dummy_utxo(10, hash160(&pub_key.serialize().to_vec()).to_vec());

            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig_hash = transaction.signing_hash(i, &utxo.script_pub_key, SigHashType::All);

            let mut sig = sign_tx(&sig_hash, &sk).serialize_der().to_vec();
            sig.extend((SigHashType::All as u32).to_le_bytes());

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            transaction.inputs[i].script_sig = script;

            ledger
                .add_utxo(transaction.inputs[i].previous_output.clone(), utxo)
                .unwrap();
        }

        for (idx, input) in transaction.inputs.iter().enumerate() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();
            let res = ScriptVerifier::verify(&transaction, idx, utxo);
            assert_eq!(res, Err(VmError::VerifyFailed));
        }
    }

    #[test]
    fn bad_signature() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = get_valid_tx(&mut ledger, 50, 0, 40);

        let (sk, _pk) = generate_keypair_dummy();

        let utxo = ledger
            .get_utxo(&transaction.inputs[0].previous_output)
            .unwrap();
        let sig_hash = transaction.signing_hash(0, &utxo.script_pub_key, SigHashType::All);
        let mut signature = sign_tx(&sig_hash, &sk).serialize_der().to_vec();
        signature.extend((SigHashType::All as u32).to_le_bytes());

        // change signature
        transaction.inputs[0].script_sig.items[0] = ScriptItem::PushData(signature); // and 4 bytes of type for signing hash.

        for (idx, input) in transaction.inputs.iter().enumerate() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();

            let res = ScriptVerifier::verify(&transaction, idx, utxo);
            assert_eq!(res, Err(VmError::VerifyFailed));
        }
    }

    /// create input with empty sig script
    fn create_dummy_tx_input() -> TxInput {
        let sig_script_items: Vec<ScriptItem> = vec![];

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
            witness: Witness::new(),
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
