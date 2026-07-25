#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SigVersion {
    Legacy,
    WitnessV0,
    Taproot,
    Tapscript,
}