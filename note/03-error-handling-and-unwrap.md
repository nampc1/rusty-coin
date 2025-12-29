# 3. Error Handling: The `unwrap()` Dilemma

**Context:** Rust's `Result` and `Option` types are central to its design for robust software. A frequent question is when it is appropriate to use methods like `.unwrap()` or `.expect()`, which will cause the program to panic on an `Err` or `None` value.

**Decision:** We follow a strict set of guidelines to distinguish between recoverable errors and unrecoverable logic errors (bugs).

## The Golden Rule

**Use `.unwrap()` or `.expect()` only when you can state with 100% certainty that the value will be present. If it's not, it signifies a bug in the program's logic that should cause a crash.**

## The Flowchart

When encountering a `Result` or `Option`, we follow this decision process:

1.  **Is this for a test, example, or a quick prototype?**
    -   **Yes:** `.unwrap()` is acceptable. A panic in a test is a failure, which is the desired outcome for an incorrect assumption.
        ```rust
        #[test]
        fn test_add_point_doubling() {
            // In a test, this is fine. If the point isn't valid, the test should fail.
            let p1 = Point::new(x1, y1, a.clone(), b.clone()).unwrap();
            let p2 = Point::new(x2, y2, a, b).unwrap();
            assert_eq!(p1.clone() + p1, p2);
        }
        ```
    -   **No:** Continue.

2.  **Is failure a logical impossibility?**
    -   **Yes:** `.unwrap()` is acceptable. This is an assertion that a condition is always met. For example, `Point::new_at_infinity()` returns a `Result` for API consistency but its implementation can never fail.
        ```rust
        // This function can't fail, so unwrap is safe and communicates that fact.
        let p_inf = Point::new_at_infinity(a, b).unwrap();
        ```
    -   **No:** Continue.

3.  **If it fails, is it a bug (an unrecoverable logic error)?**
    -   **Yes:** Use `.expect("a clear message explaining why this shouldn't fail")`. `.expect()` is strongly preferred over `.unwrap()` because the custom message makes debugging panics significantly easier.
        ```rust
        // BAD: If this panics, you just get a generic message.
        let gx = BigUint::from_str_radix(gx_hex, 16).unwrap();

        // GOOD: If this panics, you know exactly what failed and why.
        let gx = BigUint::from_str_radix(gx_hex, 16)
            .expect("Failed to parse hardcoded generator point Gx. Check hex string.");
        ```
    -   **No:** It's a recoverable error. Continue.

4.  **How should this recoverable error be handled?**
    -   **`match` / `if let`:** To handle `Ok`/`Err` or `Some`/`None` cases explicitly within the current function.
        ```rust
        // `match` is great for handling all possible outcomes.
        match Point::new(x, y, a, b) {
            Ok(point) => println!("Successfully created point: {:?}", point),
            Err(PointError::NotOnCurve) => println!("Error: The provided coordinates are not on the curve."),
        }

        // `if let` is useful when you only care about the success case.
        if let Ok(point) = Point::new(x, y, a, b) {
            // Do something with the valid point...
        }
        // If it was an error, the `if` block is just skipped.
        ```
    -   **The `?` operator:** To propagate an `Err` value up the call stack to a function that is designed to handle it.
        ```rust
        // The `?` operator is the most ergonomic way to propagate errors.
        // This function must return a `Result`.
        fn create_and_add() -> Result<Point, PointError> {
            let p1 = Point::new(x1, y1, a.clone(), b.clone())?; // If this fails, the function returns Err
            let p2 = Point::new(x2, y2, a.clone(), b.clone())?; // If this fails, the function returns Err

            Ok(&p1 + &p2)
        }
        ```

**Rationale:** This disciplined approach to error handling is crucial for building reliable software. It makes a clear distinction between expected, recoverable failures (like invalid user input) and unexpected programming errors (bugs). Using `.expect()` for bugs and `Result` propagation for recoverable errors makes the code safer, more robust, and easier to debug.

## Error Mapping

When writing libraries or complex systems, low-level errors (like an `InvalidNum` in a finite field math library) often need to be translated into higher-level, domain-specific errors (like `InvalidFormat` or `NotOnCurve` in an elliptic curve library) before being exposed to the user. This is called **Error Mapping**.

**Why Map Errors?**
1.  **Abstraction:** It hides implementation details. A user parsing a public key shouldn't need to know about internal finite field arithmetic errors.
2.  **API Stability:** If you change your internal math library, your public error API (`PointError`) remains the same.
3.  **Clarity:** It translates "what went wrong technically" into "what went wrong semantically" for the user.

**How to Map Errors:**

1.  **`map_err`**: The most explicit way to transform one error type into another.
    ```rust
    // Convert a low-level `FieldElementError` into a high-level `PointError`
    let x = S256FieldElement::new(num)
        .map_err(|_| PointError::InvalidFormat)?;
    ```

2.  **`From` Trait**: Implement `From<LowLevelError> for HighLevelError` to allow automatic conversion with `?`.
    ```rust
    // If you impl From<FieldElementError> for PointError...
    let x = S256FieldElement::new(num)?; // ...the `?` does the conversion automatically.
    ```

## Summary: When to Use What

| Approach | Best For | Why? |
| :--- | :--- | :--- |
| **`Result<T, E>` + Mapping** | **Library / Core Logic** | Provides a clean, stable, and semantic API (`PointError`) while handling internal failures safely. |
| **`Option<T>`** | **Lookups / Checks** | When "not found" or "invalid" is the only failure mode and the reason is obvious (e.g., `HashMap::get`, `hex::decode`). |
| **`anyhow` / `Box<dyn Error>`** | **Application / Scripts** | When you just need to propagate errors to `main()` and print them. Fast to write, but harder for callers to handle specific cases. |
| **`unwrap()` / `expect()`** | **Invariants / Bugs** | When failure is **mathematically impossible** if the code is correct. Use `.expect()` to document *why* it's impossible. |

## Advanced Tools and Libraries

While the standard library provides all the necessary tools for error handling, several popular crates can reduce boilerplate and improve ergonomics, especially in larger applications.

-   **`thiserror`**: A library for creating custom error types. It uses a `derive` macro to automatically implement `std::error::Error` and `std::fmt::Display` for your error enums, reducing boilerplate. This is ideal for library code where you want to define specific, structured error types.

    ```rust
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum DataStoreError {
        #[error("data store disconnected")]
        Disconnect(#[from] io::Error),
        #[error("the data for key `{0}` is not available")]
        Redaction(String),
        #[error("invalid header (expected {expected:?}, found {found:?})")]
        InvalidHeader { expected: String, found: String },
    }
    ```

-   **`anyhow`**: A library designed for easy error handling in applications (as opposed to libraries). It provides a single, concrete `anyhow::Error` type that can wrap any error, and its `anyhow::Result` is a drop-in replacement for `std::result::Result`. It makes it trivial to add context to errors as they are propagated.