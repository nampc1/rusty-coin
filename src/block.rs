use crate::transaction::{Transaction};
use num_bigint::BigUint;
use sha2::{Sha256, Digest};

pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(80); // Header is always 80 bytes
        
        result.extend_from_slice(&self.version.to_le_bytes());
        result.extend_from_slice(&self.prev_block_hash);
        result.extend_from_slice(&self.merkle_root);
        result.extend_from_slice(&self.timestamp.to_le_bytes());
        result.extend_from_slice(&self.bits.to_le_bytes());
        result.extend_from_slice(&self.nonce.to_le_bytes());
        
        result
    }
    
    pub fn hash(&self) -> [u8; 32] {
        let serialized_header = self.serialize();
        
        let hash1 = Sha256::digest(serialized_header);
        let hash2 = Sha256::digest(hash1);
        
        hash2.into()
    }
    
    pub fn bits_to_target(bits: u32) -> BigUint {
        let exponent = (bits >> 24) as u8;
        let coef = BigUint::from(bits & 0x00FFFFFF);
        let shift = 8 * (exponent as u32 - 3);
        
        coef << shift
    }
    
    pub fn mine(&mut self) {
        let target = Self::bits_to_target(self.bits);
        
        while BigUint::from_bytes_be(&self.hash()) >= target {
            self.nonce += 1;
        }
        
        println!("Mined! Nonce {}", self.nonce);
    }
}

pub struct Block<T: Transaction> {
    header: BlockHeader,
    txs: Vec<T>
}

impl<T: Transaction> Block<T> {
    pub fn new(prev_block_hash: [u8; 32], timestamp: u32, bits: u32, txs: Vec<T>) -> Self {
        let merkle_root = Self::calculate_merkle_root(&txs);
        let header = BlockHeader {
            version: 1,
            prev_block_hash,
            merkle_root,
            timestamp,
            bits,
            nonce: 0
        };
        
        Block {
            header,
            txs 
        }
    }

    pub fn is_valid(&self) -> bool {
        let target = BlockHeader::bits_to_target(self.header.bits);
        
        if BigUint::from_bytes_be(&self.header.hash()) >= target {
            return false;
        }
        
        let expected_merkle_root = Self::calculate_merkle_root(&self.txs);
        if expected_merkle_root != self.header.merkle_root {
            return false;
        }
        
        true
    }

    fn calculate_merkle_root(txs: &[T]) -> [u8; 32] {
        if txs.is_empty() {
            panic!("Cannot compute merkle root without transactions");
        }
        
        let hashes: Vec<[u8; 32]> = txs.iter().map(|tx| tx.hash()).collect();
        
        Self::merkle_root_from_hashes(&hashes)
    }
    
