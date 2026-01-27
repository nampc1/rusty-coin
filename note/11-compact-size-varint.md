# 11. Bitcoin "CompactSize" Variable Integer (VarInt)

**Context:** We need to serialize integers efficiently. Most counts in a transaction (number of inputs, length of scripts) are small numbers, so using a fixed 8-byte `u64` is wasteful.

**Decision:** We use the specific "CompactSize" VarInt format defined by the Bitcoin protocol. This is **NOT** the standard LEB128 format used by Protocol Buffers or WebAssembly.

## The Format

The CompactSize format uses a prefix byte to determine the length of the following integer.

| Value Range | Prefix | Total Bytes | Format |
|---|---|---|---|
| `0` to `252` | None | 1 | The byte itself `[val]` |
| `253` to `0xffff` | `0xfd` | 3 | `[fd, le16...]` |
| `0x10000` to `0xffffffff` | `0xfe` | 5 | `[fe, le32...]` |
| `0x100000000` to max | `0xff` | 9 | `[ff, le64...]` |

## Origin and Malleability

This format appears to be an original invention of Satoshi Nakamoto. While efficient to parse, its custom nature led to a significant security vulnerability known as **Transaction Malleability**.

### The Bug
In the early days, the protocol did not enforce "canonical" (shortest) encoding.
-   The number `10` should be encoded as `0x0a` (1 byte).
-   However, it was technically valid to encode it as `0xfd 0a 00` (3 bytes).

### The Attack
An attacker could intercept a valid transaction, modify the VarInt length of a signature (e.g., changing a 1-byte length to a 3-byte length), and rebroadcast it.
-   **Result:** The transaction is still valid (signature is correct), but its binary representation has changed.
-   **Impact:** Since the Transaction ID (TxID) is a hash of the binary data, **the TxID changes**.

This allowed attackers to "malleate" transactions, confusing wallet software (like Mt. Gox's) which tracked payments by TxID. The wallet would see the original TxID "fail" (never confirm), while the modified TxID confirmed secretly, leading to potential double-spending or accounting errors.

**Modern Fix:** Nodes now enforce strict canonical encoding rules. If a number fits in 1 byte, it *must* use 1 byte, or the transaction is rejected.
