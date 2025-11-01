use std::ops::Add;

use num_bigint::BigUint;

use crate::finite_field::FieldElement;

#[derive(Debug)]
pub enum PointError {
    NotOnCurve,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Point {
    x: Option<FieldElement>,
    y: Option<FieldElement>,
    a: FieldElement,
    b: FieldElement,
}

impl Point {
    pub fn new(
        x: FieldElement,
        y: FieldElement,
        a: FieldElement,
        b: FieldElement,
    ) -> Result<Point, PointError> {
        if y.pow(2u32) != x.pow(3u32) + &a * &x + &b {
            return Err(PointError::NotOnCurve);
        }

        Ok(Point {
            x: Some(x),
            y: Some(y),
            a,
            b,
        })
    }

    pub fn new_at_infinity(a: BigUint, b: BigUint) -> Result<Point, PointError> {
        Ok(Point {
            x: None,
            y: None,
            a,
            b,
        })
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.a != rhs.a || self.b != rhs.b {
            panic!("Cannot add two points on different curves")
        }

        // Case 1: point + infinity = point
        if self.is_at_infinity {
            return rhs;
        }

        if rhs.is_at_infinity {
            return self;
        }

        // now, neither point is at infinity

        if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (self.x, self.y, rhs.x, rhs.y) {
            // Case 2: self.x == other.x and self.y == -other.y
            // Self is the inverse of other, so the result is infinity
            // This check also prevents division by zero in the chord method
            if x1 == x2 && (y1 + y2 == BigUint::from(0)) {}
        } else {
            unreachable!()
        }

        Point {}
    }
}
