use std::{
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

use num_bigint::BigUint;

#[derive(Debug)]
pub enum FieldElementError {
    InvalidNum(BigUint, Arc<BigUint>),
    InvalidPrime(Arc<BigUint>),
}

impl std::fmt::Display for FieldElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldElementError::InvalidNum(num, prime) => write!(
                f,
                "Invalid number: {} is greater than or equal to prime {}",
                num, prime
            ),
            FieldElementError::InvalidPrime(prime) => {
                write!(f, "Invalid prime: {} is less than or equal to 1", prime)
            }
        }
    }
}

// Implement the `std::error::Error` trait.
// This is a "marker" trait that signals that our enum is a standard error type.
// It requires `Debug` and `Display` to be implemented first (as supertraits).
// This allows our error to be used with `?` and `Box<dyn Error>`, making it
// interoperable with other Rust libraries and error-handling mechanisms.
impl std::error::Error for FieldElementError {}

#[derive(Debug, Clone, PartialEq, Eq)]
// `FieldElement` uses an `Arc<BigUint>` for the prime modulus. This is a
// crucial performance optimization that allows many elements in the same field
// to share a single copy of the prime via atomic reference counting, avoiding
// repeated memory allocations for what is often a large number.
pub struct FieldElement {
    num: BigUint,
    prime: Arc<BigUint>,
}

impl FieldElement {
    pub fn new<E: Into<Arc<BigUint>>>(num: BigUint, prime: E) -> Result<Self, FieldElementError> {
        let prime_arc = prime.into();

        if num >= *prime_arc {
            return Err(FieldElementError::InvalidNum(num, prime_arc));
        }

        if *prime_arc <= BigUint::from(1u32) {
            return Err(FieldElementError::InvalidPrime(prime_arc));
        }

        Ok(FieldElement {
            num,
            prime: prime_arc,
        })
    }

    pub fn num(&self) -> &BigUint {
        &self.num
    }

    // This method uses the "Into Parameter" pattern for ergonomic API design.
    // By using a generic `E: Into<BigUint>`, this function can accept any type
    // that can be converted into a `BigUint` (e.g., `u32`, `u64`), making it
    // easier for the caller. See `note/02-into-parameter-pattern.md` for more
    // details on this pattern.
    pub fn pow<E: Into<BigUint>>(&self, exponent: E) -> FieldElement {
        let biguint_exponent = exponent.into();

        if biguint_exponent == BigUint::from(0u32) {
            return FieldElement {
                num: BigUint::from(1u32),
                prime: self.prime.clone(),
            };
        }

        let modified_exponent = biguint_exponent % (&*self.prime - BigUint::from(1u32));
        let num = self.num.modpow(&modified_exponent, &self.prime);

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.num == BigUint::from(0u32)
    }
}

// --- Operator Overloading Best Practice ---
//
// To provide maximum flexibility and ergonomics, we implement binary operators
// for all four combinations of references and owned values:
//
// 1. `&T + &T` (The core implementation with the actual logic)
// 2. `T + T`   (Ergonomic wrapper)
// 3. `&T + T`   (Ergonomic wrapper)
// 4. `T + &T`   (Ergonomic wrapper)
//
// This pattern is repeated for `Sub`, `Mul`, and `Div`. A macro could be used
// to reduce this boilerplate in the future. See `note/01-operator-overloading.md`
// for more details.
impl Add for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        let num = (&self.num + &rhs.num) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl Add for FieldElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl Add<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: &FieldElement) -> Self::Output {
        &self + rhs
    }
}

impl Add<FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: FieldElement) -> Self::Output {
        self + &rhs
    }
}

impl Add<&BigUint> for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: &BigUint) -> Self::Output {
        let num = (&self.num + rhs) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl Add<BigUint> for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: BigUint) -> Self::Output {
        self + &rhs
    }
}

