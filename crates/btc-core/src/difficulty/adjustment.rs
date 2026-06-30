use std::cmp::Ordering;

use crate::{
    difficulty::{
        Difficulty, DifficultyErrors,
        constants::{EXPECTED_TIMESPAN, MAX_TARGET},
    },
    types::BigUint256,
};

pub struct DifficultyAdjustment;

impl DifficultyAdjustment {
    pub fn next_bits(
        previous_bits: u32,
        mut actual_timespan: u32,
    ) -> Result<u32, DifficultyErrors> {
        let expected_timespan: u32 = EXPECTED_TIMESPAN; // in sec

        actual_timespan = actual_timespan.clamp(expected_timespan / 4, expected_timespan * 4);

        let previous_target = Difficulty::target_from_bits(previous_bits);

        let mut new_target =
            (BigUint256::from(previous_target).mul_u32(actual_timespan)).div_u32(expected_timespan);

        if new_target.cmp(&BigUint256::from(MAX_TARGET)) == Ordering::Greater {
            new_target = BigUint256::from(MAX_TARGET);
        };

        Difficulty::bits_from_target(&new_target.into_bytes())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn difficulty_increase_if_block_mined_fast() {
        let target: [u8; 32] = [
            0x00, 0x00, 0x70, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
        ];

        let previous_bits= Difficulty::bits_from_target(&target).unwrap();

        let actual_timespan = 2016 * 540;

        let res = DifficultyAdjustment::next_bits(previous_bits, actual_timespan).unwrap();

        let new_target = Difficulty::target_from_bits(res);

        assert!(BigUint256::from(new_target).cmp(&BigUint256(target)) == Ordering::Less)

    }


    #[test]
    fn difficulty_adjustment_is_clamped() {
        let target: [u8; 32] = [
            0x00, 0x00, 0x70, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
        ];

        let previous_bits= Difficulty::bits_from_target(&target).unwrap();

        let actual_timespan = 1;

        let clamped_timespan = actual_timespan.clamp(EXPECTED_TIMESPAN/4, EXPECTED_TIMESPAN*4);

        let res = DifficultyAdjustment::next_bits(previous_bits, actual_timespan).unwrap();
        let res2 = DifficultyAdjustment::next_bits(previous_bits, clamped_timespan).unwrap();

        let new_target = Difficulty::target_from_bits(res);
        let new_clamped_target = Difficulty::target_from_bits(res2);

        assert!(BigUint256::from(new_target).cmp(&BigUint256::from(new_clamped_target)) == Ordering::Equal)
    }
}
