use std::ops::{Add, Div, Mul, Sub};
use std::sync::{Arc, LazyLock};

use hmac::{Hmac, Mac};
use num_bigint::{BigUint};
use sha2::Sha256;

use crate::elliptic_curve::{Point, PointError};
use crate::finite_field::{FieldElement, FieldElementError};

type HmacSha256 = Hmac<Sha256>;

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

static A: LazyLock<S256FieldElement> =
    LazyLock::new(|| S256FieldElement::new(BigUint::from(0u32)).unwrap());

static B: LazyLock<S256FieldElement> =
    LazyLock::new(|| S256FieldElement::new(BigUint::from(7u32)).unwrap());

static N: LazyLock<BigUint> = LazyLock::new(|| {
    BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
        16,
    )
    .unwrap()
});

static N_MINUS_2: LazyLock<BigUint> = LazyLock::new(|| &*N - BigUint::from(2u32));

// The x and y coordinates of the secp256k1 generator point G.
static GX: LazyLock<BigUint> = LazyLock::new(|| {
    BigUint::parse_bytes(
        b"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        16,
    )
    .unwrap()
});

static GY: LazyLock<BigUint> = LazyLock::new(|| {
    BigUint::parse_bytes(
        b"483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
        16,
    )
    .unwrap()
});

/// The generator point `G` for the secp256k1 curve.
pub static G: LazyLock<S256Point> = LazyLock::new(|| {
    let x = S256FieldElement::new(GX.clone()).unwrap();
    let y = S256FieldElement::new(GY.clone()).unwrap();
    S256Point::new(x, y).unwrap()
});

macro_rules! impl_s256_wrappers {
    ($type:ident, $trait:ident, $method:ident) => {
        impl $trait<$type> for $type {
            type Output = $type;

            fn $method(self, rhs: $type) -> $type {
                (&self).$method(&rhs)
            }
        }

        impl $trait<&$type> for $type {
            type Output = $type;

            fn $method(self, rhs: &$type) -> $type {
                (&self).$method(rhs)
            }
        }

        impl $trait<$type> for &$type {
            type Output = $type;

            fn $method(self, rhs: $type) -> $type {
                self.$method(&rhs)
            }
        }
    };
}

