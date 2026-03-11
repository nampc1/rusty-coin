use crate::transaction::Tx;
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

pub struct Block {
    header: BlockHeader,
    txs: Vec<Tx>
}

impl Block {
    pub fn new(prev_block_hash: [u8; 32], timestamp: u32, bits: u32, txs: Vec<Tx>) -> Self {
        todo!()
    }

    pub fn is_valid(&self) -> bool {
        todo!()
    }

    fn calculate_merkle_root(txs: &[Tx]) -> [u8; 32] {
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
        let root = Block::merkle_root_from_hashes(&hashes);
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

        let root = Block::merkle_root_from_hashes(&hashes);
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

        let root = Block::merkle_root_from_hashes(&hashes);
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

        let root = Block::merkle_root_from_hashes(&hashes);
        assert_eq!(root, expected_root);
    }
}
