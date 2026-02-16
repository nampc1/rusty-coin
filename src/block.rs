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
        todo!()
    }
    
    pub fn mine(&mut self) {
        self.header.mine();
    }
}