macro_rules! impl_s256_biguint_wrappers {
    ($type:ident, $trait:ident, $method:ident) => {
        impl $trait<BigUint> for &$type {
            type Output = $type;

            fn $method(self, rhs: BigUint) -> Self::Output {
                self.$method(&rhs)
            }
        }

        impl $trait<BigUint> for $type {
            type Output = $type;

            fn $method(self, rhs: BigUint) -> Self::Output {
                (&self).$method(&rhs)
            }
        }

        impl $trait<&BigUint> for $type {
            type Output = $type;

            fn $method(self, rhs: &BigUint) -> Self::Output {
                (&self).$method(rhs)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S256FieldElement {
    element: FieldElement,
}

impl S256FieldElement {
    pub fn new<E: Into<BigUint>>(num: E) -> Result<Self, FieldElementError> {
        let element = FieldElement::new(num.into(), S256_PRIME.clone())?;

        Ok(S256FieldElement { element })
    }

    pub fn pow<E: Into<BigUint>>(&self, exponent: E) -> S256FieldElement {
        S256FieldElement {
            element: self.element.pow(exponent),
        }
    }

    pub fn num(&self) -> &BigUint {
        self.element.num()
    }

    pub fn sqrt(&self) -> S256FieldElement {
        let pow_magnitude = (&**S256_PRIME + 1u32) / 4u32;
        S256FieldElement {
            element: self.element.pow(pow_magnitude),
        }
    }
}

impl Add for &S256FieldElement {
    type Output = S256FieldElement;

    fn add(self, rhs: Self) -> Self::Output {
        S256FieldElement {
            element: &self.element + &rhs.element,
        }
    }
}

impl_s256_wrappers!(S256FieldElement, Add, add);

impl Add<&BigUint> for &S256FieldElement {
    type Output = S256FieldElement;

    fn add(self, rhs: &BigUint) -> Self::Output {
        S256FieldElement {
            element: &self.element + rhs,
        }
    }
}

impl_s256_biguint_wrappers!(S256FieldElement, Add, add);

impl Sub for &S256FieldElement {
    type Output = S256FieldElement;

    fn sub(self, rhs: Self) -> Self::Output {
        S256FieldElement {
            element: &self.element - &rhs.element,
        }
    }
}

impl_s256_wrappers!(S256FieldElement, Sub, sub);

impl Mul for &S256FieldElement {
    type Output = S256FieldElement;

    fn mul(self, rhs: Self) -> Self::Output {
        S256FieldElement {
            element: &self.element * &rhs.element,
        }
    }
}

impl_s256_wrappers!(S256FieldElement, Mul, mul);

impl Mul<&BigUint> for &S256FieldElement {
    type Output = S256FieldElement;

    fn mul(self, rhs: &BigUint) -> Self::Output {
        S256FieldElement {
            element: &self.element * rhs,
        }
    }
}

impl_s256_biguint_wrappers!(S256FieldElement, Mul, mul);

impl Div for &S256FieldElement {
    type Output = S256FieldElement;

    fn div(self, rhs: Self) -> Self::Output {
        S256FieldElement {
            element: &self.element / &rhs.element,
        }
    }
}

impl_s256_wrappers!(S256FieldElement, Div, div);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S256Point {
    point: Point,
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

    pub fn point(&self) -> &Point {
        &self.point
    }

    pub fn x_num(&self) -> Option<&BigUint> {
        self.point.x().map(|element| element.num())
    }

    pub fn y_num(&self) -> Option<&BigUint> {
        self.point.y().map(|element| element.num())
    }

    pub fn verify(&self, z: &BigUint, sig: &Signature) -> bool {
        if sig.r < 1u32.into() || sig.r >= *N || sig.s < 1u32.into() || sig.s >= *N {
            return false;
        }

        let s_inv = sig.s.modpow(&N_MINUS_2, &N);
        let u = (z * &s_inv) % &*N;
        let v = (&sig.r * s_inv) % &*N;
        // u.G + v.P = k.G
        let total_point = &*G * u + self * v;

        total_point.x_num().is_some_and(|x| (x % &*N) == sig.r)
    }

    pub fn sec(&self, compressed: Option<bool>) -> Vec<u8> {
        if self.is_at_infinity() {
            return vec![0x00];
        }

        let x_bytes = to_32_bytes(self.x_num().unwrap());

        if compressed.unwrap_or(true) {
            let mut serialized = Vec::<u8>::with_capacity(33);

            // bit(0) checks the LSB (2^0 = 1). If 1 (true), it's ODD. If 0 (false), it's EVEN.
            let marker = if self.y_num().unwrap().bit(0) {
                0x03
            } else {
                0x02
            };

            serialized.push(marker);
            serialized.extend(x_bytes);

            return serialized;
        }

        let mut serialized = Vec::<u8>::with_capacity(65);
        serialized.push(0x04);
        serialized.extend(x_bytes);
        serialized.extend(to_32_bytes(self.y_num().unwrap()));

        serialized
    }

    pub fn parse(sec_bin: &[u8]) -> Result<S256Point, PointError> {
        if sec_bin.is_empty() {
            return Err(PointError::CannotParse);
        }

        if sec_bin[0] == 0x04 {
            if sec_bin.len() != 65 {
                return Err(PointError::CannotParse);
            }

            let x_bigint = BigUint::from_bytes_be(&sec_bin[1..33]);
            let y_bigint = BigUint::from_bytes_be(&sec_bin[33..]);

            return S256Point::new(
                S256FieldElement::new(x_bigint).map_err(|_| PointError::CannotParse)?,
                S256FieldElement::new(y_bigint).map_err(|_| PointError::CannotParse)?,
            );
        }

        let is_odd = match sec_bin[0] {
            0x03 => true,
            0x02 => false,
            _ => return Err(PointError::CannotParse),
        };

        if sec_bin.len() != 33 {
            return Err(PointError::CannotParse);
        }

        let x = S256FieldElement::new(BigUint::from_bytes_be(&sec_bin[1..]))
            .map_err(|_| PointError::CannotParse)?;
        let y_squared = x.pow(BigUint::from(3u32)) + S256FieldElement::new(7u32).unwrap();
        let y = y_squared.sqrt();

        if y.num().bit(0) == is_odd {
            S256Point::new(x, y)
        } else {
            let other_y = S256FieldElement::new(0u32).unwrap() - y;

            S256Point::new(x, other_y)
        }
    }
}

pub fn to_32_bytes(num: &BigUint) -> [u8; 32] {
    let bytes = num.to_bytes_be();
    let mut result = [0u8; 32];
    let start = 32usize.saturating_sub(bytes.len());
    result[start..].copy_from_slice(&bytes);

    result
}

impl Add for &S256Point {
    type Output = S256Point;

    fn add(self, rhs: Self) -> Self::Output {
        S256Point {
            point: &self.point + &rhs.point,
        }
    }
}

impl_s256_wrappers!(S256Point, Add, add);

impl Mul<&BigUint> for &S256Point {
    type Output = S256Point;

    fn mul(self, rhs: &BigUint) -> Self::Output {
        // The order of the generator point G is N.
        // So, k * G = (k mod n) * G.
        // We can reduce the scalar modulo N before multiplication for efficiency.
        let scalar = rhs % &*N;

        S256Point {
            point: &self.point * scalar,
        }
    }
}

impl_s256_biguint_wrappers!(S256Point, Mul, mul);

#[derive(Debug, Clone)]
pub struct Signature {
    pub r: BigUint,
    pub s: BigUint,
}

impl Signature {
    pub fn der(&self) -> Vec<u8> {
        let mut result = vec![0x30];
        let mut r_bytes = self.r.to_bytes_be();
        let mut s_bytes = self.s.to_bytes_be();

        if r_bytes.is_empty() || r_bytes.first().is_some_and(|&first| first >= 0x80) {
            r_bytes.insert(0, 0x00);
        }

        if s_bytes.is_empty() || s_bytes.first().is_some_and(|&first| first >= 0x80) {
            s_bytes.insert(0, 0x00);
        }

        let total_len = 1 + 1 + r_bytes.len() + 1 + 1 + s_bytes.len();

        result.push(total_len as u8);
        result.push(0x02);
        result.push(r_bytes.len() as u8);
        result.extend_from_slice(&r_bytes);
        result.push(0x02);
        result.push(s_bytes.len() as u8);
        result.extend_from_slice(&s_bytes);

        result
    }
    
    pub fn parse_der(der: &[u8]) -> Result<Self, &'static str> {
        let mut cursor = 0;
        
        if der.len() < 2 { return Err("Too short"); }
        if der[cursor] != 0x30 { return Err("Invalid format"); }
        cursor += 1;

        let len = der[cursor] as usize;
        cursor += 1;
        if len + 2 != der.len() { return Err("Invalid length"); }

        if cursor >= der.len() || der[cursor] != 0x02 { return Err("Invalid r marker"); }
        cursor += 1;

        if cursor >= der.len() { return Err("Invalid r length byte"); }
        let r_len = der[cursor] as usize;
        cursor += 1;

        if cursor + r_len > der.len() { return Err("Invalid r bytes"); }
        let r = BigUint::from_bytes_be(&der[cursor..cursor + r_len]);
        cursor += r_len;

        if cursor >= der.len() || der[cursor] != 0x02 { return Err("Invalid s marker"); }
        cursor += 1;

        if cursor >= der.len() { return Err("Invalid s length byte"); }
        let s_len = der[cursor] as usize;
        cursor += 1;

        if cursor + s_len > der.len() { return Err("Invalid s bytes"); }
        let s = BigUint::from_bytes_be(&der[cursor..cursor + s_len]);

        Ok(Signature { r, s })
    }
}

#[derive(Debug, Clone)]
pub struct PrivateKey {
    secret: BigUint,
    point: S256Point,
}

impl PrivateKey {
    pub fn new(secret: BigUint) -> Self {
        let point = &*G * &secret;
        PrivateKey { secret, point }
    }

    pub fn sign(&self, z: &BigUint) -> Signature {
        loop {
            let k = self.deterministic_k(z);
            let r_point = &*G * &k;
            let r = r_point.x_num().unwrap() % &*N; // k is always in range [1, N) so we can safely unwrap

            // If r is zero, the signature is invalid because it doesn't bind the key.
            // This is astronomically rare, but we must handle it by generating a new k.
            if r == BigUint::from(0u32) {
                continue;
            }

            let k_inv = k.modpow(&N_MINUS_2, &N);
            // s = (z + r * secret) / k
            let mut s = ((&r * &self.secret + z) * k_inv) % &*N;

            // If s is zero, the signature is invalid (cannot compute inverse for verification).
            // We must generate a new k and retry.
            if s == BigUint::from(0u32) {
                continue;
            }

            // BIP 62: Low S values.
            // ECDSA signatures are malleable: if (r, s) is valid, so is (r, N - s).
            // To prevent transaction malleability, Bitcoin requires s to be in the lower half of the group order.
            if s > &*N / BigUint::from(2u32) {
                s = &*N - s;
            }

            return Signature { s, r };
        }
    }

    pub fn point(&self) -> &S256Point {
        &self.point
    }

    fn deterministic_k(&self, z: &BigUint) -> BigUint {
        let mut k = [0u8; 32];
        let mut v = [1u8; 32];

        let z_num = match z > &*N {
            true => z - &*N,
            false => z.clone(),
        };

        let z_bytes = to_32_bytes(&z_num);
        let secret_bytes = to_32_bytes(&self.secret);

        let mut mac = HmacSha256::new_from_slice(&k).unwrap();
        mac.update(&v);
        mac.update(&[0x00]);
        mac.update(&secret_bytes);
        mac.update(&z_bytes);
        k = mac.finalize().into_bytes().into();

        mac = HmacSha256::new_from_slice(&k).unwrap();
        mac.update(&v);
        v = mac.finalize().into_bytes().into();

        mac = HmacSha256::new_from_slice(&k).unwrap();
        mac.update(&v);
        mac.update(&[0x01]);
        mac.update(&secret_bytes);
        mac.update(&z_bytes);
        k = mac.finalize().into_bytes().into();

        mac = HmacSha256::new_from_slice(&k).unwrap();
        mac.update(&v);
        v = mac.finalize().into_bytes().into();

        loop {
            mac = HmacSha256::new_from_slice(&k).unwrap();
            mac.update(&v);
            v = mac.finalize().into_bytes().into();

            let candidate = BigUint::from_bytes_be(&v);

            if candidate >= BigUint::from(1u32) && candidate < *N {
                return candidate;
            }

            mac = HmacSha256::new_from_slice(&k).unwrap();
            mac.update(&v);
            mac.update(&[0x00]);
            k = mac.finalize().into_bytes().into();

            mac = HmacSha256::new_from_slice(&k).unwrap();
            mac.update(&v);
            v = mac.finalize().into_bytes().into();
        }
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
        let expected_x =
            S256FieldElement::new(BigUint::parse_bytes(x2_hex.as_bytes(), 16).unwrap()).unwrap();
        let expected_y =
            S256FieldElement::new(BigUint::parse_bytes(y2_hex.as_bytes(), 16).unwrap()).unwrap();
        let expected_g2 = S256Point::new(expected_x, expected_y).unwrap();

        assert_eq!(g2, expected_g2);
    }

    #[test]
    fn test_verify_signature_known_values() {
        // Public Key P
        let px_hex = "887387e452b8eacc4acfde10d9aaf7f6d9a0f975aabb10d006e4da568744d06c";
        let py_hex = "61de6d95231cd89026e286df3b6ae4a894a3378e393e93a0f45b666329a0ae34";
        let px = BigUint::parse_bytes(px_hex.as_bytes(), 16).unwrap();
        let py = BigUint::parse_bytes(py_hex.as_bytes(), 16).unwrap();
        let p_x_el = S256FieldElement::new(px).unwrap();
        let p_y_el = S256FieldElement::new(py).unwrap();
        let public_key = S256Point::new(p_x_el, p_y_el).unwrap();

        // Signature 1
        let z1_hex = "ec208baa0fc1c19f708a9ca96fdeff3ac3f230bb4a7ba4aede4942ad003c0f60";
        let r1_hex = "ac8d1c87e51d0d441be8b3dd5b05c8795b48875dffe00b7ffcfac23010d3a395";
        let s1_hex = "68342ceff8935ededd102dd876ffd6ba72d6a427a3edb13d26eb0781cb423c4";
        let z1 = BigUint::parse_bytes(z1_hex.as_bytes(), 16).unwrap();
        let r1 = BigUint::parse_bytes(r1_hex.as_bytes(), 16).unwrap();
        let s1 = BigUint::parse_bytes(s1_hex.as_bytes(), 16).unwrap();
        let sig1 = Signature { r: r1, s: s1 };
        assert!(public_key.verify(&z1, &sig1), "Signature 1 should be valid");

        // Signature 2
        let z2_hex = "7c076ff316692a3d7eb3c3bb0f8b1488cf72e1afcd929e29307032997a838a3d";
        let r2_hex = "eff69ef2b1bd93a66ed5219add4fb51e11a840f404876325a1e8ffe0529a2c";
        let s2_hex = "c7207fee197d27c618aea621406f6bf5ef6fca38681d82b2f06fddbdce6feab6";
        let z2 = BigUint::parse_bytes(z2_hex.as_bytes(), 16).unwrap();
        let r2 = BigUint::parse_bytes(r2_hex.as_bytes(), 16).unwrap();
        let s2 = BigUint::parse_bytes(s2_hex.as_bytes(), 16).unwrap();
        let sig2 = Signature { r: r2, s: s2 };
        assert!(public_key.verify(&z2, &sig2), "Signature 2 should be valid");
    }

    #[test]
    fn test_sec_serialization() {
        let g = G.clone();

        // Test Uncompressed
        let serialized_uncompressed = g.sec(Some(false));
        assert_eq!(serialized_uncompressed.len(), 65);
        assert_eq!(serialized_uncompressed[0], 0x04);

        let parsed_uncompressed = S256Point::parse(&serialized_uncompressed).unwrap();
        assert_eq!(parsed_uncompressed, g);

        // Test Compressed
        let serialized_compressed = g.sec(Some(true));
        assert_eq!(serialized_compressed.len(), 33);
        let marker = serialized_compressed[0];
        assert!(marker == 0x02 || marker == 0x03);

        // Check marker correctness
        if g.y_num().unwrap().bit(0) {
            // odd
            assert_eq!(marker, 0x03);
        } else {
            assert_eq!(marker, 0x02);
        }

        let parsed_compressed = S256Point::parse(&serialized_compressed).unwrap();
        assert_eq!(parsed_compressed, g);
    }

    #[test]
    fn test_sec_serialization_known_key() {
        // Public Key P
        let px_hex = "887387e452b8eacc4acfde10d9aaf7f6d9a0f975aabb10d006e4da568744d06c";
        let py_hex = "61de6d95231cd89026e286df3b6ae4a894a3378e393e93a0f45b666329a0ae34";
        let px = BigUint::parse_bytes(px_hex.as_bytes(), 16).unwrap();
        let py = BigUint::parse_bytes(py_hex.as_bytes(), 16).unwrap();
        let p_x_el = S256FieldElement::new(px.clone()).unwrap();
        let p_y_el = S256FieldElement::new(py.clone()).unwrap();
        let public_key = S256Point::new(p_x_el, p_y_el).unwrap();

        let px_bytes = to_32_bytes(&px);
        let py_bytes = to_32_bytes(&py);

        // Compressed
        let mut expected_compressed = vec![0x02]; // py ends in 4 (even) => 0x02
        expected_compressed.extend(&px_bytes);
        assert_eq!(public_key.sec(Some(true)), expected_compressed);

        // Uncompressed
        let mut expected_uncompressed = vec![0x04];
        expected_uncompressed.extend(&px_bytes);
        expected_uncompressed.extend(&py_bytes);
        assert_eq!(public_key.sec(Some(false)), expected_uncompressed);

        // Round trip
        assert_eq!(S256Point::parse(&expected_compressed).unwrap(), public_key);
        assert_eq!(
            S256Point::parse(&expected_uncompressed).unwrap(),
            public_key
        );
    }
}

#[cfg(test)]
mod private_key_tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_sign() {
        let secret = BigUint::parse_bytes(
            b"388c6cf54328636419a5f4792fc595c159f0197183a1c28154a688416d172905",
            16,
        )
        .unwrap();
        let pk = PrivateKey::new(secret);

        let z = BigUint::parse_bytes(
            b"bc62d4b80d9e362aa2955239e8508d7ca20c44997091cf1b373688832a87f7d1",
            16,
        )
        .unwrap();

        let sig = pk.sign(&z);

        assert!(pk.point().verify(&z, &sig));
    }

    #[test]
    fn test_sign_deterministic() {
        let secret = BigUint::from(12345u32);
        let pk = PrivateKey::new(secret);
        let z = BigUint::from(99999u32);

        let sig1 = pk.sign(&z);
        let sig2 = pk.sign(&z);

        // Signatures should be identical because k is deterministic (RFC 6979)
        assert_eq!(sig1.r, sig2.r);
        assert_eq!(sig1.s, sig2.s);

        assert!(pk.point().verify(&z, &sig1));
        assert!(pk.point().verify(&z, &sig2));
    }

    #[test]
    fn test_sign_low_s() {
        let secret = BigUint::from(12345u32);
        let pk = PrivateKey::new(secret);
        let z = BigUint::from(99999u32);
        let n_div_2 = &*N / BigUint::from(2u32);

        for _ in 0..20 {
            let sig = pk.sign(&z);
            assert!(sig.s <= n_div_2, "s value should be <= N/2");
            assert!(pk.point().verify(&z, &sig));
        }
    }
}

#[cfg(test)]
mod signature_tests {
    use super::*;
    use num_bigint::BigUint;

    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn test_der_serialization_example() {
        let r_hex = "37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6";
        let s_hex = "8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec";

        let r = BigUint::parse_bytes(r_hex.as_bytes(), 16).unwrap();
        let s = BigUint::parse_bytes(s_hex.as_bytes(), 16).unwrap();

        let sig = Signature { r, s };
        let der = sig.der();

        let expected_hex = "3045022037206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c60221008ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec";
        let expected_bytes = hex_to_bytes(expected_hex);

        assert_eq!(der, expected_bytes);
    }

    #[test]
    fn test_der_serialization_padding() {
        // Case 1: Both need padding (>= 0x80)
        let r = BigUint::from(0x80u32); // 128
        let s = BigUint::from(0x81u32); // 129
        let sig = Signature { r, s };
        let der = sig.der();

        // 0x30 [len] 0x02 [len_r] [00 80] 0x02 [len_s] [00 81]
        // len_r = 2 (00 80)
        // len_s = 2 (00 81)
        // total = 2 + 2 + 2 + 2 = 8
        // 30 08 02 02 00 80 02 02 00 81
        assert_eq!(der, hex_to_bytes("30080202008002020081"));

        // Case 2: Neither needs padding (< 0x80)
        let r = BigUint::from(0x7Fu32); // 127
        let s = BigUint::from(0x01u32); // 1
        let sig = Signature { r, s };
        let der = sig.der();

        // 0x30 [len] 0x02 [len_r] [7F] 0x02 [len_s] [01]
        // len_r = 1 (7F)
        // len_s = 1 (01)
        // total = 2 + 1 + 2 + 1 = 6
        // 30 06 02 01 7F 02 01 01
        assert_eq!(der, hex_to_bytes("300602017F020101"));
    }

    #[test]
    fn test_der_serialization_zero() {
        // Robustness check for r=0, s=0
        let r = BigUint::from(0u32);
        let s = BigUint::from(0u32);
        let sig = Signature { r, s };
        let der = sig.der();

        // 0x30 [len] 0x02 [len_r] [00] 0x02 [len_s] [00]
        // len_r = 1 (00)
        // len_s = 1 (00)
        // total = 2 + 1 + 2 + 1 = 6
        // 30 06 02 01 00 02 01 00
        assert_eq!(der, hex_to_bytes("3006020100020100"));
    }

    #[test]
    fn test_parse_der_valid() {
        let r_hex = "37206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c6";
        let s_hex = "8ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec";
        let r = BigUint::parse_bytes(r_hex.as_bytes(), 16).unwrap();
        let s = BigUint::parse_bytes(s_hex.as_bytes(), 16).unwrap();
        
        // This is a valid DER signature (from previous test)
        let der_hex = "3045022037206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c60221008ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec";
        let der_bytes = hex_to_bytes(der_hex);

        let sig = Signature::parse_der(&der_bytes).expect("Should parse valid DER");
        assert_eq!(sig.r, r);
        assert_eq!(sig.s, s);
    }

    #[test]
    fn test_parse_der_roundtrip() {
        let secret = BigUint::from(12345u32);
        let pk = PrivateKey::new(secret);
        let z = BigUint::from(99999u32);
        
        let sig = pk.sign(&z);
        let serialized = sig.der();
        let parsed = Signature::parse_der(&serialized).expect("Should parse generated signature");
        
        assert_eq!(sig.r, parsed.r);
        assert_eq!(sig.s, parsed.s);
    }

    #[test]
    fn test_parse_der_invalid_format() {
        // Empty
        assert!(Signature::parse_der(&[]).is_err());
        
        // Wrong marker (starts with 0x31 instead of 0x30)
        let der_hex = "3145022037206a0610995c58074999cb9767b87af4c4978db68c06e8e6e81d282047a7c60221008ca63759c1157ebeaec0d03cecca119fc9a75bf8e6d0fa65c841c8e2738cdaec";
        assert!(Signature::parse_der(&hex_to_bytes(der_hex)).is_err());
        
        // Too short
        assert!(Signature::parse_der(&[0x30, 0x00]).is_err());
    }
}