impl Sub for &FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        // adding prime before subtracting to make sure the result is greater than 0
        let num = (&self.num + &*self.prime - &rhs.num) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl Sub for FieldElement {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl Sub<&FieldElement> for FieldElement {
    type Output = Self;

    fn sub(self, rhs: &FieldElement) -> Self::Output {
        &self - rhs
    }
}

impl Sub<FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: FieldElement) -> Self::Output {
        self - &rhs
    }
}

impl Mul for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        let num = (&self.num * &rhs.num) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl Mul for FieldElement {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: FieldElement) -> Self::Output {
        self * &rhs
    }
}

impl Mul<&FieldElement> for FieldElement {
    type Output = Self;

    fn mul(self, rhs: &FieldElement) -> Self::Output {
        &self * rhs
    }
}

impl Mul<&BigUint> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: &BigUint) -> Self::Output {
        let num = (&self.num * rhs) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl Mul<&BigUint> for FieldElement {
    type Output = Self;

    fn mul(self, rhs: &BigUint) -> Self::Output {
        &self * rhs
    }
}

impl Mul<BigUint> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: BigUint) -> Self::Output {
        self * &rhs
    }
}

impl Mul<BigUint> for FieldElement {
    type Output = Self;

    fn mul(self, rhs: BigUint) -> Self::Output {
        self * &rhs
    }
}

impl Div for &FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            panic!("Two elements are not in the same field")
        }

        if rhs.num == BigUint::from(0u32) {
            panic!("Division by zero")
        }

        let p_minus_2 = &*self.prime - BigUint::from(2u32);
        let num = (&self.num * rhs.num.modpow(&p_minus_2, &self.prime)) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl Div for FieldElement {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        &self / &rhs
    }
}

impl Div<&FieldElement> for FieldElement {
    type Output = Self;

    fn div(self, rhs: &FieldElement) -> Self::Output {
        &self / rhs
    }
}

impl Div<FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: FieldElement) -> Self::Output {
        self / &rhs
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
        let p = Arc::new(BigUint::from(13u32));
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(3u32), p.clone()).unwrap();
        let element3 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();

        assert_ne!(element1, element2);
        assert_eq!(element1, element3);
    }

    #[test]
    fn test_add() {
        let p = Arc::new(BigUint::from(13u32));
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(3u32), p.clone()).unwrap();
        let element3 = FieldElement::new(BigUint::from(4u32), p.clone()).unwrap();
        let result1 = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();
        let result2 = FieldElement::new(BigUint::from(7u32), p.clone()).unwrap();

        assert_eq!(element1 + &element2, result1);
        assert_eq!(element2 + element3, result2);
    }

    #[test]
    fn test_subtract() {
        let p = Arc::new(BigUint::from(13u32));
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(12u32), p.clone()).unwrap();
        let element3 = FieldElement::new(BigUint::from(7u32), p.clone()).unwrap();
        let result1 = FieldElement::new(BigUint::from(3u32), p.clone()).unwrap();
        let result2 = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();

        assert_eq!(element1 - &element2, result1);
        assert_eq!(element2 - element3, result2);
    }

    #[test]
    fn test_mul() {
        let p = Arc::new(BigUint::from(13u32));
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(12u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(11u32), p.clone()).unwrap();

        assert_eq!(element1 * element2, result);
    }

    #[test]
    fn test_div() {
        let p = Arc::new(BigUint::from(13u32));
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let element2 = FieldElement::new(BigUint::from(12u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(11u32), p.clone()).unwrap();

        assert_eq!(element1 / element2, result);
    }

    #[test]
    fn test_pow() {
        let p = Arc::new(BigUint::from(13u32));
        let element1 = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        let result = FieldElement::new(BigUint::from(1u32), p.clone()).unwrap();

        assert_eq!(element1.pow(12u32), result);
    }

    #[test]
    fn test_display() {
        let p = Arc::new(BigUint::from(13u32));
        let element = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();
        assert_eq!(format!("{}", element), "FieldElement(5, 13)");
        let error = FieldElementError::InvalidNum(BigUint::from(15u32), p.clone());
        assert_eq!(
            format!("{}", error),
            "Invalid number: 15 is greater than or equal to prime 13"
        );
    }
}
