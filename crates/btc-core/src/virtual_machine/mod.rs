pub mod vm;
pub mod error;
pub mod stack_item;
pub mod config;

pub use vm::VirtualMachine;
pub use error::VmError;
pub use stack_item::StackItem;
pub use config::{
    MAX_SCRIPT_ELEMENT_SIZE,
    MAX_SCRIPT_SIZE,
    MAX_STACK_SIZE
};