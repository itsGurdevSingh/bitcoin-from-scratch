pub trait BitcoinDeserialize: Sized {
    type Error;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error>;
}