use crate::U256;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A 257-bit signed integer type.
/// It is represented as a 256-bit unsigned integer and a boolean
/// indicating whether the number is negative.
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct I257 {
    value: U256,
    is_negative: bool,
}

impl I257 {
    #[inline]
    pub fn new(value: U256, is_negative: bool) -> Self {
        I257 { value, is_negative }
    }

    #[inline]
    pub fn abs_value(&self) -> U256 {
        self.value
    }

    #[inline]
    pub fn sign(&self) -> i8 {
        if self.is_negative { -1 } else { 1 }
    }

    pub const ZERO: I257 = I257 {
        value: U256::ZERO,
        is_negative: false,
    };

    pub const NEGATIVE_ZERO: I257 = I257 {
        value: U256::ZERO,
        is_negative: true,
    };
}

impl From<U256> for I257 {
    fn from(val: U256) -> Self {
        I257 {
            value: val,
            is_negative: false,
        }
    }
}

/// Implementing the `Add` and `Sub` traits for I257 to handle addition and subtraction
impl Add for I257 {
    type Output = I257;

    fn add(self, other: I257) -> I257 {
        if self.is_negative == other.is_negative {
            I257 {
                value: self.value.saturating_add(other.value),
                is_negative: self.is_negative,
            }
        } else if self.abs_value() > other.abs_value() {
            I257 {
                value: self.value - other.value,
                is_negative: self.is_negative,
            }
        } else {
            I257 {
                value: other.value - self.value,
                is_negative: other.is_negative,
            }
        }
    }
}

impl AddAssign for I257 {
    fn add_assign(&mut self, other: I257) {
        *self = *self + other;
    }
}

impl Sub for I257 {
    type Output = I257;

    fn sub(self, other: I257) -> I257 {
        if self.is_negative != other.is_negative {
            I257 {
                value: self.value.saturating_add(other.value),
                is_negative: self.is_negative,
            }
        } else if self.abs_value() > other.abs_value() {
            I257 {
                value: self.value - other.value,
                is_negative: self.is_negative,
            }
        } else {
            I257 {
                value: other.value - self.value,
                is_negative: !self.is_negative,
            }
        }
    }
}

impl SubAssign for I257 {
    fn sub_assign(&mut self, other: I257) {
        *self = *self - other;
    }
}
