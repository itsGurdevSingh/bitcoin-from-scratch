use crate::{
    crypto::{generate_keypair_dummy, hash::hash160, sign_tx}, ledger::Ledger, script::{OpCode, Script, ScriptItem}, transaction::{OutPoint, Transaction, TransactionSigHash, TxInput, TxOutput, Witness}, types::TxId, utxo::Utxo, virtual_machine::SigHashType,
};

pub fn get_valid_tx(
    ledger: &mut Ledger,
    input_val: u64,
    input_vout: u32,
    output_val: u64,
) -> Transaction {
    let tx_input = create_dummy_tx_input(input_vout);
    let tx_output = create_dummy_tx_output(output_val);

    // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .

    let mut transaction = Transaction {
        version: 10,
        inputs: vec![tx_input],
        outputs: vec![tx_output],
        lock_time: 1000,
    };

    for i in 0..transaction.inputs.len() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let utxo = create_dummy_utxo(input_val, hash160(&pk.serialize().to_vec()).to_vec());

            let message = transaction.signing_hash(i, &utxo.script_pub_key, SigHashType::All);

            let mut sig: Vec<u8> = sign_tx(&message, &sk).serialize_der().to_vec();
            sig.extend((SigHashType::All as u32).to_le_bytes());

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            transaction.inputs[i].script_sig = script;

            // add valid utxo
            ledger
                .add_utxo(transaction.inputs[i].previous_output.clone(), utxo)
                .unwrap();
        }

    transaction
}

fn create_dummy_tx_input(vout: u32) -> TxInput {
    let sig_script_items: Vec<ScriptItem> = vec![
        ScriptItem::PushData(vec![0u8; 32]),
        ScriptItem::PushData(vec![0u8; 64]),
    ];

    let script_sig = Script {
        items: sig_script_items,
    };

    let previous_output = OutPoint {
        txid: TxId([0u8; 32]),
        vout,
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
