use crate::difficulty::constants::{DUMMY_BITS, DUMMY_TARGET};

pub struct Difficulty;

impl Difficulty {
    pub fn target_from_bits(bits: u32) -> [u8; 32] {
        match bits {
            DUMMY_BITS => DUMMY_TARGET,
            _ => panic!("unsupported bits"),
        }
    }
}