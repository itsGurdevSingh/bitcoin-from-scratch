use crate::{script::opcode::FlowControl, virtual_machine::{ExecutionFrame, StackOps, VirtualMachine, VmError}};

impl<'a> FlowControl for VirtualMachine<'a> {
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
