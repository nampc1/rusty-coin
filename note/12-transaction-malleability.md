# 12. Transaction Malleability

**Context:** A transaction's unique identifier (`txid`) is a double-SHA256 hash of its complete serialized data. If a third party can change even a single byte of the transaction data without invalidating it, they can change its `txid`. This is known as **Transaction Malleability**.

## The Attack Vector: The `scriptSig`

The primary vector for malleability attacks was the `scriptSig` field. Before the SegWit upgrade, the `scriptSig` was part of the data hashed to create the `txid`, but its contents were not fully committed to by the signature. This allowed a third party to change *how* the `scriptSig` was encoded without changing its meaning.

### An Example Attack

1.  **The Original Transaction:** Alice creates a transaction. Her `scriptSig` needs to push her 72-byte signature onto the stack. The most compact way to do this is with a single prefix byte: `[0x48, <72-byte signature>]`. The transaction is broadcast with `txid_A`.

2.  **The Malleation:** A malicious node, Mallory, sees this transaction. He can't steal the money, but he can change the `txid`. He changes the `scriptSig` to use the `OP_PUSHDATA1` opcode, which is an alternative way to push the same data: `[0x4c, 0x48, <72-byte signature>]`.

3.  **The Result:**
    *   This new script is functionally identical to the original; it still pushes the exact same signature to the stack and will pass validation.
    *   However, the raw serialization of the transaction is now one byte longer.
    *   This means when the transaction is hashed, it produces a new, different identifier: `txid_B`.

## The Consequence

There are now two valid versions of the same payment floating around the network, `txid_A` and `txid_B`. This created major problems for the ecosystem:

*   **Unreliable Tracking:** Wallet or exchange software watching for `txid_A` to confirm might never see it, because a miner included the `txid_B` version in a block instead. This could lead the software to mistakenly believe the payment failed.
*   **Blocked Innovation:** More complex, multi-transaction contracts (like the original vision for the Lightning Network) were impossible to build safely because they relied on stable, predictable `txid`s.

## The Solutions

The problem has been solved through several layers of fixes over Bitcoin's history.

1.  **The SIGHASH Commitment:** The signature hash process itself is a primary defense. Before hashing and signing, a temporary copy of the transaction is created and modified:
    *   All `scriptSig` fields in all inputs are completely blanked out.
    *   For the one input being signed, its now-empty `scriptSig` is replaced with the `scriptPubKey` from the output it is spending.

    This creates a canonical "state" of the transaction for signing. By signing a hash of this specific state, the signature becomes a commitment to the transaction *without* its own malleable script code. It directly prevents attacks where a third party adds or changes data within the `scriptSig` itself.

2.  **Strict "Canonical" Encoding:** Nodes now enforce strict encoding rules. In the example above, a modern node would reject Mallory's version because `OP_PUSHDATA1` is not the most compact ("canonical") way to push 72 bytes of data.

3.  **Segregated Witness (SegWit) - The Definitive Fix:** The ultimate solution was the SegWit upgrade. It "segregates" (separates) the witness data (signatures and public keys) from the main transaction data.
    *   The `txid` is now calculated from **only** the main data, which does not include the malleable signature information.
    *   The witness data is stored in a separate part of the transaction.
    *   This makes it impossible for a third party to change the `txid` by tampering with the signature, permanently solving the malleability problem for all SegWit transactions.
