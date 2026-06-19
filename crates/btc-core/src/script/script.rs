use crate::script::OpCode;

#[derive(Debug, Clone)]
pub enum ScriptItem {
    Op(OpCode),
    PushData(Vec<u8>),
}

pub type Script = Vec<ScriptItem>;
