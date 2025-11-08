# 6. Lazy Static Initialization with `OnceLock`

**Context:** We need to define global, immutable constants for our cryptographic primitives, such as the prime modulus `P` for the `secp256k1` field. These values must be available throughout the program's lifetime.

## The Problem: `const` vs. Complex Initialization

The most natural way to define a constant in Rust is with `const`:

```rust
// This does NOT compile!
const P: BigUint = BigUint::from(2u32.pow(256) - 2u32.pow(32) - 977);
```

This fails because `const` values must be fully evaluated at **compile time**. However, `BigUint` operations (like `from`, `pow`, and arithmetic) require heap allocations and complex logic that can only be executed at **runtime**. Functions that perform such operations are not `const fn`, leading to a compile error.

## Decision: Use `std::sync::OnceLock` for Lazy Initialization

We will use a `static` variable of type `std::sync::OnceLock` to store the value. This provides thread-safe, lazy, one-time initialization.

The pattern involves two parts:

1.  A module-private `static` variable.
2.  A private helper function to access and initialize the value.

```rust
// In `secp256k1.rs`

use std::sync::OnceLock;
use num_bigint::BigUint;

static S256_PRIME: OnceLock<BigUint> = OnceLock::new();

fn get_s256_prime() -> &'static BigUint {
  S256_PRIME.get_or_init(|| {
    let two = BigUint::from(2u32);
    two.pow(256) - two.pow(32) - BigUint::from(977u32)
  })
}
```

The `get_or_init` method ensures the closure containing the expensive calculation is run exactly once, the first time `get_s256_prime()` is called. All subsequent calls will return a reference to the already-computed value without re-running the closure.

## Rationale

This approach is the idiomatic Rust solution for this problem:

1.  **Solves the `const` Problem:** It defers the complex calculation to runtime, avoiding compile-time evaluation constraints.
2.  **Thread-Safe:** `OnceLock` is designed to be safely used across multiple threads.
3.  **Lazy and Efficient:** The expensive calculation is only performed if and when the value is actually needed.
4.  **Standard Library:** As of Rust 1.70, `OnceLock` is part of the standard library, so it requires no external dependencies.