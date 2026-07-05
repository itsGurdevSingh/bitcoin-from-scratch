#[derive(Clone, PartialEq, Eq)]
pub enum StackItem {
    Bytes(Vec<u8>),
    Bool(bool)
}