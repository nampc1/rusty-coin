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

        if self.is_zero() {
            return FieldElement {
                num: BigUint::from(0u32),
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

macro_rules! impl_field_element_wrappers {
    ($trait:ident, $method:ident) => {
        impl $trait<FieldElement> for FieldElement {
            type Output = FieldElement;

            fn $method(self, rhs: FieldElement) -> FieldElement {
                (&self).$method(&rhs)
            }
        }

        impl $trait<&FieldElement> for FieldElement {
            type Output = FieldElement;

            fn $method(self, rhs: &FieldElement) -> FieldElement {
                (&self).$method(rhs)
            }
        }

        impl $trait<FieldElement> for &FieldElement {
            type Output = FieldElement;

            fn $method(self, rhs: FieldElement) -> FieldElement {
                self.$method(&rhs)
            }
        }
    };
}

macro_rules! impl_biguint_wrappers {
    ($trait:ident, $method:ident) => {
        impl $trait<BigUint> for &FieldElement {
            type Output = FieldElement;

            fn $method(self, rhs: BigUint) -> Self::Output {
                self.$method(&rhs)
            }
        }

        impl $trait<BigUint> for FieldElement {
            type Output = FieldElement;

            fn $method(self, rhs: BigUint) -> Self::Output {
                (&self).$method(&rhs)
            }
        }

        impl $trait<&BigUint> for FieldElement {
            type Output = FieldElement;

            fn $method(self, rhs: &BigUint) -> Self::Output {
                (&self).$method(rhs)
            }
        }
    };
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

impl_field_element_wrappers!(Add, add);

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

impl_biguint_wrappers!(Add, add);

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

impl_field_element_wrappers!(Sub, sub);

impl Sub<&BigUint> for &FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: &BigUint) -> Self::Output {
        // adding prime before subtracting to make sure the result is greater than 0
        let num = (&self.num + &*self.prime - (rhs % &*self.prime)) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl_biguint_wrappers!(Sub, sub);

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

impl_field_element_wrappers!(Mul, mul);

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

impl_biguint_wrappers!(Mul, mul);

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

impl_field_element_wrappers!(Div, div);

impl Div<&BigUint> for &FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: &BigUint) -> Self::Output {
        if rhs == &BigUint::from(0u32) {
            panic!("Division by zero")
        }

        let p_minus_2 = &*self.prime - BigUint::from(2u32);
        let num = (&self.num * rhs.modpow(&p_minus_2, &self.prime)) % &*self.prime;

        FieldElement {
            num,
            prime: self.prime.clone(),
        }
    }
}

impl_biguint_wrappers!(Div, div);

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
    fn test_pow_zero_base() {
        let p = Arc::new(BigUint::from(13u32));
        let element = FieldElement::new(BigUint::from(0u32), p.clone()).unwrap();
        let result = element.pow(12u32);
        assert_eq!(result.num, BigUint::from(0u32));
    }

    #[test]
    fn test_pow_zero_base_zero_exponent() {
        let p = Arc::new(BigUint::from(13u32));
        let element = FieldElement::new(BigUint::from(0u32), p.clone()).unwrap();
        // 0^0 should be 1. This ensures the exponent==0 check takes precedence over base==0.
        let result = element.pow(0u32);
        assert_eq!(result.num, BigUint::from(1u32));
    }

    #[test]
    fn test_pow_zero_exponent() {
        let p = Arc::new(BigUint::from(13u32));
        let element = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();
        let result = element.pow(0u32);
        assert_eq!(result.num, BigUint::from(1u32));
    }

    #[test]
    fn test_pow_large_exponent() {
        let p = Arc::new(BigUint::from(13u32));
        let element = FieldElement::new(BigUint::from(2u32), p.clone()).unwrap();
        // 2^(12*100 + 1) = 2^1 = 2 (mod 13) by Fermat's Little Theorem
        let large_exponent = BigUint::from(1201u32);
        let result = element.pow(large_exponent);
        assert_eq!(result.num, BigUint::from(2u32));
    }

    #[test]
    fn test_prime_two() {
        let p = Arc::new(BigUint::from(2u32));
        let zero = FieldElement::new(BigUint::from(0u32), p.clone()).unwrap();
        let one = FieldElement::new(BigUint::from(1u32), p.clone()).unwrap();

        // 1 + 1 = 0 (mod 2)
        assert_eq!(&one + &one, zero);
        // 0 - 1 = 1 (mod 2)
        assert_eq!(&zero - &one, one);
        // 1 * 1 = 1 (mod 2)
        assert_eq!(&one * &one, one);
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

    #[test]
    fn test_new_invalid_prime() {
        let p = BigUint::from(1u32);
        let res = FieldElement::new(BigUint::from(0u32), Arc::new(p));
        assert!(matches!(res, Err(FieldElementError::InvalidPrime(_))));
    }

    #[test]
    fn test_new_invalid_num() {
        let p = Arc::new(BigUint::from(13u32));
        let res = FieldElement::new(BigUint::from(13u32), p);
        assert!(matches!(res, Err(FieldElementError::InvalidNum(_, _))));
    }

    #[test]
    #[should_panic(expected = "Two elements are not in the same field")]
    fn test_add_mismatched_fields() {
        let p1 = Arc::new(BigUint::from(13u32));
        let p2 = Arc::new(BigUint::from(17u32));
        let a = FieldElement::new(BigUint::from(1u32), p1).unwrap();
        let b = FieldElement::new(BigUint::from(1u32), p2).unwrap();
        let _ = a + b;
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero_element() {
        let p = Arc::new(BigUint::from(13u32));
        let a = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();
        let b = FieldElement::new(BigUint::from(0u32), p).unwrap();
        let _ = a / b;
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero_biguint() {
        let p = Arc::new(BigUint::from(13u32));
        let a = FieldElement::new(BigUint::from(5u32), p).unwrap();
        let _ = a / BigUint::from(0u32);
    }

    #[test]
    fn test_sub_biguint_large() {
        let p = Arc::new(BigUint::from(13u32));
        let a = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap();
        // 5 - 20 (mod 13) = 5 - 7 = -2 = 11
        let rhs = BigUint::from(20u32);
        let res = a - rhs;
        assert_eq!(res.num, BigUint::from(11u32));
    }

    #[test]
    fn test_consistency_with_biguint_ops() {
        let p_val = 31u32;
        let p = Arc::new(BigUint::from(p_val));
        let val1 = 20u32;
        let val2 = 15u32;

        let fe1 = FieldElement::new(BigUint::from(val1), p.clone()).unwrap();
        let fe2 = FieldElement::new(BigUint::from(val2), p.clone()).unwrap();

        let b1 = BigUint::from(val1);
        let b2 = BigUint::from(val2);
        let prime = BigUint::from(p_val);

        // Addition
        let fe_add = &fe1 + &fe2;
        let b_add = (&b1 + &b2) % &prime;
        assert_eq!(fe_add.num, b_add, "Addition result mismatch with BigUint");

        // Subtraction (fe1 - fe2) -> 20 - 15 = 5
        let fe_sub = &fe1 - &fe2;
        let b_sub = (&b1 + &prime - &b2) % &prime;
        assert_eq!(
            fe_sub.num, b_sub,
            "Subtraction result mismatch with BigUint"
        );

        // Subtraction wrapping (fe2 - fe1) -> 15 - 20 = -5 = 26 (mod 31)
        let fe_sub_wrap = &fe2 - &fe1;
        let b_sub_wrap = (&b2 + &prime - &b1) % &prime;
        assert_eq!(
            fe_sub_wrap.num, b_sub_wrap,
            "Subtraction wrapping result mismatch with BigUint"
        );

        // Multiplication
        let fe_mul = &fe1 * &fe2;
        let b_mul = (&b1 * &b2) % &prime;
        assert_eq!(
            fe_mul.num, b_mul,
            "Multiplication result mismatch with BigUint"
        );

        // Division
        let fe_div = &fe1 / &fe2;
        let p_minus_2 = &prime - BigUint::from(2u32);
        let b_div = (&b1 * b2.modpow(&p_minus_2, &prime)) % &prime;
        assert_eq!(fe_div.num, b_div, "Division result mismatch with BigUint");
    }
}
