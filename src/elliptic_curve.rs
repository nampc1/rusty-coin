use num_bigint::BigUint;

use crate::finite_field::FieldElement;
use std::ops::{Add, Mul};

#[derive(Debug)]
pub enum PointError {
    NotOnCurve,
}

impl std::fmt::Display for PointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointError::NotOnCurve => write!(f, "Point is not on the elliptic curve"),
        }
    }
}

impl std::error::Error for PointError {}

// This enum follows the "make invalid states unrepresentable" pattern.
// A point on an elliptic curve is either a finite point with both an x and a y
// coordinate, or it is the point at infinity. By using an enum, we make it
// impossible to represent an invalid state (e.g., a point with an x but no y).
// This allows the compiler to prove our `match` statements are exhaustive,
// making the code safer and more readable. See `note/04-making-states-unrepresentable.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PointKind<'a> {
    Coordinates(FieldElement<'a>, FieldElement<'a>),
    Infinity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point<'a> {
    kind: PointKind<'a>,
    a: FieldElement<'a>,
    b: FieldElement<'a>,
}

impl<'a> Point<'a> {
    pub fn new(
        x: FieldElement<'a>,
        y: FieldElement<'a>,
        a: FieldElement<'a>,
        b: FieldElement<'a>,
    ) -> Result<Point<'a>, PointError> {
        if y.pow(2u32) != x.pow(3u32) + &a * &x + &b {
            return Err(PointError::NotOnCurve);
        }

        Ok(Point {
            kind: PointKind::Coordinates(x, y),
            a,
            b,
        })
    }

    pub fn new_at_infinity(
        a: FieldElement<'a>,
        b: FieldElement<'a>,
    ) -> Result<Point<'a>, PointError> {
        Ok(Point {
            kind: PointKind::Infinity,
            a,
            b,
        })
    }

    pub fn is_at_infinity(&self) -> bool {
        matches!(self.kind, PointKind::Infinity)
    }
}

impl<'a> Add for &Point<'a> {
    type Output = Point<'a>;

    fn add(self, rhs: &Point<'a>) -> Self::Output {        
        match (&self.kind, &rhs.kind) {
            (PointKind::Infinity, _) => rhs.clone(),
            (_, PointKind::Infinity) => self.clone(),
            (PointKind::Coordinates(x1, y1), PointKind::Coordinates(x2, y2)) => {
                // Case 2: self.x == other.x and self.y == -other.y
                // Self is the inverse of other, so the result is infinity
                // This check also prevents division by zero in the chord method
                if x1 == x2 && (y1 + y2).is_zero() {
                    return Point::new_at_infinity(self.a.clone(), self.b.clone()).unwrap();
                }

                // Case 3: self == other (point doubling)
                if self == rhs {
                    // The tangent line is vertical
                    if y1.is_zero() {
                        return Point::new_at_infinity(self.a.clone(), self.b.clone()).unwrap();
                    }

                    // The tangent line intersects the curve at point -2P
                    let slope = (x1.pow(2u32) * 3u32 + &self.a) / (y1 * 2u32);
                    let x = slope.pow(2u32) - x1 * 2u32;
                    let y = &slope * (x1 - &x) - y1;

                    return Point::new(x, y, self.a.clone(), self.b.clone()).unwrap();
                }

                // Case 4: self.x != other.x (chord method)
                let slope = (y1 - y2) / (x1 - x2);
                let x = slope.pow(2u32) - x1 - x2;
                let y = &slope * (x1 - &x) - y1;

                Point::new(x, y, self.a.clone(), self.b.clone()).unwrap()
            }
        }
    }
}

impl<'a> Add for Point<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl<'a> Add<&Point<'a>> for Point<'a> {
    type Output = Self;

    fn add(self, rhs: &Point<'a>) -> Self::Output {
        &self + rhs
    }
}

impl<'a> Add<Point<'a>> for &Point<'a> {
    type Output = Point<'a>;

    fn add(self, rhs: Point<'a>) -> Self::Output {
        self + &rhs
    }
}

impl<'a> Mul<BigUint> for &Point<'a> {
    type Output = Point<'a>;

    fn mul(self, rhs: BigUint) -> Self::Output {
        let mut scalar = rhs;
        let mut result = Point::new_at_infinity(self.a.clone(), self.b.clone()).unwrap();
        let mut current = self.clone();

        while scalar > BigUint::from(0u32) {
            if (&scalar & BigUint::from(1u32)) == BigUint::from(1u32) {
                // --- Performance Note on Ownership ---
                // We use `&result + &current` instead of `result + &current`.
                // The latter would *move* `result` into the `add` function, which
                // involves a `memcpy` of its stack data. By using references for
                // both operands, we only pass pointers, which is more efficient
                // inside a loop. See `note/05-moves-vs-borrows-in-ops.md`.
                result = &result + &current;
            }

            current = &current + &current;
            scalar >>= 1;
        }

        result
    }
}

#[cfg(test)]
mod elliptic_curve_tests {
    use super::*;
    use crate::finite_field::FieldElement;
    use num_bigint::BigUint;

