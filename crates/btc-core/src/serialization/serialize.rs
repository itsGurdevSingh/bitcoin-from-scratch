pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8>;
}