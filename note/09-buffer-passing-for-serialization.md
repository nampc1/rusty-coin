# 9. Buffer Passing for Efficient Serialization

**Context:** A naive implementation of serialization often looks like this:

```rust
fn serialize(&self) -> Vec<u8> {
    let mut vec = Vec::new();
    // ... write data ...
    vec
}
```

While simple, this approach causes performance issues when serializing complex nested structures (like a Transaction containing many Inputs and Outputs).
1.  **Excessive Allocation:** Every sub-object allocates its own new `Vec` on the heap.
2.  **Excessive Copying:** The data from these small vectors must be copied *again* into the parent's buffer.

**Decision:** We prefer passing a mutable buffer (or a Writer) to serialization functions.

```rust
// Preferred Pattern
fn serialize(&self, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&self.some_data);
    // ...
}
```

**Rationale:**

1.  **Allocation Efficiency:** We can allocate one large buffer at the top level and reuse it. All sub-objects write directly into this shared memory, eliminating intermediate allocations.
2.  **Composability:** It allows for seamless chaining of serialization logic.
    ```rust
    fn serialize_tx(&self, buf: &mut Vec<u8>) {
        // Serializes directly into the main buffer
        for output in &self.outputs {
            output.serialize(buf); 
        }
    }
    ```
3.  **Idiomatic Rust:** This aligns with standard library traits like `std::io::Write` and popular ecosystems like `serde`. It makes the code easily adaptable to write to files or network streams later.