    fn merkle_root_from_hashes(hashes: &[[u8; 32]]) -> [u8; 32] {
        if hashes.is_empty() {
            panic!("Cannot compute merkle root without hashes");
        }

        if hashes.len() == 1 {
            return hashes[0];
        }
        
        let mut current_level: Vec<[u8; 32]> = hashes.to_vec();

        loop {
            let mut next_level: Vec<[u8; 32]> = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let first_hash = chunk.first().unwrap();
                let mut concat_hash = [0u8; 64];
                
                if let Some(second_hash) = chunk.last() {
                    concat_hash[0..32].copy_from_slice(first_hash);
                    concat_hash[32..64].copy_from_slice(second_hash);
                } else {
                    concat_hash[0..32].copy_from_slice(first_hash);
                    concat_hash[32..64].copy_from_slice(first_hash);
                }
                
                let hash1 = Sha256::digest(concat_hash);
                let hash2 = Sha256::digest(hash1);
                next_level.push(hash2.into());
            }
            
            if next_level.len() == 1 {
                return next_level[0];
            } else {
                current_level = next_level;
            }
        }
    }
    
    pub fn mine(&mut self) {
        self.header.mine();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d_sha256(data: &[u8]) -> [u8; 32] {
        let h1 = Sha256::digest(data);
        Sha256::digest(h1).into()
    }

    #[test]
    fn test_merkle_root_single_tx() {
        let h1 = d_sha256(b"tx1");
        let hashes = vec![h1];
        // For a single hash, the root is the hash itself.
        let root = Block::<MockTx>::merkle_root_from_hashes(&hashes);
        assert_eq!(root, h1);
    }

    #[test]
    fn test_merkle_root_two_txs() {
        let h1 = d_sha256(b"tx1");
        let h2 = d_sha256(b"tx2");
        let hashes = vec![h1, h2];

        // Manually calculate the expected root for two transactions.
        let mut concat = [0u8; 64];
        concat[0..32].copy_from_slice(&h1);
        concat[32..64].copy_from_slice(&h2);
        let expected_root = d_sha256(&concat);

        let root = Block::<MockTx>::merkle_root_from_hashes(&hashes);
        assert_eq!(root, expected_root);
    }

    #[test]
    fn test_merkle_root_three_txs() {
        let h1 = d_sha256(b"tx1");
        let h2 = d_sha256(b"tx2");
        let h3 = d_sha256(b"tx3");
        let hashes = vec![h1, h2, h3];

        // --- Level 1 ---
        // Hash of (h1 + h2)
        let mut concat_1_2 = [0u8; 64];
        concat_1_2[0..32].copy_from_slice(&h1);
        concat_1_2[32..64].copy_from_slice(&h2);
        let h12 = d_sha256(&concat_1_2);

        // Hash of (h3 + h3) because it's an odd number
        let mut concat_3_3 = [0u8; 64];
        concat_3_3[0..32].copy_from_slice(&h3);
        concat_3_3[32..64].copy_from_slice(&h3);
        let h33 = d_sha256(&concat_3_3);
        
        // --- Level 2 ---
        // Hash of (h12 + h33)
        let mut concat_final = [0u8; 64];
        concat_final[0..32].copy_from_slice(&h12);
        concat_final[32..64].copy_from_slice(&h33);
        let expected_root = d_sha256(&concat_final);

        let root = Block::<MockTx>::merkle_root_from_hashes(&hashes);
        assert_eq!(root, expected_root);
    }

    #[test]
    fn test_merkle_root_four_txs() {
        let h1 = d_sha256(b"tx1");
        let h2 = d_sha256(b"tx2");
        let h3 = d_sha256(b"tx3");
        let h4 = d_sha256(b"tx4");
        let hashes = vec![h1, h2, h3, h4];

        // --- Level 1 ---
        // Hash of (h1 + h2)
        let mut concat_1_2 = [0u8; 64];
        concat_1_2[0..32].copy_from_slice(&h1);
        concat_1_2[32..64].copy_from_slice(&h2);
        let h12 = d_sha256(&concat_1_2);

        // Hash of (h3 + h4)
        let mut concat_3_4 = [0u8; 64];
        concat_3_4[0..32].copy_from_slice(&h3);
        concat_3_4[32..64].copy_from_slice(&h4);
        let h34 = d_sha256(&concat_3_4);
        
        // --- Level 2 ---
        // Hash of (h12 + h34)
        let mut concat_final = [0u8; 64];
        concat_final[0..32].copy_from_slice(&h12);
        concat_final[32..64].copy_from_slice(&h34);
        let expected_root = d_sha256(&concat_final);

        let root = Block::<MockTx>::merkle_root_from_hashes(&hashes);
        assert_eq!(root, expected_root);
    }
    
    // Mock transaction struct for testing purposes.
    #[derive(Clone)]
    struct MockTx {
        id: Vec<u8>,
    }

    impl MockTx {
        fn new(id: &[u8]) -> Self {
            MockTx { id: id.to_vec() }
        }
    }

    // A mock implementation of the Tx trait for our MockTx.
    // This is a simplified stand-in for the real transaction logic.
    impl crate::transaction::Transaction for MockTx {
        fn hash(&self) -> [u8; 32] {
            d_sha256(&self.id)
        }
    }

    fn create_test_block(txs: Vec<MockTx>) -> Block<MockTx> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let prev_block_hash = [0; 32];
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        // A high `bits` value creates a very easy target.
        let bits = 0x207fffff; 

        Block::new(prev_block_hash, timestamp, bits, txs)
    }

    #[test]
    fn test_new_block_creation() {
        let tx1 = MockTx::new(b"tx1");
        let tx2 = MockTx::new(b"tx2");
        let txs = vec![tx1.clone(), tx2.clone()];
        
        let block = create_test_block(txs);
        
        let h1 = tx1.hash();
        let h2 = tx2.hash();
        let mut concat = [0u8; 64];
        concat[0..32].copy_from_slice(&h1);
        concat[32..64].copy_from_slice(&h2);
        let expected_merkle_root = d_sha256(&concat);

        assert_eq!(block.header.version, 1);
        assert_eq!(block.header.prev_block_hash, [0; 32]);
        assert_eq!(block.header.merkle_root, expected_merkle_root);
        assert_eq!(block.header.nonce, 0);
        assert_eq!(block.txs.len(), 2);
    }
    
    #[test]
    fn test_mining_and_is_valid() {
        let tx = MockTx::new(b"some tx");
        let mut block = create_test_block(vec![tx]);

        // 1. Before mining, the block should be invalid because PoW is not met.
        // We set a difficult target to ensure the initial nonce is invalid.
        block.header.bits = 0x1e7fffff; // A much harder target
        assert!(!block.is_valid(), "Block should be invalid before mining");

        // 2. Mine the block.
        block.mine();
        
        // 3. After mining, the block should be valid.
        assert!(block.is_valid(), "Block should be valid after mining");

        // 4. Test invalid Merkle root
        let mut invalid_block = block;
        invalid_block.header.merkle_root[5] = !invalid_block.header.merkle_root[5]; // Corrupt the root
        assert!(!invalid_block.is_valid(), "Block with invalid merkle root should be invalid");
    }
}
