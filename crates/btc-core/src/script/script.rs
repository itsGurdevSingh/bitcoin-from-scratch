pub enum ScriptItem {
    Op(OpCode),
    PushData(vec<u8>),
}

pub type Script = vec<ScriptItem> ;