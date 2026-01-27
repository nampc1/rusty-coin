# Design Notes

This directory contains documents that record key design decisions and architectural patterns used in the `rusty-coin` project.

## Table of Contents

1. [Operator Overloading for Custom Types](./01-operator-overloading.md)
2. [Ergonomic Function Arguments with the `Into` Trait](./02-into-parameter-pattern.md)
3. Error Handling: The `unwrap()` Dilemma
4. Making Invalid States Unrepresentable
5. Moves vs. Borrows in Operator Implementations
6. [Lazy Static Initialization](./06-lazy-static-initialization.md)
7. [Architectural Refactor: Borrowing vs. Owning Shared Data](./07-borrowing-shared-data.md)
8. [Architectural Refactor: Escaping "Lifetime Hell" with `Arc`](./08-escaping-lifetime-hell-with-arc.md)
9. [Buffer Passing for Efficient Serialization](./09-buffer-passing-for-serialization.md)
10. [Data Representation: Bytes vs. Encodings](./10-bytes-vs-encodings.md)
11. [Bitcoin "CompactSize" Variable Integer (VarInt)](./11-compact-size-varint.md)