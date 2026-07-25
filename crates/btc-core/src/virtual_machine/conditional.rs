use crate::{
    script::{OpCode, opcode::FlowControl},
    virtual_machine::{ExecutionFrame, StackOps, VirtualMachine, VmError},
};

impl FlowControl for VirtualMachine {
    fn op_if(&mut self) -> Result<(), VmError> {
        if !self.conditional_stack.should_execute() {
            Ok(self
                .conditional_stack
                .push_frame(ExecutionFrame::new(false)))
        } else {
            let frame = ExecutionFrame::new(self.pop_bool()?);
            Ok(self.conditional_stack.push_frame(frame))
        }
    }
    fn op_not_if(&mut self) -> Result<(), VmError> {
        if !self.conditional_stack.should_execute() {
            Ok(self
                .conditional_stack
                .push_frame(ExecutionFrame::new(false)))
        } else {
            let frame = ExecutionFrame::new(!self.pop_bool()?);
            Ok(self.conditional_stack.push_frame(frame))
        }
    }
    fn op_else(&mut self) -> Result<(), VmError> {
        let frame = self.conditional_stack.last_mut_frame()?;
        frame.mark_else()?;
        frame.flip();
        Ok(())
    }
    fn op_endif(&mut self) -> Result<(), VmError> {
        self.conditional_stack.pop_frame()?;
        Ok(())
    }

    fn op_return(&self) -> Result<(), VmError> {
        if !self.conditional_stack.should_execute() {
            return Ok(());
        }
        Err(VmError::ReturnOp)
    }
}

impl VirtualMachine {
    fn op_if_validate(&mut self) -> Result<(), VmError> {
        if !self.conditional_stack.should_execute() {
            Ok(self
                .conditional_stack
                .push_frame(ExecutionFrame::new(false)))
        } else {
            Ok(self.conditional_stack.push_frame(ExecutionFrame::new(true)))
        }
    }
    fn op_not_if_validate(&mut self) -> Result<(), VmError> {
        if !self.conditional_stack.should_execute() {
            Ok(self
                .conditional_stack
                .push_frame(ExecutionFrame::new(false)))
        } else {
            Ok(self.conditional_stack.push_frame(ExecutionFrame::new(true)))
        }
    }

    pub fn execute_conditionals(&mut self, opcode: &OpCode) -> Result<(), VmError> {
        match opcode {
            OpCode::If => self.op_if(),
            OpCode::NotIf => self.op_not_if(),
            OpCode::Else => self.op_else(),
            OpCode::EndIf => self.op_endif(),
            OpCode::Return => self.op_return(),
            _ => Err(VmError::InvalidOpcode),
        }
    }

    pub fn conditionals_syntax_validation(&mut self, opcode: &OpCode) -> Result<(), VmError> {
        self.conditional_stack.clear_stack();
        match opcode {
            OpCode::If => self.op_if_validate(),
            OpCode::NotIf => self.op_not_if_validate(),
            OpCode::Else => self.op_else(),
            OpCode::EndIf => self.op_endif(),
            _ => Err(VmError::InvalidOpcode),
        }
    }
}
