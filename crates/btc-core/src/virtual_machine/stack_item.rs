#[derive(Clone)]
pub enum StackItem {
    Bytes(Vec<u8>),
    Bool(bool)
}