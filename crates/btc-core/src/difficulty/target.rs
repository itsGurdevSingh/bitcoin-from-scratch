
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

    pub fn bits_form_target(target: &[u8; 32]) -> u32 {
        let mut mantissa: [u8; 3] = [0u8; 3];
        let mut exponent: u8 = 0;

        let mut matissa_idx: usize = 0;

        for (idx, byte) in target.iter().enumerate() {
            if matissa_idx < 3 {
                if *byte != 0 as u8 {
                    mantissa[matissa_idx] = *byte;
                    matissa_idx += 1;
                    if exponent == 0 {
                        exponent = idx as u8
                    }
                }
            }
        }

        let bits = ((32 - exponent as u32) << 24)
            | ((mantissa[0] as u32) << 16)
            | ((mantissa[1] as u32) << 8) 
            | (mantissa[2] as u32);

            bits
    }
}
