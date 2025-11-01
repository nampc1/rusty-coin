mod elliptic_curve;
mod finite_field;

use crate::finite_field::FieldElement;
use num_bigint::BigUint;

fn main() {
    // Define a prime number for our field
    let prime = BigUint::from(19u32);

    // Create two field elements.
    // The .unwrap() is safe here because we know the inputs are valid.
    let a = FieldElement::new(BigUint::from(7u32), prime.clone()).unwrap();
    let b = FieldElement::new(BigUint::from(12u32), prime.clone()).unwrap();

    println!("Working with the finite field of order {}", prime);
    println!("a = {}", a);
    println!("b = {}", b);
    println!("---");

    // Perform some operations
    let sum = a.clone() + b.clone();
    println!("a + b = {}  (since 7 + 12 = 19, which is 0 mod 19)", sum);

    let difference = a.clone() - b.clone();
    println!(
        "a - b = {} (since 7 - 12 = -5, which is 14 mod 19)",
        difference
    );

    let product = a.clone() * b.clone();
    println!("a * b = {} (since 7 * 12 = 84, which is 8 mod 19)", product);
}
