pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time:u32,
}