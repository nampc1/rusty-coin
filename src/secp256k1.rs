use std::ops::{Add, Sub, Mul, Div};
use std::sync::{Arc, LazyLock};

use num_bigint::BigUint;

use crate::elliptic_curve::{Point, PointError};
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

static A: LazyLock<S256FieldElement> = LazyLock::new(|| {
  S256FieldElement::new(BigUint::from(0u32)).unwrap()
});

static B: LazyLock<S256FieldElement> = LazyLock::new(|| {
  S256FieldElement::new(BigUint::from(7u32)).unwrap()
});

static N: LazyLock<BigUint> = LazyLock::new(|| {
  BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap()
});

// The x and y coordinates of the secp256k1 generator point G.
static GX: LazyLock<BigUint> = LazyLock::new(|| {
    BigUint::parse_bytes(b"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798", 16).unwrap()
});

static GY: LazyLock<BigUint> = LazyLock::new(|| {
    BigUint::parse_bytes(b"483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8", 16).unwrap()
});

/// The generator point `G` for the secp256k1 curve.
pub static G: LazyLock<S256Point> = LazyLock::new(|| {
    let x = S256FieldElement::new(GX.clone()).unwrap();
    let y = S256FieldElement::new(GY.clone()).unwrap();
    S256Point::new(x, y).unwrap()
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
  point: Point
}

impl S256Point {
  pub fn new(x: S256FieldElement, y: S256FieldElement) -> Result<S256Point, PointError> {
    let point = Point::new(x.element, y.element, A.element.clone(), B.element.clone())?;

    Ok(S256Point { point })
  }

  pub fn infinity() -> Result<S256Point, PointError> {
    let point = Point::infinity(A.element.clone(), B.element.clone())?;

    Ok(S256Point { point })
  }

  pub fn is_at_infinity(&self) -> bool {
    self.point.is_at_infinity()
  }
}

impl Add for &S256Point {
  type Output = S256Point;

  fn add(self, rhs: Self) -> Self::Output {
      S256Point {
        point: &self.point + &rhs.point
      }
  }
}

impl Add for S256Point {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
      &self + &rhs
  }
}

impl Add<&S256Point> for S256Point {
  type Output = Self;

  fn add(self, rhs: &S256Point) -> Self::Output {
      &self + rhs
  }
}

impl Add<S256Point> for &S256Point {
  type Output = S256Point;

  fn add(self, rhs: S256Point) -> Self::Output {
      self + &rhs
  }
}

impl Mul<BigUint> for &S256Point {
    type Output = S256Point;

    fn mul(self, rhs: BigUint) -> Self::Output {
        // The order of the generator point G is N.
        // So, k * G = (k mod n) * G.
        // We can reduce the scalar modulo N before multiplication for efficiency.
        let scalar = rhs % &*N;
        S256Point {
            point: &self.point * scalar,
        }
    }
}

impl Mul<BigUint> for S256Point {
    type Output = S256Point;

    fn mul(self, rhs: BigUint) -> Self::Output {
        &self * rhs
    }
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

#[cfg(test)]
mod s256_point_tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_point_creation_and_on_curve() {
        // G is initialized via LazyLock. Cloning it will trigger its creation
        // and panic if the point is not on the curve. This serves as a test.
        let _g = G.clone();

        // Test a point that is known to not be on the curve.
        // For y^2 = x^3 + 7, let x=1, y=3. Then y^2=9, x^3+7=8. 9 != 8.
        let x = S256FieldElement::new(BigUint::from(1u32)).unwrap();
        let y = S256FieldElement::new(BigUint::from(3u32)).unwrap();
        let p = S256Point::new(x, y);
        assert!(p.is_err());
        match p {
            Err(PointError::NotOnCurve) => (), // Expected error
            _ => panic!("Expected NotOnCurve error"),
        }
    }

    #[test]
    fn test_add_infinity() {
        let g = G.clone();
        let inf = S256Point::infinity().unwrap();

        // G + Infinity = G
        assert_eq!(&g + &inf, g);
        // Infinity + G = G
        assert_eq!(&inf + &g, g);
    }

    #[test]
    fn test_add_inverse_points() {
        let g = G.clone();
        let inf = S256Point::infinity().unwrap();

        // The inverse of G=(x,y) is (x, P-y).
        let neg_gy_biguint = &**S256_PRIME - &*GY;
        let neg_y_element = S256FieldElement::new(neg_gy_biguint).unwrap();
        let x_element = S256FieldElement::new(GX.clone()).unwrap();

        let g_inverse = S256Point::new(x_element, neg_y_element).unwrap();

        // G + (-G) should be infinity
        assert_eq!(&g + &g_inverse, inf);
    }

    #[test]
    fn test_scalar_multiplication_basics() {
        let g = G.clone();
        let inf = S256Point::infinity().unwrap();

        // 0 * G = Infinity
        assert_eq!(&g * BigUint::from(0u32), inf);

        // 1 * G = G
        assert_eq!(&g * BigUint::from(1u32), g.clone());

        // 2 * G = G + G
        let g2_mul = &g * BigUint::from(2u32);
        let g2_add = &g + &g;
        assert_eq!(g2_mul, g2_add);
    }

    #[test]
    fn test_scalar_multiplication_order() {
        // n * G = Infinity, where n is the order of the group
        let g = G.clone();
        let inf = S256Point::infinity().unwrap();

        // This is a long computation, but it's a critical test.
        let result = &g * N.clone();
        assert!(result.is_at_infinity());
        assert_eq!(result, inf);
    }

    #[test]
    fn test_scalar_multiplication_known_values() {
        // Test 2*G against a known value.
        let g = G.clone();
        let g2 = &g * BigUint::from(2u32);

        let x2_hex = "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";
        let y2_hex = "1ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a";
        let expected_x = S256FieldElement::new(BigUint::parse_bytes(x2_hex.as_bytes(), 16).unwrap()).unwrap();
        let expected_y = S256FieldElement::new(BigUint::parse_bytes(y2_hex.as_bytes(), 16).unwrap()).unwrap();
        let expected_g2 = S256Point::new(expected_x, expected_y).unwrap();

        assert_eq!(g2, expected_g2);
    }
}
