use std::ops::{Add, Div, Mul, Sub};

use num_bigint::BigUint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldElement {
    num: BigUint,
    prime: BigUint,
}

impl FieldElement {
    pub fn new(num: BigUint, prime: BigUint) -> Result<Self, String> {
        if num >= prime {
            return Err(format!(
                "Num {} is not in field range 0 to {}",
                num,
                &prime - 1u32 // prime - 1u32 would also work thanks to "auto borrowing"
            ));
        }

        if prime <= BigUint::from(1u32) {
            return Err("Invalid prime".to_string());
        }

        Ok(FieldElement { num, prime })
    }

    pub fn pow(self, exponent: BigUint) -> Self {
        let modified_exponent = &exponent % (&self.prime - BigUint::from(1u32));
        let num = self.num.modpow(&modified_exponent, &self.prime);

        FieldElement {
            num: num,
            prime: self.prime,
        }
    }
}

impl Add for FieldElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        let num = (self.num + rhs.num) % &self.prime;

        FieldElement {
            num,
            prime: self.prime,
        }
    }
}

impl Sub for FieldElement {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        // adding prime before subtracting to make sure the result is greater than 0
        let num = (self.num + &self.prime - rhs.num) % &self.prime;

        FieldElement {
            num,
            prime: self.prime,
        }
    }
}

impl Mul for FieldElement {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        let num = (self.num * rhs.num) % &self.prime;

        FieldElement {
            num,
            prime: self.prime,
        }
    }
}

impl Div for FieldElement {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        if rhs.num == BigUint::from(0u32) {
            panic!("Division by zero")
        }

        let p_minus_2 = &self.prime - BigUint::from(2u32);
        let num = (self.num * rhs.num.modpow(&p_minus_2, &self.prime)) % &self.prime;

        FieldElement {
            num,
            prime: self.prime,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
