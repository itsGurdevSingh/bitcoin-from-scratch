use crate::virtual_machine::VmError;

#[derive(Debug, Clone)]
pub struct ExecutionFrame {
    pub execute: bool,
    pub else_seen: bool,
}

impl ExecutionFrame {
    pub fn new(execute: bool) -> Self {
        Self {
            execute,
            else_seen: false,
        }
    }

    pub fn flip(&mut self) {
        self.execute = !self.execute
    }

    pub fn mark_else(&mut self) -> Result<(), VmError> {
        if self.else_seen {
            return Err(VmError::InvalidSriptFormat);
        } else {
            self.else_seen = true;
            Ok(())
        }
    }
}
