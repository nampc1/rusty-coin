use std::ops::{Add, Sub, Mul, Div};
use std::sync::{Arc, LazyLock};

use num_bigint::BigUint;

use crate::elliptic_curve::Point;
use crate::finite_field::{FieldElement, FieldElementError};


// The prime modulus of the secp256k1 field, P = 2^256 - 2^32 - 977.
//
// This value cannot be a `const` because `BigUint` operations involve heap
// allocations and are not `const fn`. Instead, we use `std::sync::OnceLock`
// for thread-safe, lazy, one-time initialization. The value is computed and
// stored the first time it's accessed. See `note/06-lazy-static-initialization.md`.
static S256_PRIME: LazyLock<Arc<BigUint>> = LazyLock::new(|| {
    let two_bigint = BigUint::from(2u32);

    Arc::new(two_bigint.pow(256) - two_bigint.pow(32) - BigUint::from(977u32))
  });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S256FieldElement {
  element: FieldElement
}

impl S256FieldElement {
  pub fn new(num: BigUint) -> Result<Self, FieldElementError> {
    let element = FieldElement::new(num, S256_PRIME.clone())?;

    Ok(S256FieldElement {
      element
    })
  }

  pub fn pow<E: Into<BigUint>>(&self, exponent: E) -> S256FieldElement {
    S256FieldElement { element: self.element.pow(exponent) }
  }
}

impl Add for &S256FieldElement {
  type Output = S256FieldElement;

  fn add(self, rhs: Self) -> Self::Output {
     S256FieldElement {
      element: &self.element + &rhs.element
     } 
  }
}

impl Add for S256FieldElement {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    &self + &rhs
  }
}

impl Add<&S256FieldElement> for S256FieldElement {
  type Output = S256FieldElement;

  fn add(self, rhs: &S256FieldElement) -> Self::Output {
      &self + rhs
  }
}

impl Add<S256FieldElement> for &S256FieldElement {
  type Output = S256FieldElement;

  fn add(self, rhs: S256FieldElement) -> Self::Output {
      self + &rhs
  }
}

impl Sub for &S256FieldElement {
  type Output = S256FieldElement;

  fn sub(self, rhs: Self) -> Self::Output {
      S256FieldElement {
        element: &self.element - &rhs.element
      }
  }
}

impl Sub for S256FieldElement {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
      &self - &rhs
  }
}

impl Sub<&S256FieldElement> for S256FieldElement {
  type Output = S256FieldElement;

  fn sub(self, rhs: &S256FieldElement) -> Self::Output {
      &self - rhs
  }
}

impl Sub<S256FieldElement> for &S256FieldElement {
  type Output = S256FieldElement;

  fn sub(self, rhs: S256FieldElement) -> Self::Output {
      self - &rhs
  }
}

impl Mul for &S256FieldElement {
  type Output = S256FieldElement;

  fn mul(self, rhs: Self) -> Self::Output {
      S256FieldElement {
        element: &self.element * &rhs.element
      }
  }
}

impl Mul for S256FieldElement {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
      &self * &rhs
  }
}

impl Mul<&S256FieldElement> for S256FieldElement {
  type Output = S256FieldElement;

  fn mul(self, rhs: &S256FieldElement) -> Self::Output {
      &self * rhs
  }
}

impl Mul<S256FieldElement> for &S256FieldElement {
  type Output = S256FieldElement;

  fn mul(self, rhs: S256FieldElement) -> Self::Output {
      self * &rhs
  }
}

impl Mul<u32> for &S256FieldElement {
    type Output = S256FieldElement;

    fn mul(self, rhs: u32) -> Self::Output {
      S256FieldElement {
        element: &self.element * rhs
      }
    }
}

impl Mul<u32> for S256FieldElement {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        &self * rhs
    }
}

impl Div for &S256FieldElement {
  type Output = S256FieldElement;

  fn div(self, rhs: Self) -> Self::Output {
      S256FieldElement {
        element: &self.element / &rhs.element
      }
  }
}

impl Div for S256FieldElement {
  type Output = Self;

  fn div(self, rhs: Self) -> Self::Output {
      &self / &rhs
  }
}

impl Div<&S256FieldElement> for S256FieldElement {
  type Output = S256FieldElement;

  fn div(self, rhs: &S256FieldElement) -> Self::Output {
      &self / rhs
  }
}

impl Div<S256FieldElement> for &S256FieldElement {
  type Output = S256FieldElement;

  fn div(self, rhs: S256FieldElement) -> Self::Output {
      self / &rhs
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S256Point {
  point: Point,
  a: S256FieldElement,
  b: S256FieldElement,
  n: BigUint
}

#[cfg(test)]
mod s256_field_element_tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_new() {
        // Valid case
        let el1 = S256FieldElement::new(BigUint::from(12345u32));
        assert!(el1.is_ok());

        // Invalid case: num >= prime
        let num2 = (**S256_PRIME).clone();
        let el2 = S256FieldElement::new(num2);
        assert!(el2.is_err());
        match el2 {
            Err(FieldElementError::InvalidNum(_, _)) => (), // Expected
            _ => panic!("Expected InvalidNum error"),
        }
    }

    #[test]
    fn test_add_sub() {
        let a = S256FieldElement::new(BigUint::from(100u32)).unwrap();
        let b = S256FieldElement::new(BigUint::from(200u32)).unwrap();
        let c = S256FieldElement::new(BigUint::from(300u32)).unwrap();

        assert_eq!(a.clone() + b.clone(), c);
        assert_eq!(c - b, a);
    }

    #[test]
    fn test_add_sub_wrap_around() {
        // p - 1
        let p_minus_1 = S256FieldElement::new(&**S256_PRIME - BigUint::from(1u32)).unwrap();
        // 2
        let two = S256FieldElement::new(BigUint::from(2u32)).unwrap();
        // (p - 1) + 2 = p + 1 = 1 (mod p)
        let one = S256FieldElement::new(BigUint::from(1u32)).unwrap();

        assert_eq!(p_minus_1.clone() + two.clone(), one);
        assert_eq!(one - two, p_minus_1);
    }

    #[test]
    fn test_mul_div() {
        let a = S256FieldElement::new(BigUint::from(10u32)).unwrap();
        let b = S256FieldElement::new(BigUint::from(20u32)).unwrap();
        let c = S256FieldElement::new(BigUint::from(200u32)).unwrap();

        assert_eq!(a.clone() * b.clone(), c);
        assert_eq!(c / b, a);
    }

    #[test]
    fn test_pow_and_fermats_little_theorem() {
        // a^(p-1) === 1 (mod p)
        let p_minus_1 = &**S256_PRIME - BigUint::from(1u32);
        let a = S256FieldElement::new(BigUint::from(999u32)).unwrap();
        let one = S256FieldElement::new(BigUint::from(1u32)).unwrap();

        assert_eq!(a.pow(p_minus_1), one);
    }
}
