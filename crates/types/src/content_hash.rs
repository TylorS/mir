//! Content-addressable hashing using SHA-256

use serde::{Deserialize, Serialize};
use std::fmt;
use sha2::{Sha256, Digest};

/// A content-addressable hash using SHA-256
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Create a new content hash from bytes using SHA-256
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        
        ContentHash(bytes)
    }
    
    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    /// Create from existing bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        ContentHash(bytes)
    }
    
    /// Create a zero hash (useful for testing)
    pub fn zero() -> Self {
        ContentHash([0u8; 32])
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0[..8] {  // Show first 8 bytes for readability
            write!(f, "{:02x}", byte)?;
        }
        write!(f, "...")
    }
}

/// Trait for types that can be content-addressed
pub trait Hashable {
    fn content_hash(&self) -> ContentHash;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_different_inputs() {
        let hash1 = ContentHash::new(b"i32");
        let hash2 = ContentHash::new(b"string");
        
        println!("i32 hash: {}", hash1);
        println!("string hash: {}", hash2);
        println!("i32 hash bytes: {:?}", hash1.as_bytes());
        println!("string hash bytes: {:?}", hash2.as_bytes());
        
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_content_hash_same_inputs() {
        let hash1 = ContentHash::new(b"test");
        let hash2 = ContentHash::new(b"test");
        
        assert_eq!(hash1, hash2);
    }
}