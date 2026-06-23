use crate::{
    crypto::{hash::hash160, verify_signature},
    script::{OpCode, OpCodeTrait, Script, ScriptItem},
    virtual_machine::{StackItem, VmError},
};

pub struct VirtualMachine<'a> {
    stack: Vec<StackItem>,
    message: &'a [u8],
}

impl<'a> VirtualMachine<'a> {
    pub fn new(message: &'a [u8]) -> Self {
        Self {
            stack: Vec::new(),
            message,
        }
    }

    pub fn execute_script(
        &mut self,
        script_sig: &Script,
        script_pub_key: &Script,
    ) -> Result<(), VmError> {
        // combine both script in execution manner .
        let mut script = script_sig.items.clone();
        script.extend(script_pub_key.items.clone());

        for item in &script {
            match item {
                ScriptItem::PushData(data) => {
                    self.stack.push(StackItem::Bytes(data.clone()));
                }

                ScriptItem::Op(op) => {
                    self.execute_opcode(op)?;
                }
            }
        }
        match self.stack.last() {
            Some(StackItem::Bool(true)) => Ok(()),
            _ => Err(VmError::VerifyFailed),
        }
    }

    fn execute_opcode(&mut self, opcode: &OpCode) -> Result<(), VmError> {
        // we will add more opcode here.
        match opcode {
            OpCode::Dup => self.dup(),
            OpCode::Hash160 => self.hash160(),
            OpCode::EqualVerify => self.equal_verify(),
            OpCode::CheckSig => self.check_sig(),
        }
    }
}

impl<'a> OpCodeTrait for VirtualMachine<'a> {
    fn dup(&mut self) -> Result<(), VmError> {
        let top_elem = self.stack.last().cloned().ok_or(VmError::EmptyStack)?;

        self.stack.push(top_elem);
        Ok(())
    }

    fn hash160(&mut self) -> Result<(), VmError> {
        let top_elem = match self.stack.pop() {
            Some(data) => data,
            None => return Err(VmError::EmptyStack),
        };

        if let StackItem::Bytes(bytes) = top_elem {
            let hash = hash160(&bytes);
            self.stack.push(StackItem::Bytes(hash.to_vec()));
        } else {
            return Err(VmError::InvalidData);
        }

        Ok(())
    }

    fn equal_verify(&mut self) -> Result<(), VmError> {
        let top_elem = match self.stack.pop() {
            Some(data) => match data {
                StackItem::Bytes(value) => value,
                _ => return Err(VmError::VerifyFailed),
            },
            None => return Err(VmError::EmptyStack),
        };
        let top_elem_2 = match self.stack.pop() {
            Some(data) => match data {
                StackItem::Bytes(value) => value,
                _ => return Err(VmError::VerifyFailed),
            },
            None => return Err(VmError::EmptyStack),
        };

        if top_elem != top_elem_2 {
            return Err(VmError::VerifyFailed);
        }

        Ok(())
    }

    fn check_sig(&mut self) -> Result<(), VmError> {
        let pubkey = match self.stack.pop() {
            Some(data) => match data {
                StackItem::Bytes(value) => value,
                _ => return Err(VmError::VerifyFailed),
            },
            None => return Err(VmError::EmptyStack),
        };
        let signature = match self.stack.pop() {
            Some(data) => match data {
                StackItem::Bytes(value) => value,
                _ => return Err(VmError::VerifyFailed),
            },
            None => return Err(VmError::EmptyStack),
        };

        let res = verify_signature(&pubkey, self.message, &signature);

        if !res {
            return Err(VmError::VerifyFailed);
        }

        self.stack.push(StackItem::Bool(res));

        Ok(())
    }
}

#[cfg(test)]

mod test {
    use crate::{
        crypto::{generate_keypair_dummy, sha256d, sign_tx},
        ledger::Ledger,
        script::{OpCode, Script, ScriptItem},
        serialization::BitcoinSerialize,
        transaction::{OutPoint, Transaction, TxInput, TxOutput},
        types::TxId,
        utxo::Utxo,
    };

    use super::*;

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
            let utxo = create_dummy_utxo(10, vec![2,3,21,21,53,3,11]);

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
fn bad_signature(){
    
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

            assert_eq!(res,Err(VmError::VerifyFailed));
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
