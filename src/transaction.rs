use crate::varint::encode_varint;
use sha2::{Sha256, Digest};

pub struct Tx {
   pub version: u32,
   pub tx_ins: Vec<TxIn>,
   pub tx_outs: Vec<TxOut>,
   pub locktime: u32
}

impl Tx {
    pub fn serialize(&self, serialized: &mut Vec<u8>) {
        serialized.extend_from_slice(&self.version.to_le_bytes());
        
        encode_varint(serialized, self.tx_ins.len() as u64);
        for tx_in in &self.tx_ins {
            tx_in.serialize(serialized);
        }
        
        encode_varint(serialized, self.tx_outs.len() as u64);
        for tx_out in &self.tx_outs {
            tx_out.serialize(serialized);
        }
        
        serialized.extend_from_slice(&self.locktime.to_le_bytes());
    }
    
    pub fn hash(&self) -> [u8; 32] {
        let mut serialized = Vec::new();
        
        self.serialize(&mut serialized);
        
        let hash1 = Sha256::digest(serialized);
        let hash2 = Sha256::digest(hash1);
        
        hash2.into()
    }
}

pub struct TxIn {
    pub prev_tx_hash: [u8; 32],
    pub prev_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32
}

impl TxIn {
    /// Serializes the transaction input into the provided buffer.
    pub fn serialize(&self, serialized: &mut Vec<u8>) {
        serialized.extend_from_slice(&self.prev_tx_hash);
        serialized.extend_from_slice(&self.prev_index.to_le_bytes());
        encode_varint(serialized, self.script_sig.len() as u64);
        serialized.extend_from_slice(&self.script_sig);
        serialized.extend_from_slice(&self.sequence.to_le_bytes());
    }
}

pub struct TxOut {
    pub amount: u64,
    pub script_pub_key: Vec<u8>
}

impl TxOut {
    /// Serializes the transaction output into the provided buffer.
    /// See `note/09-buffer-passing-for-serialization.md` for design rationale.
    pub fn serialize(&self, serialized: &mut Vec<u8>) {
        serialized.extend_from_slice(&self.amount.to_le_bytes());
        encode_varint(serialized, self.script_pub_key.len() as u64);
        serialized.extend_from_slice(&self.script_pub_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_serialization() {
        let tx = Tx {
            version: 1,
            tx_ins: vec![TxIn {
                prev_tx_hash: [0; 32],
                prev_index: 0,
                script_sig: vec![0x01, 0x02],
                sequence: 0xffffffff,
            }],
            tx_outs: vec![TxOut {
                amount: 5000,
                script_pub_key: vec![0x03, 0x04],
            }],
            locktime: 0,
        };

        let mut serialized = Vec::new();
        tx.serialize(&mut serialized);

        let mut expected = Vec::new();
        // Version (4 bytes, LE)
        expected.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        // Input Count (VarInt)
        expected.push(0x01);
        // Input 1
        expected.extend_from_slice(&[0; 32]); // prev_tx_hash
        expected.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // prev_index
        expected.push(0x02); // script length
        expected.extend_from_slice(&[0x01, 0x02]); // script
        expected.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // sequence
        // Output Count (VarInt)
        expected.push(0x01);
        // Output 1
        expected.extend_from_slice(&5000u64.to_le_bytes()); // amount
        expected.push(0x02); // script length
        expected.extend_from_slice(&[0x03, 0x04]); // script
        // Locktime
        expected.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_tx_hash_consistency() {
        // Hashing the same transaction twice should produce the same hash
        let tx = Tx {
            version: 1,
            tx_ins: vec![],
            tx_outs: vec![],
            locktime: 0,
        };

        let h1 = tx.hash();
        let h2 = tx.hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_empty_tx() {
        let tx = Tx {
            version: 1,
            tx_ins: vec![],
            tx_outs: vec![],
            locktime: 0,
        };

        let mut serialized = Vec::new();
        tx.serialize(&mut serialized);

        // Version (4) + InCount(1 byte: 0) + OutCount(1 byte: 0) + Locktime (4)
        // 01 00 00 00 | 00 | 00 | 00 00 00 00
        let expected = vec![
            0x01, 0x00, 0x00, 0x00,
            0x00,
            0x00,
            0x00, 0x00, 0x00, 0x00
        ];
        
        assert_eq!(serialized, expected);
    }
}