//! Content-addressable hashing implementation
//! 
//! Provides SHA-256 based content hashing for all MIR values and types.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A content-addressable hash using SHA-256
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Create a new content hash from bytes
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hasher.finalize().into())
    }

    /// Create a zero hash (for testing)
    pub fn zero() -> Self {
        Self([0; 32])
    }

    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Trait for types that can be content-hashed
pub trait Hashable {
    fn content_hash(&self) -> ContentHash;
}

impl Hashable for &[u8] {
    fn content_hash(&self) -> ContentHash {
        ContentHash::new(self)
    }
}

impl<const N: usize> Hashable for &[u8; N] {
    fn content_hash(&self) -> ContentHash {
        ContentHash::new(*self)
    }
}

impl Hashable for String {
    fn content_hash(&self) -> ContentHash {
        ContentHash::new(self.as_bytes())
    }
}

impl Hashable for &str {
    fn content_hash(&self) -> ContentHash {
        ContentHash::new(self.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_creation() {
        let data = b"hello world";
        let hash = ContentHash::new(data);
        
        // SHA-256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash.to_hex(), expected);
    }

    #[test]
    fn test_content_hash_deterministic() {
        let data = b"test data";
        let hash1 = ContentHash::new(data);
        let hash2 = ContentHash::new(data);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_different_data() {
        let hash1 = ContentHash::new(b"data1");
        let hash2 = ContentHash::new(b"data2");
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_zero() {
        let zero_hash = ContentHash::zero();
        assert_eq!(zero_hash.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_content_hash_hex_conversion() {
        let data = b"test";
        let hash = ContentHash::new(data);
        let hex = hash.to_hex();
        let restored = ContentHash::from_hex(&hex).unwrap();
        
        assert_eq!(hash, restored);
    }

    #[test]
    fn test_content_hash_invalid_hex() {
        let result = ContentHash::from_hex("invalid");
        assert!(result.is_err());
        
        let result = ContentHash::from_hex("deadbeef"); // Too short
        assert!(result.is_err());
    }

    #[test]
    fn test_hashable_trait_string() {
        let s = "test string".to_string();
        let hash = s.content_hash();
        let expected = ContentHash::new(s.as_bytes());
        
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hashable_trait_str() {
        let s = "test string";
        let hash = s.content_hash();
        let expected = ContentHash::new(s.as_bytes());
        
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hashable_trait_bytes() {
        let data = b"test bytes";
        let hash = data.content_hash();
        let expected = ContentHash::new(data);
        
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_content_hash_serialization() {
        let hash = ContentHash::new(b"test");
        let serialized = serde_json::to_string(&hash).unwrap();
        let deserialized: ContentHash = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(hash, deserialized);
    }

    #[test]
    fn test_content_hash_display() {
        let hash = ContentHash::new(b"test");
        let display = format!("{}", hash);
        let hex = hash.to_hex();
        
        assert_eq!(display, hex);
    }

    #[test]
    fn test_content_hash_empty_data() {
        let hash = ContentHash::new(b"");
        // SHA-256 of empty string
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash.to_hex(), expected);
    }

    #[test]
    fn test_content_hash_large_data() {
        let large_data = vec![0u8; 1024 * 1024]; // 1MB of zeros
        let hash = ContentHash::new(&large_data);
        
        // Should handle large data without issues
        assert_ne!(hash, ContentHash::zero());
    }
}