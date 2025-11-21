# Rusty Coin

A project to learn and implement the cryptographic primitives behind Bitcoin from scratch in Rust, following the principles outlined in the book "Programming Bitcoin" by Jimmy Song.

## About This Project

This repository is a hands-on exercise in building the fundamental components of Bitcoin. The primary goal is educational: to gain a deep understanding of finite fields, elliptic curves, and digital signatures by implementing them in a modern, safe systems language.

## Features Implemented

-   **Finite Field Arithmetic:** A `FieldElement` type that performs modular arithmetic over a prime field.
-   **Elliptic Curve Arithmetic:** A `Point` type representing points on an elliptic curve, with support for:
    -   Point Addition (Chord and Tangent methods)
    -   Scalar Multiplication (using the binary expansion method)
-   **secp256k1:** Constants and types for the specific curve used by Bitcoin.

## How to Run

Clone the repository and run the main example:

```bash
git clone https://github.com/your-username/rusty-coin.git
cd rusty-coin
cargo run
```

To run the test suite:

```bash
cargo test
```

## Architectural Notes & Design Patterns

This project emphasizes clean, idiomatic Rust. The codebase serves as a practical example of several important software engineering patterns. The `note/` directory contains detailed explanations of key architectural decisions.

-   **Note 1: Operator Overloading for Custom Types**
    -   *How to implement `+`, `-`, `*`, `/` ergonomically for custom numeric types.*

-   **Note 4: Making Invalid States Unrepresentable**
    -   *Using Rust's `enum` type to enforce domain logic at compile time, making code safer and more robust.*

-   **Note 7: Borrowing vs. Owning Shared Data**
    -   *The trade-offs between owning data (`T`) and borrowing it (`&'a T`) for shared "configuration" data like a field's prime.*

-   **Note 8: Escaping "Lifetime Hell" with `Arc`**
    -   *Why we refactored from lifetimes (`&'a T`) to atomically reference-counted pointers (`Arc<T>`) to solve a complex ownership problem in elliptic curve arithmetic.*

These notes document the evolution of the project's architecture and provide a deeper insight into the design process.