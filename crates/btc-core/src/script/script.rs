use crate::script::OpCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptItem {
    Op(OpCode),
    PushData(Vec<u8>),
}

pub type Script = Vec<ScriptItem>;
