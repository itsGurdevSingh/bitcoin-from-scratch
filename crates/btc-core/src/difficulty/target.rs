use crate::difficulty::DifficultyErrors;


pub struct Difficulty;

impl Difficulty {
    pub fn target_from_bits(bits: u32) -> [u8; 32] {

        let mut target: [u8; 32] = [0u8; 32];

        let exponent = (bits >> 24) as usize;

        let first_index = 32 - exponent;

        target[first_index] = (bits >> 16) as u8;
        target[first_index + 1] = (bits >> 8) as u8;
        target[first_index + 2] = bits as u8;
        target

    }

    pub fn bits_from_target(target: &[u8; 32]) -> Result<u32, DifficultyErrors> {


        // This implementation only supports compact targets that can be
        // normalized without overflowing the exponent. Targets beginning
        // at byte 0 are rejected because they would require an exponent of 33.
        if target[0] != 0 {
            return Err(DifficultyErrors::InvalidTarget);
        }

        let mut mantissa: [u8; 3] = [0u8; 3];
        let mut first_index: u8 = 0;

        let mut matissa_idx: usize = 0;

        for (idx, byte) in target.iter().enumerate() {
            if matissa_idx < 3 {
                if *byte != 0 as u8 {
                    mantissa[matissa_idx] = *byte;
                    matissa_idx += 1;
                    if first_index == 0 {
                        first_index = idx as u8
                    }
                }
            }
        }

        // if MSB(most significant bit ) is one we will seal the mantissa value.
        // in u8 we have limit 0 to 255 . 
        // when MSB start occuring in u8 is = 10000000 which is equal to 128 in decimal and 0x80 in hex we are comparing hex.
        if mantissa[0] >= 0x80 {
            mantissa = [0, mantissa[0], mantissa[1]];
            
            first_index -= 1; // decrease exponent to sift exponent.
        }

        let bits = ((32 - first_index as u32) << 24) // exponent 32 - first index .
            | ((mantissa[0] as u32) << 16)
            | ((mantissa[1] as u32) << 8) 
            | (mantissa[2] as u32);

            Ok(bits)
    }
}


#[cfg(test)]
mod test {

use super::*;

    #[test]
    fn encode_and_decode_valid_target() {
        let target: [u8; 32] = [
        0x00, 0x00, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        ];

    let expected_target = [
        0x00, 0x00, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        ];

    let bits = Difficulty::bits_from_target(&target).unwrap();


    let target_res = Difficulty::target_from_bits(bits);

    assert_eq!(expected_target, target_res);
    }


    #[test]
    fn invalid_target_return_error() {
        // first bit is non zero
        let target: [u8; 32] = [
        0xff, 0x00, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(Difficulty::bits_from_target(&target), Err(DifficultyErrors::InvalidTarget));

    }
}
