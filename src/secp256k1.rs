use std::sync::{LazyLock};

use num_bigint::BigUint;

use crate::elliptic_curve::Point;
use crate::finite_field::{FieldElement, FieldElementError};


// The prime modulus of the secp256k1 field, P = 2^256 - 2^32 - 977.
//
// This value cannot be a `const` because `BigUint` operations involve heap
// allocations and are not `const fn`. Instead, we use `std::sync::OnceLock`
// for thread-safe, lazy, one-time initialization. The value is computed and
// stored the first time it's accessed. See `note/06-lazy-static-initialization.md`.
static S256_PRIME: LazyLock<BigUint> = LazyLock::new(|| {
    let two_bigint = BigUint::from(2u32);

    two_bigint.pow(256) - two_bigint.pow(32) - BigUint::from(977u32)
  });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S256FieldElement {
  element: FieldElement
}

impl S256FieldElement {
  fn new(num: BigUint) -> Result<Self, FieldElementError> {
    let element = FieldElement::new(num, S256_PRIME.clone()).unwrap();

    Ok(S256FieldElement {
      element
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S256Point {
  point: Point,
  a: S256FieldElement,
  b: S256FieldElement,
  n: BigUint
}