    #[test]
    fn test_point_creation() {
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        // Valid point (192, 105) on y^2 = x^3 + 7
        let x1 = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y1 = FieldElement::new(BigUint::from(105u32), &prime).unwrap();
        let p1 = Point::new(x1, y1, a.clone(), b.clone());
        assert!(p1.is_ok());

        // Invalid point (42, 99) on y^2 = x^3 + 7
        let x2 = FieldElement::new(BigUint::from(42u32), &prime).unwrap();
        let y2 = FieldElement::new(BigUint::from(99u32), &prime).unwrap();
        let p2 = Point::new(x2, y2, a.clone(), b.clone());
        assert!(p2.is_err());
        match p2 {
            Err(PointError::NotOnCurve) => (), // Expected error
            _ => panic!("Expected NotOnCurve error"),
        }
    }

    #[test]
    fn test_point_at_infinity() {
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        let p_inf = Point::new_at_infinity(a.clone(), b.clone()).unwrap();
        assert!(p_inf.is_at_infinity());

        let x = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y = FieldElement::new(BigUint::from(105u32), &prime).unwrap();
        let p = Point::new(x, y, a, b).unwrap();
        assert!(!p.is_at_infinity());
    }

    #[test]
    fn test_add_point_and_infinity() {
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        let x = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y = FieldElement::new(BigUint::from(105u32), &prime).unwrap();
        let p1 = Point::new(x, y, a.clone(), b.clone()).unwrap();
        let p_inf = Point::new_at_infinity(a.clone(), b.clone()).unwrap();

        // p + infinity = p
        let res1 = p1.clone() + p_inf.clone();
        assert_eq!(res1, p1);

        // infinity + p = p
        let res2 = p_inf + p1.clone();
        assert_eq!(res2, p1);
    }

    #[test]
    fn test_add_inverse_points() {
        // Test P + (-P) = Infinity
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        let x1 = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y1 = FieldElement::new(BigUint::from(105u32), &prime).unwrap();
        let p1 = Point::new(x1, y1, a.clone(), b.clone()).unwrap();

        // -y mod p = -105 mod 223 = 118
        let x2 = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y2 = FieldElement::new(BigUint::from(118u32), &prime).unwrap();
        let p2 = Point::new(x2, y2, a.clone(), b.clone()).unwrap();

        let p_inf = Point::new_at_infinity(a, b).unwrap();
        assert_eq!(p1 + p2, p_inf);
    }

    #[test]
    fn test_add_chord_method() {
        // Test P1 + P2 = P3 where x1 != x2
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        // P1 = (170, 142)
        let x1 = FieldElement::new(BigUint::from(170u32), &prime).unwrap();
        let y1 = FieldElement::new(BigUint::from(142u32), &prime).unwrap();
        let p1 = Point::new(x1, y1, a.clone(), b.clone()).unwrap();

        // P2 = (60, 139)
        let x2 = FieldElement::new(BigUint::from(60u32), &prime).unwrap();
        let y2 = FieldElement::new(BigUint::from(139u32), &prime).unwrap();
        let p2 = Point::new(x2, y2, a.clone(), b.clone()).unwrap();

        // Expected result P3 = (220, 181)
        let x3 = FieldElement::new(BigUint::from(220u32), &prime).unwrap();
        let y3 = FieldElement::new(BigUint::from(181u32), &prime).unwrap();
        let p3 = Point::new(x3, y3, a, b).unwrap();

        assert_eq!(p1 + p2, p3);
    }

    #[test]
    fn test_add_point_doubling() {
        // Test P + P = 2P
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        // P1 = (192, 105)
        let x1 = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y1 = FieldElement::new(BigUint::from(105u32), &prime).unwrap();
        let p1 = Point::new(x1, y1, a.clone(), b.clone()).unwrap();

        // Expected result 2*P1 = (49, 71)
        let x2 = FieldElement::new(BigUint::from(49u32), &prime).unwrap();
        let y2 = FieldElement::new(BigUint::from(71u32), &prime).unwrap();
        let p2 = Point::new(x2, y2, a, b).unwrap();

        assert_eq!(p1.clone() + p1, p2);
    }

    #[test]
    fn test_scalar_multiplication() {
        let prime = BigUint::from(223u32);
        let a = FieldElement::new(BigUint::from(0u32), &prime).unwrap();
        let b = FieldElement::new(BigUint::from(7u32), &prime).unwrap();

        // P = (192, 105)
        let x1 = FieldElement::new(BigUint::from(192u32), &prime).unwrap();
        let y1 = FieldElement::new(BigUint::from(105u32), &prime).unwrap();
        let p = Point::new(x1, y1, a.clone(), b.clone()).unwrap();

        let p_inf = Point::new_at_infinity(a.clone(), b.clone()).unwrap();

        // Test case 1: 0 * P = Infinity
        let res1 = &p * BigUint::from(0u32);
        assert_eq!(res1, p_inf, "0 * P should be Infinity");

        // Test case 2: 1 * P = P
        let res2 = &p * BigUint::from(1u32);
        assert_eq!(res2, p, "1 * P should be P");

        // Test case 3: 2 * P = P + P
        // Expected result 2*P = (49, 71) from `test_add_point_doubling`
        let x2 = FieldElement::new(BigUint::from(49u32), &prime).unwrap();
        let y2 = FieldElement::new(BigUint::from(71u32), &prime).unwrap();
        let p2 = Point::new(x2, y2, a.clone(), b.clone()).unwrap();

        let res3 = &p * BigUint::from(2u32);
        assert_eq!(res3, p2, "2 * P should be the same as point doubling");
    }
}
