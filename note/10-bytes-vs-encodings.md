# 10. Data Representation: Bytes vs. Encodings

**Context:** In cryptographic applications, data often exists in two forms: raw binary (e.g., a 32-byte public key) and human-readable encodings (e.g., a 64-character hex string or Base58 address). A common anti-pattern is passing these encoded strings deep into the application logic.

**Decision:** We strictly enforce a separation between internal logic and external representation.

1.  **Internal Logic Uses Bytes:** All core business logic, cryptographic operations, and data structures must store and process raw binary data (`Vec<u8>`, `[u8; N]`, or specialized wrapper types like `PublicKey`).
2.  **Strings Are for Boundaries:** Hex, Base58, and other string encodings are treated strictly as **presentation layers**. They are used only for:
    -   User Input (CLI args, JSON APIs)
    -   User Output (Logs, Console display)
    -   Configuration Files

## The Boundary Rule

Data should be converted from its encoded string form to its raw byte form **immediately** at the system boundary.

```rust
// BAD: Passing a hex string deep into the system
fn process_transaction(tx_hex: String) {
    // Logic has to decode it or carry the string around...
}

// GOOD: Converting at the boundary
fn main() {
    let tx_hex = get_user_input();
    let tx_bytes = hex::decode(tx_hex).expect("Invalid hex"); // Boundary check
    process_transaction(&tx_bytes);
}

fn process_transaction(tx: &[u8]) {
    // Logic operates purely on bytes
}
```

## Rationale

1.  **Type Safety & Correctness:** `String` implies valid UTF-8 text. Arbitrary binary data (like ciphertext) is often not valid UTF-8. Storing binary data in a `String` is semantically wrong and can lead to panics if you accidentally try to treat it as text.
2.  **Performance:** A hex string uses 2 bytes for every 1 byte of actual data. Passing strings doubles memory usage and requires constant re-parsing (CPU overhead) whenever you need to do math or crypto on the value.
3.  **Ambiguity:** If a function takes a `String`, is it expected to be Hex? Base64? Base58? If it takes `[u8; 32]`, the requirement is unambiguous.
