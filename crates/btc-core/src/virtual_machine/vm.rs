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
