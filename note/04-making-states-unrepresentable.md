# 4. Making Invalid States Unrepresentable

**Context:** When modeling complex data, it's easy to define structs that allow for logically impossible states. For example, our `Point` on an elliptic curve could be defined with separate `Option` fields for its coordinates:

```rust
// "Before" - A loose definition that allows invalid states
pub struct Point {
    x: Option<FieldElement>,
    y: Option<FieldElement>,
    // ... curve parameters
}
```

This definition allows for four states: `(Some, Some)`, `(None, None)`, `(Some, None)`, and `(None, Some)`. However, only the first two are valid in our domain (a finite point and the point at infinity). The latter two are meaningless. This forces us to handle these impossible states in our logic, often with a `panic!` or `unreachable!()`.

```rust
// This `match` is not exhaustive without a catch-all branch
match (&p1.x, &p1.y, &p2.x, &p2.y) {
    (Some(x1), Some(y1), Some(x2), Some(y2)) => { /* ... */ }
    (None, None, _, _) => { /* ... */ }
    (_, _, None, None) => { /* ... */ }
    _ => unreachable!("Point struct is in an invalid state"),
}
```

**Decision:** We will leverage Rust's `enum` type to make these invalid states unrepresentable at compile time. The type definition itself should enforce the logic of the domain.

```rust
// "After" - A precise definition where invalid states are impossible
enum PointKind {
    Coordinates(FieldElement, FieldElement),
    Infinity,
}

pub struct Point {
    kind: PointKind,
    // ... curve parameters
}
```

With this pattern, a `Point` is *either* `Coordinates` (which guarantees both an `x` and a `y`) *or* `Infinity`. The invalid states are now impossible to construct.

This makes our logic cleaner and safer, as the compiler can prove that our `match` statements are exhaustive.

```rust
// This `match` is exhaustive, no `unreachable!` needed
match (&p1.kind, &p2.kind) {
    (PointKind::Coordinates(x1, y1), PointKind::Coordinates(x2, y2)) => { /* ... */ }
    (PointKind::Infinity, _) => { /* ... */ }
    (_, PointKind::Infinity) => { /* ... */ }
}
```

**Rationale:** This is a core principle of idiomatic Rust. It shifts the burden of correctness from runtime checks (and potential panics) to the compiler.

-   **Safety:** Eliminates an entire class of bugs by making them compile-time errors.
-   **Clarity:** The type definition becomes a form of documentation, clearly stating the possible states.
-   **Maintainability:** Code is simpler and easier to reason about without needing to handle impossible cases.

## Alternative Considered: Separate Structs and Trait Objects

An alternative approach, common in object-oriented languages, would be to define two separate structs and unify them using a trait.

```rust
// Alternative "inheritance" pattern
trait EllipticCurvePoint {
    // ... methods like `add`, `is_infinity`, etc.
}

struct PointCoordinate { /* x, y, a, b */ }
impl EllipticCurvePoint for PointCoordinate { /* ... */ }

struct PointInfinity { /* a, b */ }
impl EllipticCurvePoint for PointInfinity { /* ... */ }
```

To use these heterogeneously (e.g., in a collection or as a function argument), they would need to be handled as trait objects, like `Box<dyn EllipticCurvePoint>`.

**Trade-offs:** This pattern was rejected due to significant disadvantages in a Rust context compared to the `enum` approach.

| Feature         | `enum PointKind` (Chosen)                                     | `dyn EllipticCurvePoint` (Rejected)                               |
|:----------------|:--------------------------------------------------------------|:------------------------------------------------------------------|
| **Memory**      | Stack-allocated by default. Size is `size_of(largest_variant) + tag`. No extra allocations. | **Heap-allocated** (`Box`). Each point creation requires an allocation. |
| **Performance** | **Static Dispatch**. `match` compiles to a direct jump. Extremely fast. | **Dynamic Dispatch**. Method calls go through a vtable lookup, which is slower. |
| **Safety**      | **Compile-time exhaustiveness**. The compiler proves all cases are handled. | **Runtime checks**. Implementations would need slow, error-prone downcasting to check the type of `other`. |
| **Complexity**  | **Low**. Logic is centralized and easy to reason about.        | **High**. Logic is spread out and requires complex patterns to handle interactions between types. |

The `enum` approach is more idiomatic in Rust for this kind of problem because it leverages the type system to provide stronger compile-time guarantees, better performance, and simpler code.

## Deeper Dive: Static vs. Dynamic Dispatch

The performance difference mentioned in the trade-off table is a direct result of how Rust calls methods for enums versus trait objects.

### Static Dispatch (The `enum` way)

When you use an `enum`, the compiler uses **static dispatch**. This means the exact function to be called is known at compile time.

-   **Mechanism:** The compiler generates specialized code for each `enum` variant. A `match` statement becomes a highly optimized lookup table. It checks the enum's "tag" (an internal integer identifying the variant) and makes a direct jump to the correct code block.
-   **Performance:** This is extremely fast, with no runtime overhead. It's the same performance as a `switch` statement in C. This also allows the compiler to perform further optimizations like inlining.
-   **Our Code:** `match &p1.kind` uses static dispatch.

### Dynamic Dispatch (The `trait object` way)

When you use a trait object like `&dyn EllipticCurvePoint`, the compiler uses **dynamic dispatch**. The specific function to be called is determined at runtime.

-   **Mechanism:** A trait object is a "fat pointer" containing two things: a pointer to the data and a pointer to a **vtable** (virtual method table). A vtable is a list of function pointers for the trait's methods. When you call a method, the program follows the vtable pointer at runtime, looks up the correct function pointer in the table, and then calls it.
-   **Performance:** This runtime lookup process (pointer indirection) is slower than a direct function call and happens for every method invocation. It also prevents the compiler from inlining the function.
-   **Our Code:** The rejected `trait EllipticCurvePoint` pattern would have relied on dynamic dispatch.

For our use case, where the set of point states is fixed and known, static dispatch provides superior performance and safety with no real downside.