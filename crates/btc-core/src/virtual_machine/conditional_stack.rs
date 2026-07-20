use crate::virtual_machine::{ExecutionFrame, VmError};

pub struct ConditionalStack {
    stack: Vec<ExecutionFrame>,
}

impl ConditionalStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push_frame(&mut self, frame: ExecutionFrame) {
        self.stack.push(frame)
    }

    pub fn pop_frame(&mut self) -> Result<ExecutionFrame, VmError> {
        self.stack.pop().ok_or(VmError::InvalidScriptFormat)
    }

    pub fn last_mut_frame(&mut self) -> Result<&mut ExecutionFrame, VmError> {
        self.stack.last_mut().ok_or(VmError::InvalidScriptFormat)
    }

    pub fn should_execute(&self) -> bool {
        self.stack.iter().all(|f| f.execute)
    }

    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }


}
