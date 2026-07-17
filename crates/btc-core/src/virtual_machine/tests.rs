#[cfg(test)]

mod test {
    use crate::{
        crypto::{generate_keypair_dummy, hash::hash160, sha256d, sign_tx}, ledger::Ledger, script::{OpCode, Script, ScriptItem}, serialization::BitcoinSerialize, transaction::{OutPoint, Transaction, TxInput, TxOutput, Witness}, types::TxId, utxo::Utxo, virtual_machine::{VirtualMachine, VmError},
    };

    #[test]
    fn valid_script() {
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

        // get message serilize transaction and double hash that.
        let serialize = transaction.serialize();
        let message = sha256d(&serialize);

        let mut vm = VirtualMachine::new(&message);

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

        for input in transaction.inputs.iter() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();

            let res = vm.execute_script(&input.script_sig, &utxo.script_pub_key);

            println!(" result of test is for input {:?}", res);

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

        // get message serilize transaction and double hash that.
        let serialize = transaction.serialize();
        let message = sha256d(&serialize);

        let mut vm = VirtualMachine::new(&message);

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

            // add valid utxo but public key hash is wrong
            let utxo = create_dummy_utxo(10, vec![2, 3, 21, 21, 53, 3, 11]);

            ledger
                .add_utxo(input.previous_output.clone(), utxo)
                .unwrap();
        }

        for input in transaction.inputs.iter() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();

            let res = vm.execute_script(&input.script_sig, &utxo.script_pub_key);

            println!(" result of test is for input {:?}", res);

            assert_eq!(res, Err(VmError::VerifyFailed));
        }
    }

    #[test]
    fn bad_signature() {
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

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        // make wrong message to invalidate signature.
        let message = [0u8; 32];

        let mut vm = VirtualMachine::new(&[1u8; 32]);

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

        for input in transaction.inputs.iter() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();

            let res = vm.execute_script(&input.script_sig, &utxo.script_pub_key);

            println!(" result of test is for input {:?}", res);

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
