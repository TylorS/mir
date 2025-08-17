//! Universal value operations and serialization
//! 
//! Provides comprehensive value operations for all types in the MIR system.

use crate::{ContentHash, Hashable, TypeHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Universal value type that can represent any MIR value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    // Scalar types
    I32(i32),
    I64(i64),
    I128(i128),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    
    // Composite types
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
    Record(Vec<(String, Value)>),
    Union { tag: u32, value: Box<Value> },
    
    // Special values
    Null,
    Undefined,
}

impl Value {
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::I32(_) => "i32",
            Value::I64(_) => "i64",
            Value::I128(_) => "i128",
            Value::U32(_) => "u32",
            Value::U64(_) => "u64",
            Value::U128(_) => "u128",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Struct(_) => "struct",
            Value::Record(_) => "record",
            Value::Union { .. } => "union",
            Value::Null => "null",
            Value::Undefined => "undefined",
        }
    }

    /// Check if this value is a scalar type
    pub fn is_scalar(&self) -> bool {
        matches!(self, 
            Value::I32(_) | Value::I64(_) | Value::I128(_) |
            Value::U32(_) | Value::U64(_) | Value::U128(_) |
            Value::F32(_) | Value::F64(_) | Value::Bool(_) |
            Value::String(_)
        )
    }

    /// Check if this value is a composite type
    pub fn is_composite(&self) -> bool {
        matches!(self, 
            Value::Array(_) | Value::Struct(_) | 
            Value::Record(_) | Value::Union { .. }
        )
    }

    /// Get the size hint for this value in bytes
    pub fn size_hint(&self) -> usize {
        match self {
            Value::I32(_) => 4,
            Value::I64(_) => 8,
            Value::I128(_) => 16,
            Value::U32(_) => 4,
            Value::U64(_) => 8,
            Value::U128(_) => 16,
            Value::F32(_) => 4,
            Value::F64(_) => 8,
            Value::Bool(_) => 1,
            Value::String(s) => s.len(),
            Value::Array(arr) => arr.iter().map(|v| v.size_hint()).sum::<usize>() + 8,
            Value::Struct(map) => map.values().map(|v| v.size_hint()).sum::<usize>() + 8,
            Value::Record(vec) => vec.iter().map(|(_, v)| v.size_hint()).sum::<usize>() + 8,
            Value::Union { value, .. } => value.size_hint() + 4,
            Value::Null | Value::Undefined => 0,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::I128(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::U128(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            Value::Struct(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            },
            Value::Record(vec) => {
                write!(f, "(")?;
                for (i, (key, value)) in vec.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, ")")
            },
            Value::Union { tag, value } => write!(f, "Union({}: {})", tag, value),
            Value::Null => write!(f, "null"),
            Value::Undefined => write!(f, "undefined"),
        }
    }
}

impl Hashable for Value {
    fn content_hash(&self) -> ContentHash {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        ContentHash::new(&serialized)
    }
}

/// Universal value operations trait
pub trait UniversalValueOperations {
    /// Serialize the value losslessly
    fn lossless_serialize(&self) -> Result<SerializedValue, SerializationError>;
    
    /// Compute a stable, deterministic hash
    fn stable_hash(&self) -> ContentHash;
    
    /// Compare values structurally
    fn structural_equals(&self, other: &Value) -> bool;
    
    /// Pretty-print the value with type annotations
    fn pretty_print(&self) -> String;
    
    /// Deep clone the value
    fn deep_clone(&self) -> Value;
    
    /// Get the type hash for this value
    fn type_hash(&self) -> TypeHash;
}

impl UniversalValueOperations for Value {
    fn lossless_serialize(&self) -> Result<SerializedValue, SerializationError> {
        let data = serde_json::to_vec(self)
            .map_err(|e| SerializationError::JsonError(e.to_string()))?;
        
        let checksum = ContentHash::new(&data);
        
        Ok(SerializedValue {
            type_hash: self.type_hash(),
            data,
            metadata: SerializationMetadata {
                version: 1,
                compression: None,
                encoding: "json".to_string(),
                checksum,
            },
        })
    }

