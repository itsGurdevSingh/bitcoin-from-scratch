use std::{
    cmp::Ordering,
    ops::{Add, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BigUint256(pub [u8; 32]);

impl From<[u8; 32]> for BigUint256 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl BigUint256 {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl BigUint256 {
    pub fn mul_u64(&self, rhs: u64) -> Self {
        let mut unit256 = Self([0u8; 32]);

        let mut carry: u64 = 0;

        for i in (0..32).rev() {
            let byte = self.0[i];

            let res: u64 = (byte as u64) * rhs + carry;

            unit256.0[i] = (res % 256) as u8;

            carry = res / 256;
        }
        unit256
    }

    pub fn div_u64(&self, rhs: u64) -> Self {
        let mut unit256 = Self([0u8; 32]);

        let mut remainder = 0;

        for i in 0..32 {
            let byte = self.0[i];

            let value = remainder * 256 + byte as u64;

            unit256.0[i] = (value / rhs) as u8;
            remainder = value % rhs;
        }

        unit256
    }
}

impl Add for BigUint256 {
    type Output = BigUint256;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Self([0u8; 32]);
        let mut carry: u8 = 0;

        for i in (0..32).rev() {
            let sum = self.0[i] as u16 + rhs.0[i] as u16 + carry as u16;

            carry = (sum / 256) as u8;

            result.0[i] = (sum % 256) as u8;
        }
        result
    }
}

impl Sub for BigUint256 {
    type Output = BigUint256;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut borrow: u32 = 0;
        let mut result = Self([0u8; 32]);

        for i in (0..32).rev() {
            let mut diff = self.0[i] as i16 - rhs.0[i] as i16 - borrow as i16;

            if diff < 0 {
                diff += 256;
                borrow = 1;
            } else {
                borrow = 0;
            }

            result.0[i] = diff as u8;
        }

        result
    }
}

impl PartialOrd for BigUint256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        {
            for i in 0..32 {
                if self.0[i] > other.0[i] {
                    return Some(Ordering::Greater);
                }

                if self.0[i] < other.0[i] {
                    return Some(Ordering::Less);
                }
            }

            Some(Ordering::Equal)
        }
    }
}
