use std::cmp::Ordering;

use crate::{difficulty::{Difficulty, DifficultyErrors, constants::{EXPECTED_TIMESPAN, MAX_TARGET}}, types::BigUint256};

pub struct DifficultyAdjustment;

impl DifficultyAdjustment {
    pub fn next_bits(
        previous_bits: u32,
        mut actual_timespan: u32,
    ) -> Result<u32, DifficultyErrors>{

        let expected_timespan: u32 = EXPECTED_TIMESPAN; // in sec

        actual_timespan = actual_timespan.clamp(expected_timespan / 4, expected_timespan * 4);

        let previous_target = Difficulty::target_from_bits(previous_bits);

        let mut new_target = (BigUint256::from(previous_target).mul_u32(actual_timespan)).div_u32(expected_timespan);

        if new_target.cmp(&BigUint256::from(MAX_TARGET)) == Ordering::Greater{
            new_target = BigUint256::from(MAX_TARGET);
        };

        Difficulty::bits_from_target(&new_target.into_bytes())
    }
}