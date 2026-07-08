use std::ops::Neg;

use crate::{
    crypto::{
        hash::{hash160, hash256, sha1},
        sha256, verify_signature,
    },
    script::opcode::FlowControl,
    script::{OpCode, OpCodeTrait, Script, ScriptItem},
    virtual_machine::{
        MAX_SCRIPT_ELEMENT_SIZE, MAX_SCRIPT_SIZE, MAX_STACK_SIZE, StackItem, StackOps, VmError,
        conditional_stack::ConditionalStack,
    },
};

pub struct VirtualMachine<'a> {
    stack: Vec<StackItem>,
    message: &'a [u8],
    pub conditional_stack: ConditionalStack,
}

impl<'a> StackOps for VirtualMachine<'a> {
    fn push_bytes(&mut self, bytes: Vec<u8>) -> Result<(), VmError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }

        if bytes.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(VmError::ScriptTooLarge);
        }

        self.stack.push(StackItem::Bytes(bytes));
        Ok(())
    }

    fn push_number(&mut self, n: i64) -> Result<(), VmError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }
        // Convert i64 to big-endian bytes (8 bytes)
        self.stack.push(StackItem::Bytes(n.to_be_bytes().to_vec()));

        Ok(())
    }

    fn push_bool(&mut self, value: bool) -> Result<(), VmError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }

        // In Bitcoin Script, boolean is usually represented as 0x01 or empty/0x00
        let byte = if value { 1u8 } else { 0u8 };
        self.stack.push(StackItem::Bytes(vec![byte]));
        Ok(())
    }

    fn pop_bytes(&mut self) -> Result<Vec<u8>, VmError> {
        match self.stack.pop() {
            Some(StackItem::Bytes(bytes)) => Ok(bytes),
            None => Err(VmError::EmptyStack),
        }
    }

    fn pop_number(&mut self) -> Result<i64, VmError> {
        let bytes = self.pop_bytes()?;

        if bytes.is_empty() {
            return Ok(0);
        }

        // Convert big-endian bytes back to i64
        // We pad to 8 bytes if shorter
        let mut padded = [0u8; 8];
        let start = 8usize.saturating_sub(bytes.len());
        padded[start..].copy_from_slice(&bytes);

        Ok(i64::from_be_bytes(padded))
    }

    fn pop_bool(&mut self) -> Result<bool, VmError> {
        let bytes = self.pop_bytes()?;

        // Bitcoin Script truthy rules:
        // - Empty stack item or [0] or [0,0,...] = false
        // - Anything else = true
        let is_true = !bytes.is_empty() && bytes.iter().any(|&b| b != 0);

        Ok(is_true)
    }

    fn last_bytes(&self) -> Result<&Vec<u8>, VmError> {
        let last_elem = self.stack.last().ok_or(VmError::EmptyStack)?;

        match last_elem {
            StackItem::Bytes(bytes) => Ok(bytes),
        }
    }

    fn last_number(&self) -> Result<i64, VmError> {
        let bytes = self.last_bytes()?;

        if bytes.is_empty() {
            return Ok(0);
        }

        // Convert big-endian bytes back to i64
        // We pad to 8 bytes if shorter
        let mut padded = [0u8; 8];
        let start = 8usize.saturating_sub(bytes.len());
        padded[start..].copy_from_slice(&bytes);

        Ok(i64::from_be_bytes(padded))
    }

    fn last_bool(&self) -> Result<bool, VmError> {
        let bytes = self.last_bytes()?;

        let is_true = !bytes.is_empty() && bytes.iter().any(|&b| b != 0);

        Ok(is_true)
    }
}

