use crate::{
    crypto::{hash::hash160, verify_signature},
    script::{OpCode, OpCodeTrait, Script, ScriptItem},
    virtual_machine::{MAX_SCRIPT_ELEMENT_SIZE, MAX_SCRIPT_SIZE, MAX_STACK_SIZE},
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
        if script_sig.items.len() == 0 || script_pub_key.items.len() == 0 {
            return Err(VmError::EmptyScript);
        }

        // combine both script in execution manner .
        let mut script = script_sig.items.clone();
        script.extend(script_pub_key.items.clone());

        if script.len() > MAX_SCRIPT_SIZE {
            return Err(VmError::ScriptTooLarge);
        }

        for item in &script {
            match item {
                ScriptItem::PushData(data) => {
                    if self.stack.len() >= MAX_STACK_SIZE {
                        return Err(VmError::StackOverflow);
                    }

                    if data.len() > MAX_SCRIPT_ELEMENT_SIZE {
                        return Err(VmError::ScriptTooLarge);
                    }
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
            OpCode::CheckSig => self.check_sig(),
            OpCode::Equal => self.equal(),
            OpCode::Verify => self.verify(),
            OpCode::EqualVerify => self.equal_verify(),
        }
    }

    fn pop_bytes(&mut self) -> Result<Vec<u8>, VmError> {
        match self.stack.pop() {
            Some(StackItem::Bytes(bytes)) => Ok(bytes),
            Some(_) => Err(VmError::InvalidData),
            None => Err(VmError::EmptyStack),
        }
    }
}

impl<'a> OpCodeTrait for VirtualMachine<'a> {
    fn dup(&mut self) -> Result<(), VmError> {
        let top_elem = self.stack.last().cloned().ok_or(VmError::EmptyStack)?;

        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }

        self.stack.push(top_elem);
        Ok(())
    }

    fn hash160(&mut self) -> Result<(), VmError> {
        let top_elem = self.stack.pop().ok_or(VmError::EmptyStack)?;

        if let StackItem::Bytes(bytes) = top_elem {
            let hash = hash160(&bytes).to_vec();

            if self.stack.len() >= MAX_STACK_SIZE {
                return Err(VmError::StackOverflow);
            }

            if hash.len() > MAX_SCRIPT_ELEMENT_SIZE {
                return Err(VmError::ScriptTooLarge);
            }

            self.stack.push(StackItem::Bytes(hash));
        } else {
            return Err(VmError::InvalidData);
        }

        Ok(())
    }

    fn equal(&mut self) -> Result<(), VmError> {
        let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
        let b = self.stack.pop().ok_or(VmError::EmptyStack)?;

        self.stack.push(StackItem::Bool(a == b));
        Ok(())
    }

    fn verify(&mut self) -> Result<(), VmError> {
        let top_elem = self.stack.pop().ok_or(VmError::EmptyStack)?;

        if top_elem != StackItem::Bool(true) {
            return Err(VmError::VerifyFailed);
        };
        Ok(())
    }

    fn equal_verify(&mut self) -> Result<(), VmError> {
        self.equal()?;
        self.verify()?;
        Ok(())
    }

    fn check_sig(&mut self) -> Result<(), VmError> {
        let pubkey = self.pop_bytes()?;
        let signature = self.pop_bytes()?;

        let valid = verify_signature(&pubkey, self.message, &signature);

        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }
        self.stack.push(StackItem::Bool(valid));

        Ok(())
    }

}
