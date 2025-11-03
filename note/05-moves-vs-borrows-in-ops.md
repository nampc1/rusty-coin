# 5. Moves vs. Borrows in Operator Implementations

**Context:** When implementing arithmetic operators in a loop, like `result = result + &current`, a subtle question of ownership arises. Should we write `result = &result + &current` or `result = result + &current`?

**Decision:** We will consistently prefer the borrow-based version (`&result + &current`) inside hot loops.

```rust
// Inside scalar multiplication loop
while ... {
    // ...
    // PREFERRED: Borrows `result` and `current`
    result = &result + &current;

    // AVOIDED: Moves `result`, borrows `current`
    // result = result + &current;
}
```

## The Technical Difference

The choice comes down to which `impl Add` Rust selects:

-   `result = &result + &current;` uses `impl Add<&Point> for &Point`. This implementation takes both `self` and `rhs` by reference (`&Point`). It only passes pointers into the `add` function.

-   `result = result + &current;` uses `impl Add<&Point> for Point`. This implementation takes `self` by value (`Point`), which means the `result` variable is **moved** into the `add` function.

## Is Moving Costly?

A "move" for a type like our `Point` struct is a shallow, byte-for-byte copy of its data on the stack. Our `Point` contains `FieldElement`s, which in turn contain `BigUint`s. A `BigUint` is a struct on the stack holding a pointer and length/capacity for its actual number data on the heap.

When we move `Point`, we are **not** copying the large number data on the heap. We are only performing a `memcpy` of the stack data (a few pointers and `usize`s). For this struct, this `memcpy` is not "very costly" in absolute terms.

### When is Moving Actually Costly? (The Red Flag)

Moving becomes a significant performance cost when a struct contains a large amount of data *directly on the stack*. This is common with large arrays.

Consider this hypothetical struct:

```rust
// A struct with a large array directly inside it.
// This entire array lives on the stack.
struct LargeStackStruct {
    data: [u8; 1_048_576], // One megabyte of data!
}

fn process_data() {
    let s1 = LargeStackStruct { data: [0; 1_048_576] };

    // This move is VERY EXPENSIVE!
    let s2 = s1; // Rust must perform a `memcpy` of the entire 1MB of data.
}
```

In this case, the move `let s2 = s1;` is not just copying a few pointers. It is a command to copy the **entire one-megabyte array** from one stack location to another. This is a major performance red flag.

This contrasts sharply with our `Point` struct, where the bulk of the data lives on the heap and only pointers are copied during a move.

## Rationale: Why Borrowing is Still Preferred

Despite the low cost of the move, borrowing is superior for several key reasons:

1.  **It's Fundamentally More Efficient:** Passing a single pointer (`&Point`) is cheaper than performing a `memcpy` of the entire stack portion of the `Point` struct. In a loop that runs 256 times for a typical key, these micro-optimizations are worthwhile.

2.  **It's More Idiomatic:** The borrow-based approach (`&result + &current`) more clearly communicates the intent: "Calculate a new value based on these inputs without consuming them." It's a cleaner data flow and the standard pattern in performance-sensitive Rust.

3.  **It Scales Better:** If the `Point` struct were to grow larger on the stack, the cost of the move would increase, while the cost of the borrow would remain constant (a single pointer). Adhering to this pattern makes our code robust to future changes.

By choosing to borrow, we write code that is more performant, idiomatic, and maintainable.