impl<'a> VirtualMachine<'a> {
    pub fn new(message: &'a [u8]) -> Self {
        Self {
            stack: Vec::new(),
            conditional_stack: ConditionalStack::new(),
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
                    // if any condition stop this instruction to perform.
                    if !self.conditional_stack.should_execute() {
                        continue;
                    }

                    self.push_bytes(data.clone())?;
                }

                ScriptItem::Op(op) => match op {
                    OpCode::If | OpCode::NotIf | OpCode::Else | OpCode::EndIf | OpCode::Return => {
                        self.execute_conditionals(op)?
                    }
                    _ => {
                        if !self.conditional_stack.should_execute() {
                            continue;
                        }
                        self.execute_opcode(op)?;
                    }
                },
            }
        }

        self.verify_final_stack()
    }

    fn verify_final_stack(&mut self) -> Result<(), VmError> {
        match self.pop_bool()? {
            true => return Ok(()),
            false => return Err(VmError::VerifyFailed),
        }
    }

    fn execute_conditionals(&mut self, opcode: &OpCode) -> Result<(), VmError> {
        match opcode {
            OpCode::If => self.op_if(),
            OpCode::NotIf => self.op_not_if(),
            OpCode::Else => self.op_else(),
            OpCode::EndIf => self.op_endif(),
            OpCode::Return => self.op_return(),
            _ => Err(VmError::InvalidOpcode),
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

            OpCode::Op0 => self.op_0(),
            OpCode::Op1 => self.op_1(),
            OpCode::Op2 => self.op_2(),
            OpCode::Op3 => self.op_3(),
            OpCode::Op4 => self.op_4(),
            OpCode::Op5 => self.op_5(),
            OpCode::Op6 => self.op_6(),
            OpCode::Op7 => self.op_7(),
            OpCode::Op8 => self.op_8(),
            OpCode::Op9 => self.op_9(),
            OpCode::Op10 => self.op_10(),
            OpCode::Op11 => self.op_11(),
            OpCode::Op12 => self.op_12(),
            OpCode::Op13 => self.op_13(),
            OpCode::Op14 => self.op_14(),
            OpCode::Op15 => self.op_15(),
            OpCode::Op16 => self.op_16(),
            OpCode::Op1Negate => self.op_1negate(),

            OpCode::Drop => self.drop(),
            OpCode::Swap => self.swap(),
            OpCode::Over => self.over(),
            OpCode::Depth => self.depth(),

            OpCode::Nip => self.nip(),
            OpCode::Tuck => self.tuck(),
            OpCode::Dup2 => self.dup_2(),
            OpCode::Drop2 => self.drop_2(),
            OpCode::IfDup => self.if_dup(),
            OpCode::NumEqual => self.num_equal(),

            OpCode::Add => self.add(),
            OpCode::Sub => self.sub(),
            OpCode::Negate => self.negate(),
            OpCode::Abs => self.abs(),
            OpCode::Not => self.not(),
            OpCode::NotEqual0 => self.not_equal_0(),

            OpCode::BoolAnd => self.bool_and(),
            OpCode::BoolOr => self.bool_or(),
            OpCode::GreaterThan => self.grater_than(),
            OpCode::LessThan => self.less_than(),
            OpCode::Max => self.max(),
            OpCode::Min => self.min(),
            OpCode::WithIn => self.within(),

            _ => Err(VmError::InvalidOpcode),
        }
    }

    fn ensure_stack_size(&self, size: u32) -> Result<(), VmError> {
        if self.stack.len() >= size as usize {
            Ok(())
        } else {
            Err(VmError::StackUnderflow)
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
        let top_elem = self.pop_bytes()?;
        let hash = hash160(&top_elem).to_vec();
        self.push_bytes(hash)?;
        Ok(())
    }

    fn hash256(&mut self) -> Result<(), VmError> {
        let a = self.pop_bytes()?;
        self.push_bytes(hash256(&a).to_vec())
    }

    fn sha1(&mut self) -> Result<(), VmError> {
        let a = self.pop_bytes()?;
        self.push_bytes(sha1(&a).to_vec())
    }

    fn sha256(&mut self) -> Result<(), VmError> {
        let a = self.pop_bytes()?;
        self.push_bytes(sha256(&a).to_vec())
    }

    fn equal(&mut self) -> Result<(), VmError> {
        let a = self.stack.pop().ok_or(VmError::EmptyStack)?;
        let b = self.stack.pop().ok_or(VmError::EmptyStack)?;

        self.push_bool(a == b)?;
        Ok(())
    }

    fn verify(&mut self) -> Result<(), VmError> {
        let is_valid = self.pop_bool()?;

        if is_valid {
            return Ok(());
        };
        Err(VmError::VerifyFailed)
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
        self.push_bool(valid)?;

        Ok(())
    }

    fn check_sig_verify(&mut self) -> Result<(), VmError> {
        self.check_sig()?;
        self.verify()
    }

    fn op_1negate(&mut self) -> Result<(), VmError> {
        self.push_number(-1)
    }

    fn op_0(&mut self) -> Result<(), VmError> {
        self.push_bool(false)
    }
    fn op_1(&mut self) -> Result<(), VmError> {
        self.push_number(1)
    }
    fn op_2(&mut self) -> Result<(), VmError> {
        self.push_number(2)
    }
    fn op_3(&mut self) -> Result<(), VmError> {
        self.push_number(3)
    }
    fn op_4(&mut self) -> Result<(), VmError> {
        self.push_number(4)
    }
    fn op_5(&mut self) -> Result<(), VmError> {
        self.push_number(5)
    }
    fn op_6(&mut self) -> Result<(), VmError> {
        self.push_number(6)
    }
    fn op_7(&mut self) -> Result<(), VmError> {
        self.push_number(7)
    }
    fn op_8(&mut self) -> Result<(), VmError> {
        self.push_number(8)
    }
    fn op_9(&mut self) -> Result<(), VmError> {
        self.push_number(9)
    }
    fn op_10(&mut self) -> Result<(), VmError> {
        self.push_number(10)
    }
    fn op_11(&mut self) -> Result<(), VmError> {
        self.push_number(11)
    }
    fn op_12(&mut self) -> Result<(), VmError> {
        self.push_number(12)
    }
    fn op_13(&mut self) -> Result<(), VmError> {
        self.push_number(13)
    }
    fn op_14(&mut self) -> Result<(), VmError> {
        self.push_number(14)
    }
    fn op_15(&mut self) -> Result<(), VmError> {
        self.push_number(15)
    }
    fn op_16(&mut self) -> Result<(), VmError> {
        self.push_number(16)
    }

    fn drop(&mut self) -> Result<(), VmError> {
        self.pop_bytes()?;
        Ok(())
    }
    fn swap(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_bytes()?;
        let b = self.pop_bytes()?;

        self.push_bytes(a)?;
        self.push_bytes(b)
    }

    fn over(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;
        let a = self.stack[self.stack.len() - 2].clone();
        self.stack.push(a);

        Ok(())
    }
    fn depth(&mut self) -> Result<(), VmError> {
        let depth = self.stack.len();

        self.push_number(depth as i64)
    }
    fn nip(&mut self) -> Result<(), VmError> {
        let a = self.pop_bytes()?;
        self.pop_bytes()?;

        self.push_bytes(a)
    }
    fn tuck(&mut self) -> Result<(), VmError> {
        if !(self.stack.len() < MAX_STACK_SIZE) {
            return Err(VmError::StackOverflow);
        }
        self.ensure_stack_size(2)?;

        let a = self.pop_bytes()?;
        let b = self.pop_bytes()?;

        self.push_bytes(a.clone())?;
        self.push_bytes(b)?;
        self.push_bytes(a)
    }
    fn drop_2(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        self.pop_bytes()?;
        self.pop_bytes()?;

        Ok(())
    }
    fn dup_2(&mut self) -> Result<(), VmError> {
        if !(self.stack.len() < MAX_STACK_SIZE - 2) {
            return Err(VmError::StackOverflow);
        };

        self.ensure_stack_size(2)?;

        let a = self.stack[self.stack.len() - 1].clone();
        let b = self.stack[self.stack.len() - 2].clone();

        self.stack.push(b);
        self.stack.push(a);

        Ok(())
    }

    fn if_dup(&mut self) -> Result<(), VmError> {
        if self.last_bool()? {
            let a = self.last_bytes()?;
            self.push_bytes(a.clone())?;
        };
        Ok(())
    }

    fn num_equal(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        if a == b {
            self.push_bool(true)
        } else {
            self.push_bool(false)
        }
    }

    fn add(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        self.push_number(a + b)
    }
    fn sub(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        self.push_number(b - a)
    }
    fn negate(&mut self) -> Result<(), VmError> {
        let x = self.pop_number()?;

        self.push_number(x.neg())
    }
    fn abs(&mut self) -> Result<(), VmError> {
        let x = self.pop_number()?;
        self.push_number(x.abs())
    }
    fn not(&mut self) -> Result<(), VmError> {
        let x = self.pop_bool()?;
        self.push_bool(!x)
    }
    fn not_equal_0(&mut self) -> Result<(), VmError> {
        let x = self.pop_bool()?;
        self.push_bool(x)
    }
    fn bool_and(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        if self.pop_bool()? && self.pop_bool()? {
            self.push_bool(true)
        } else {
            self.push_bool(false)
        }
    }
    fn bool_or(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        if self.pop_bool()? || self.pop_bool()? {
            self.push_bool(true)
        } else {
            self.push_bool(false)
        }
    }

    fn grater_than(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        self.push_bool(b < a)
    }
    fn less_than(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        self.push_bool(b > a)
    }
    fn max(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        if a < b {
            self.push_number(b)
        } else {
            self.push_number(a)
        }
    }
    fn min(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(2)?;

        let a = self.pop_number()?;
        let b = self.pop_number()?;

        if a < b {
            self.push_number(a)
        } else {
            self.push_number(b)
        }
    }
    fn within(&mut self) -> Result<(), VmError> {
        self.ensure_stack_size(3)?;

        let max = self.pop_number()?;
        let min = self.pop_number()?;
        let x = self.pop_number()?;

        self.push_bool(min <= x && x < max)
    }
}
