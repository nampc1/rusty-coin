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

    pub fn new_at_infinity(a: FieldElement, b: FieldElement) -> Result<Point, PointError> {
        Ok(Point {
            x: None,
            y: None,
            a,
            b,
        })
    }

    pub fn is_at_infinity(&self) -> bool {
        self.x == None && self.y == None
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.a != rhs.a || self.b != rhs.b {
            panic!("Cannot add two points on different curves")
        }

        // Case 1: point + infinity = point
        if self.is_at_infinity() {
            return rhs;
        }

        if rhs.is_at_infinity() {
            return self;
        }

        // now, neither point is at infinity

        if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (self.x, self.y, rhs.x, rhs.y) {
            // Case 2: self.x == other.x and self.y == -other.y
            // Self is the inverse of other, so the result is infinity
            // This check also prevents division by zero in the chord method
            let zero_element = FieldElement::new(BigUint::from(0u32), y1.get_prime()).unwrap();
            if x1 == x2 && (&y1 + &y2 == zero_element) {
                return Point::new_at_infinity(self.a.clone(), self.b.clone()).unwrap();
            }

            // Case 3: self == other (point doubling)
            if x1 == x2 && y1 == y2 && self.a == rhs.a && self.b == rhs.b {
                // The tangent line is vertical
                if y1 == zero_element {
                    return Point::new_at_infinity(self.a.clone(), self.b.clone()).unwrap();
                }

                // The tangent line intersects the curve at point -2P
                let slope = (x1.pow(2u32)
                    * FieldElement::new(BigUint::from(3u32), y1.get_prime()).unwrap()
                    + &self.a)
                    / (&y1 * FieldElement::new(BigUint::from(3u32), y1.get_prime()).unwrap());
                let x = slope.pow(2u32)
                    - &x1 * FieldElement::new(BigUint::from(2u32), y1.get_prime()).unwrap();
                let y = (slope * (x1 - &x)) - y1;

                return Point {
                    x: Some(x),
                    y: Some(y),
                    a: self.a.clone(),
                    b: self.b.clone(),
                };
            }

            // Case 4: self.x != other.x (chord method)
            let slope = (&y1 - &y2) / (&x1 - &x2);
            let x = slope.pow(2u32) - &x1 - &x2;
            let y = slope * (x1 - &x) - y1;

            return Point {
                x: Some(x),
                y: Some(y),
                a: self.a.clone(),
                b: self.b.clone(),
            };
        } else {
            unreachable!()
        }
    }
}