    fn stable_hash(&self) -> ContentHash {
        self.content_hash()
    }

    fn structural_equals(&self, other: &Value) -> bool {
        self == other
    }

    fn pretty_print(&self) -> String {
        format!("{}: {}", self.type_name(), self)
    }

    fn deep_clone(&self) -> Value {
        self.clone()
    }

    fn type_hash(&self) -> TypeHash {
        let type_data = format!("type:{}", self.type_name());
        TypeHash::new(ContentHash::new(type_data.as_bytes()))
    }
}

/// Serialized value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedValue {
    pub type_hash: TypeHash,
    pub data: Vec<u8>,
    pub metadata: SerializationMetadata,
}

/// Metadata for serialized values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationMetadata {
    pub version: u32,
    pub compression: Option<String>,
    pub encoding: String,
    pub checksum: ContentHash,
}

/// Serialization errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    JsonError(String),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_names() {
        assert_eq!(Value::I32(42).type_name(), "i32");
        assert_eq!(Value::I64(42).type_name(), "i64");
        assert_eq!(Value::U32(42).type_name(), "u32");
        assert_eq!(Value::F32(3.14).type_name(), "f32");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::String("test".to_string()).type_name(), "string");
        assert_eq!(Value::Array(vec![]).type_name(), "array");
        assert_eq!(Value::Struct(HashMap::new()).type_name(), "struct");
        assert_eq!(Value::Null.type_name(), "null");
    }

    #[test]
    fn test_value_is_scalar() {
        assert!(Value::I32(42).is_scalar());
        assert!(Value::Bool(true).is_scalar());
        assert!(Value::String("test".to_string()).is_scalar());
        assert!(!Value::Array(vec![]).is_scalar());
        assert!(!Value::Struct(HashMap::new()).is_scalar());
        assert!(!Value::Null.is_scalar());
    }

    #[test]
    fn test_value_is_composite() {
        assert!(!Value::I32(42).is_composite());
        assert!(!Value::Bool(true).is_composite());
        assert!(Value::Array(vec![]).is_composite());
        assert!(Value::Struct(HashMap::new()).is_composite());
        assert!(Value::Union { tag: 0, value: Box::new(Value::I32(42)) }.is_composite());
        assert!(!Value::Null.is_composite());
    }

    #[test]
    fn test_value_size_hint() {
        assert_eq!(Value::I32(42).size_hint(), 4);
        assert_eq!(Value::I64(42).size_hint(), 8);
        assert_eq!(Value::Bool(true).size_hint(), 1);
        assert_eq!(Value::String("hello".to_string()).size_hint(), 5);
        assert_eq!(Value::Null.size_hint(), 0);
        
        let array = Value::Array(vec![Value::I32(1), Value::I32(2)]);
        assert_eq!(array.size_hint(), 4 + 4 + 8); // two i32s + array overhead
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::I32(42)), "42");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::String("test".to_string())), "\"test\"");
        assert_eq!(format!("{}", Value::Null), "null");
        
        let array = Value::Array(vec![Value::I32(1), Value::I32(2)]);
        assert_eq!(format!("{}", array), "[1, 2]");
        
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::I32(42));
        let struct_val = Value::Struct(map);
        assert_eq!(format!("{}", struct_val), "{key: 42}");
    }

    #[test]
    fn test_value_content_hash() {
        let val1 = Value::I32(42);
        let val2 = Value::I32(42);
        let val3 = Value::I32(43);
        
        assert_eq!(val1.content_hash(), val2.content_hash());
        assert_ne!(val1.content_hash(), val3.content_hash());
    }

    #[test]
    fn test_universal_value_operations_stable_hash() {
        let value = Value::String("test".to_string());
        let hash1 = value.stable_hash();
        let hash2 = value.stable_hash();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_universal_value_operations_structural_equals() {
        let val1 = Value::I32(42);
        let val2 = Value::I32(42);
        let val3 = Value::I32(43);
        
        assert!(val1.structural_equals(&val2));
        assert!(!val1.structural_equals(&val3));
    }

    #[test]
    fn test_universal_value_operations_pretty_print() {
        let value = Value::I32(42);
        let pretty = value.pretty_print();
        
        assert_eq!(pretty, "i32: 42");
    }

    #[test]
    fn test_universal_value_operations_deep_clone() {
        let original = Value::Array(vec![Value::I32(1), Value::I32(2)]);
        let cloned = original.deep_clone();
        
        assert_eq!(original, cloned);
        assert!(original.structural_equals(&cloned));
    }

    #[test]
    fn test_universal_value_operations_type_hash() {
        let val1 = Value::I32(42);
        let val2 = Value::I32(43);
        let val3 = Value::I64(42);
        
        // Same type should have same type hash
        assert_eq!(val1.type_hash(), val2.type_hash());
        // Different types should have different type hashes
        assert_ne!(val1.type_hash(), val3.type_hash());
    }

    #[test]
    fn test_lossless_serialization() {
        let value = Value::String("test value".to_string());
        let serialized = value.lossless_serialize().unwrap();
        
        assert_eq!(serialized.type_hash, value.type_hash());
        assert_eq!(serialized.metadata.version, 1);
        assert_eq!(serialized.metadata.encoding, "json");
        assert!(!serialized.data.is_empty());
    }

    #[test]
    fn test_serialized_value_metadata() {
        let value = Value::I32(42);
        let serialized = value.lossless_serialize().unwrap();
        
        let metadata = &serialized.metadata;
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.encoding, "json");
        assert!(metadata.compression.is_none());
        assert_eq!(metadata.checksum, ContentHash::new(&serialized.data));
    }

    #[test]
    fn test_complex_value_serialization() {
        let mut map = HashMap::new();
        map.insert("field1".to_string(), Value::I32(42));
        map.insert("field2".to_string(), Value::String("test".to_string()));
        
        let complex_value = Value::Struct(map);
        let serialized = complex_value.lossless_serialize().unwrap();
        
        assert!(!serialized.data.is_empty());
        assert_eq!(serialized.type_hash, complex_value.type_hash());
    }

    #[test]
    fn test_union_value() {
        let union_val = Value::Union {
            tag: 1,
            value: Box::new(Value::String("test".to_string())),
        };
        
        assert_eq!(union_val.type_name(), "union");
        assert!(union_val.is_composite());
        assert_eq!(format!("{}", union_val), "Union(1: \"test\")");
    }

    #[test]
    fn test_record_value() {
        let record = Value::Record(vec![
            ("field1".to_string(), Value::I32(42)),
            ("field2".to_string(), Value::Bool(true)),
        ]);
        
        assert_eq!(record.type_name(), "record");
        assert!(record.is_composite());
        assert_eq!(format!("{}", record), "(field1: 42, field2: true)");
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::Array(vec![Value::I32(1), Value::I32(2)]);
        let val2 = Value::Array(vec![Value::I32(1), Value::I32(2)]);
        let val3 = Value::Array(vec![Value::I32(1), Value::I32(3)]);
        
        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_value_serialization_roundtrip() {
        let original = Value::Array(vec![
            Value::I32(42),
            Value::String("test".to_string()),
            Value::Bool(true),
        ]);
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_large_value_handling() {
        let large_array = Value::Array((0..1000).map(Value::I32).collect());
        let hash = large_array.stable_hash();
        let serialized = large_array.lossless_serialize().unwrap();
        
        assert!(!serialized.data.is_empty());
        assert_eq!(hash, large_array.content_hash());
    }
}