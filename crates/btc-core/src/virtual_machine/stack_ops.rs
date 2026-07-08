use crate::virtual_machine::VmError;

pub trait StackOps {
    fn pop_bytes(&mut self) -> Result<Vec<u8>, VmError>;
    fn pop_number(&mut self) -> Result<i64, VmError>;
    fn pop_bool(&mut self) -> Result<bool, VmError>;
    fn push_bytes(&mut self, bytes: Vec<u8>)-> Result<(), VmError>;
    fn push_number(&mut self, n: i64) -> Result<(), VmError>;
    fn push_bool(&mut self, value: bool)-> Result<(), VmError>;
    fn last_bytes(&self) -> Result<&Vec<u8>, VmError>;
    fn last_number(&self) -> Result<i64, VmError>;
    fn last_bool(&self) -> Result<bool, VmError>;
}