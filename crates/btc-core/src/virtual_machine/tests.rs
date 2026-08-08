#[cfg(test)]

mod test {
    use std::collections::HashMap;

    use secp256k1::{Keypair, Secp256k1, XOnlyPublicKey};

    use crate::{
        crypto::{generate_keypair_dummy, hash::hash160, schnorr::sign_tx_tr, sign_tx}, ledger::Ledger, presistaence::Store, script::{OpCode, Script, ScriptItem}, serialization::BitcoinSerialize, taproot::{
            ControlBlock, control_block::LeafVesrion, sighash::taproot_sighash, tweak_public_key,
        }, tests::dummy_tx::get_valid_tx, transaction::{
            OutPoint, SpendType, TaprootPrecomputed, Transaction, TransactionSigHash, TxInput,
            TxOutput, Witness, sighash::TransactionTaprootSigHash,
        }, types::TxId, utxo::Utxo, virtual_machine::{ScriptVerifier, SigHashType, VmError},
    };

    #[test]
    fn valid_script() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

        let transaction = get_valid_tx(&mut ledger, 50, 0, 40);

        let mut spending_utxo: HashMap<&OutPoint, Utxo> = HashMap::new();

        for input in transaction.inputs.iter() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();
            spending_utxo.insert(&input.previous_output, utxo);
        }

        let res = ScriptVerifier::verify_transaction_scripts(&transaction, &spending_utxo);
        assert!(res.is_ok());
    }

    #[test]
    fn bad_hash() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(5);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

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

        let mut spending_utxo: HashMap<&OutPoint, Utxo> = HashMap::new();

        for input in transaction.inputs.iter() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();
            spending_utxo.insert(&input.previous_output, utxo);
        }

        let res = ScriptVerifier::verify_transaction_scripts(&transaction, &spending_utxo);

        assert_eq!(res, Err(VmError::VerifyFailed));
    }

    #[test]
    fn bad_signature() {
        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new(Store::new());

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

        let mut spending_utxo: HashMap<&OutPoint, Utxo> = HashMap::new();

        for input in transaction.inputs.iter() {
            let utxo = ledger.get_utxo(&input.previous_output).unwrap();
            spending_utxo.insert(&input.previous_output, utxo);
        }

        let res = ScriptVerifier::verify_transaction_scripts(&transaction, &spending_utxo);
        assert_eq!(res, Err(VmError::VerifyFailed));
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

    #[test]
    fn taproot_key_path() {
        //==================================================================================//
        let (sk, _pk) = generate_keypair_dummy();
        let secp = Secp256k1::new();

        let keypair = Keypair::from_secret_key(&secp, &sk);
        let xonly_pk = tweak_public_key(&keypair.x_only_public_key().0, None)
            .unwrap()
            .0
            .serialize();
        //=================================================================================//

        let mut ledger  = Ledger::new(Store::new());
        let mut tx = get_valid_tx(&mut ledger, 50, 0, 40);

        let mut spent_utxo_set: HashMap<&OutPoint, Utxo> = HashMap::new();

        let utxo_xonly_pk = Utxo {
            value: 50,
            script_pub_key: Script {
                items: vec![
                    ScriptItem::Op(OpCode::Op1),
                    ScriptItem::PushData(xonly_pk.to_vec()),
                ],
            },
            is_coinbase: false,
            block_height: 0,
        };

        let mut outpoints: Vec<OutPoint> = Vec::new();
        for i in 0..tx.inputs.len() {
            outpoints.push(tx.inputs[i].previous_output.clone());
        }

        for i in 0..tx.inputs.len() {
            ledger.spend_utxo(&tx.inputs[i].previous_output).unwrap();
            ledger
                .add_utxo(tx.inputs[i].previous_output.clone(), utxo_xonly_pk.clone())
                .unwrap();

            tx.inputs[i].script_sig = Script::new();

            spent_utxo_set.insert(&outpoints[i], utxo_xonly_pk.clone());
        }

        let mut spent_utxo = Vec::new();
        let mut spent_utxo_ref = Vec::new();

        for (_outpoint, utxo) in spent_utxo_set.clone() {
            spent_utxo.push(utxo.clone());
        }
        for utxo in spent_utxo.iter() {
            spent_utxo_ref.push(utxo);
        }

        let precompute = TaprootPrecomputed::new(&tx, &spent_utxo_ref);

        for idx in 0..tx.inputs.len() {
            let utxo_set = spent_utxo_set.clone();
            let utxo = utxo_set.get(&tx.inputs[idx].previous_output).unwrap();
            let sig_msg = tx
                .signing_hash_taproot(idx, &precompute, utxo, SigHashType::All, SpendType::KeyPath)
                .unwrap();
            let message = taproot_sighash(&sig_msg);

            let mut signature = sign_tx_tr(&message, &sk, None)
                .unwrap()
                .to_byte_array()
                .to_vec();
            signature.push(SigHashType::All as u8);

            for input in tx.inputs.iter_mut() {
                input.witness.stack.push(signature.to_vec())
            }

            // assert!(verify_signature_tr(&xonly_pk, &message, &signature));
            assert_eq!(
                Ok(()),
                ScriptVerifier::verify_transaction_scripts(&tx, &spent_utxo_set)
            );
        }
    }

    #[test]
    fn taproot_script_path() {
        let mut ledger= Ledger::new(Store::new());

        let mut tx = get_valid_tx(&mut ledger, 50, 0, 40);

        let keys_a = generate_keypair_dummy();
        let keys_b = generate_keypair_dummy();
        let keys_c = generate_keypair_dummy();

        let secp = Secp256k1::new();
        let keypair_b = Keypair::from_secret_key(&secp, &keys_b.0);
        let (xonly_pub_key_b, _parity) = XOnlyPublicKey::from_keypair(&keypair_b);
        let xonly_pub_key_b_tweaked = tweak_public_key(&xonly_pub_key_b, None)
            .unwrap()
            .0
            .serialize()
            .to_vec();

        let script_a = create_p2pkh_script(keys_a.1.serialize().to_vec());
        let script_b = create_p2pkh_script(xonly_pub_key_b_tweaked.clone());
        let script_c = create_p2pkh_script(keys_c.1.serialize().to_vec());
        let scripts = [&script_a, &script_b, &script_c];
        let leaf_version = LeafVesrion::V1;

        //======================GENERATE KEY PAIR ======================//
        let (sk, _pk) = generate_keypair_dummy();
        let keypair = Keypair::from_secret_key(&secp, &sk);
        let (xonly_pub_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        //=====================NEW CONTROL BLOCK =============================//
        let (control_block, xonly_public_key_tweaked) = ControlBlock::new(
            &script_b,
            &scripts,
            leaf_version,
            &xonly_pub_key.serialize(),
        )
        .unwrap();

        tx.inputs[0].script_sig = Script::new();

        let utxo = Utxo {
            value: 50,
            script_pub_key: Script {
                items: vec![
                    ScriptItem::Op(OpCode::Op1),
                    ScriptItem::PushData(xonly_public_key_tweaked.serialize().to_vec()),
                ],
            },
            is_coinbase: false,
            block_height: 0,
        };

        let _ = ledger.spend_utxo(&tx.inputs[0].previous_output).unwrap();
        ledger
            .add_utxo(tx.inputs[0].previous_output.clone(), utxo.clone())
            .unwrap();

        // updata witness [signature, public_key, tap_script, control_block]
        let mut witness = Witness::new();
        let precompute: TaprootPrecomputed = TaprootPrecomputed::new(&tx, &[&utxo]);
        let signing_hash = tx
            .signing_hash_taproot(
                0,
                &precompute,
                &utxo,
                SigHashType::All,
                SpendType::ScriptPath,
            )
            .unwrap();
        let message = taproot_sighash(&signing_hash);
        let mut signature = sign_tx_tr(&message, &keys_b.0, None)
            .unwrap()
            .to_byte_array()
            .to_vec();
        signature.push(SigHashType::All as u8);

        witness.stack.push(signature);
        witness.stack.push(xonly_pub_key_b_tweaked);
        witness.stack.push(script_b.serialize());
        witness.stack.push(control_block.serialize());

        tx.inputs[0].witness = witness;

        let mut utxo_set: HashMap<&OutPoint, Utxo> = HashMap::new();
        utxo_set.insert(&tx.inputs[0].previous_output, utxo);

        let res = ScriptVerifier::verify_transaction_scripts(&tx, &utxo_set);

        assert_eq!(Ok(()), res)
    }

    fn create_p2pkh_script(pub_key_bytes: Vec<u8>) -> Script {
        let pub_key_hash = hash160(&pub_key_bytes);
        Script {
            items: vec![
                ScriptItem::Op(OpCode::Dup),
                ScriptItem::Op(OpCode::Hash160),
                ScriptItem::PushData(pub_key_hash.to_vec()),
                ScriptItem::Op(OpCode::EqualVerify),
                ScriptItem::Op(OpCode::CheckSig),
            ],
        }
    }
}
