# 7. Architectural Refactor: Borrowing vs. Owning Shared Data

**Context:** An earlier design decision led to a significant, but necessary, architectural refactoring. This note documents the problem and the solution to guide future design choices.

## Problem: The High Cost of Owning Shared Data

The initial implementation of `FieldElement` owned its prime number:

```rust
// Initial design
pub struct FieldElement {
    num: BigUint,
    prime: BigUint, // Owns the prime
}
```

This simple design had a severe performance drawback. `BigUint` is a heap-allocated type. When creating many elements in the same field, each `FieldElement` required its own deep copy of the prime number, usually via `prime.clone()`.

```rust
// The performance trap
let p = BigUint::from(101u32);
let e1 = FieldElement::new(BigUint::from(5u32), p.clone()).unwrap(); // Allocates & copies P
let e2 = FieldElement::new(BigUint::from(10u32), p.clone()).unwrap(); // Allocates & copies P again!
```

This led to excessive memory usage and CPU overhead from repeated allocations and copies of identical data.

## Solution: Use Lifetimes to Borrow Shared Data

The solution was to refactor `FieldElement` to be generic over a lifetime and hold a *reference* to the prime number.

```rust
// Refactored design
pub struct FieldElement<'a> {
    num: BigUint,
    prime: &'a BigUint, // Borrows the prime
}
```

This change allows many `FieldElement` instances to share a single instance of the prime number, storing only a lightweight reference (a pointer). This is vastly more memory-efficient and faster.

## Design Heuristic: "Configuration" vs. "Instance" Data

This refactoring teaches a valuable design lesson:

- **Instance Data:** Data unique to each object (like the `num` in `FieldElement`). The object should own this.
- **Configuration Data:** Data that defines the context or environment for many objects (like the `prime` of a field). This data should be **borrowed**, not owned.

When designing a struct, always ask: "Does this struct own this data, or is it just using it as part of a shared context?" If the data is shared, prefer borrowing (`&T`) over owning (`T`). This often involves adding lifetime parameters, which is the correct and idiomatic way to model this relationship in Rust.

## A Note on Lifetime Syntax

This refactoring introduces a generic lifetime parameter to `FieldElement`. It's crucial to use the correct syntax when referring to this new type.

The full name of the type is now `FieldElement<'a>`. The syntax `FieldElement` by itself is incomplete, much like `Vec` is incomplete without `Vec<T>`.

### Correct vs. Insufficient Syntax

Consider a function argument that is a reference to a `FieldElement`.

```rust
// INSUFFICIENT: This will not compile.
fn my_func(element: &'a FieldElement) { ... }
```

The syntax `&'a FieldElement` is insufficient because it only describes the lifetime of the *outer reference*, but it fails to specify the lifetime required by the `FieldElement` type itself.

```rust
// CORRECT: This provides all necessary information.
fn my_func(element: &'a FieldElement<'a>) { ... }
```

The correct syntax, `&'a FieldElement<'a>`, specifies both the lifetime of the reference and the lifetime parameter of the generic type it points to. This explicitness is required for the compiler to prove the code is safe.