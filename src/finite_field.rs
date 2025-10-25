use std::ops::{Add, Div, Mul, Sub};

use num_bigint::BigUint;

#[derive(Debug)]
pub enum FieldElementError {
    InvalidNum(BigUint, BigUint),
    InvalidPrime(BigUint)
}

impl std::fmt::Display for FieldElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldElementError::InvalidNum(num, prime) => write!(f, "Invalid number: {} is greater than or equal to prime {}", num, prime),
            FieldElementError::InvalidPrime(prime) => write!(f, "Invalid prime: {} is less than or equal to 1", prime),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldElement {
    num: BigUint,
    prime: BigUint,
}

impl FieldElement {
    pub fn new(num: BigUint, prime: BigUint) -> Result<Self, FieldElementError> {
        if num >= prime {
            return Err(FieldElementError::InvalidNum(num, prime));
        }

        if prime <= BigUint::from(1u32) {
            return Err(FieldElementError::InvalidPrime(prime));
        }

        Ok(FieldElement { num, prime })
    }

    pub fn pow(self, exponent: BigUint) -> Self {
        let modified_exponent = &exponent % (&self.prime - BigUint::from(1u32));
        let num = self.num.modpow(&modified_exponent, &self.prime);

        FieldElement {
            num,
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

impl std::fmt::Display for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FieldElement({}, {})", self.num, self.prime)
    }
}

#[cfg(test)]
mod finite_field_tests {
    use super::*;

    #[test]
    fn test_new_and_eq() {
        let p = BigUint::from(13u32);
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(3u32), p.clone()).unwrap();
        let element3 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();

        assert_ne!(element1, element2);
        assert_eq!(element1, element3);
    }

    #[test]
    fn test_add() {
        let p = BigUint::from(13u32);
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(3u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();

        assert_eq!(element1 + element2, result);
    }

    #[test]
    fn test_subtract() {
        let p = BigUint::from(13u32);
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(12u32), p.clone()).unwrap();
        let element3 = FieldElement::new(BigUint::from(7u32), p.clone()).unwrap();
        let result1 = FieldElement::new(BigUint::from(3u32), p.clone()).unwrap();
        let result2 = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();

        assert_eq!(element1 - element2.clone(), result1);
        assert_eq!(element2 - element3, result2);
    }

    #[test]
    fn test_mul() {
        let p = BigUint::from(13u32);
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(12u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(11u32), p.clone()).unwrap();

        assert_eq!(element1 * element2, result);
    }

    #[test]
    fn test_div() {
        let p = BigUint::from(13u32);
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(12u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(11u32), p.clone()).unwrap();

        assert_eq!(element1 / element2, result);
    }

    #[test]
    fn test_pow() {
        let p = BigUint::from(13u32);
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(1u32), p.clone()).unwrap();

        assert_eq!(element1.pow(BigUint::from(12u32)), result);
    }

    #[test]
    fn test_display() {
        let p = BigUint::from(13u32);
        let element = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();
        assert_eq!(format!("{}", element), "FieldElement(5, 13)");
        let error = FieldElementError::InvalidNum(BigUint::from(15u32), p.clone());
        assert_eq!(format!("{}", error), "Invalid number: 15 is greater than or equal to prime 13");
    }
}
