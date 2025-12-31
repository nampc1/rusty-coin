use num_bigint::BigUint;
use std::convert::TryInto;

const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub fn encode(data: &[u8]) -> String {
    let mut prefix = String::new();

    for char in data.iter() {
        if *char == 0x00 {
            prefix.push('1');
        } else {
            break;
        }
    }

    let mut num = BigUint::from_bytes_be(data);
    let mut result = String::new();

    while num > BigUint::from(0u32) {
        let remainder = &num % 58u32;
        num /= 58u32;

        let index: usize = remainder.try_into().unwrap(); // remainder is alway between 0 and 58
        let next_char = BASE58_ALPHABET[index];
        result.push(next_char as char);
    }

    result = result.chars().rev().collect();

    prefix + &result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn test_encode_simple() {
        let data = b"hello world";
        assert_eq!(encode(data), "StV1DL6CwTryKyV");
    }

    #[test]
    fn test_encode_leading_zeros() {
        // 0x00 0x00 -> "11"
        assert_eq!(encode(&[0, 0]), "11");

        // 0x00 0x00 + "hello" -> "11Cn8eVZg"
        let mut data = vec![0, 0];
        data.extend_from_slice(b"hello");
        // "hello" is Cn8eVZg
        assert_eq!(encode(&data), "11Cn8eVZg");
    }

    #[test]
    fn test_encode_visual_example() {
        // [0x00, 0x00, 0x02, 0x84] -> "11C7"
        let data = vec![0x00, 0x00, 0x02, 0x84];
        assert_eq!(encode(&data), "11C7");
    }
}
