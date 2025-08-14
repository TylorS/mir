//! Universal value type for the MIR runtime

use crate::{ContentHash, TypeHash, Hashable, SchemaVersion};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    // Composite types (placeholder for now)
    Struct {
        type_hash: TypeHash,
        fields: Vec<(String, Value)>,
    },
    Array {
        element_type: TypeHash,
        elements: Vec<Value>,
    },

    // Function types
    Function {
        type_hash: TypeHash,
        // Function data would be stored separately in the runtime
        function_id: u64,
    },
    Closure {
        type_hash: TypeHash,
        // Closure data would be stored separately in the runtime
        closure_id: u64,
    },
    Continuation {
        type_hash: TypeHash,
        // Continuation data would be stored separately in the runtime
        continuation_id: u64,
    },

    // Special values
    Null,
}

impl Value {
    /// Get the type hash for this value
    pub fn type_hash(&self) -> TypeHash {
        match self {
            Value::Function { type_hash, .. } => *type_hash,
            Value::Closure { type_hash, .. } => *type_hash,
            Value::Continuation { type_hash, .. } => *type_hash,
            Value::Struct { type_hash, .. } => *type_hash,
            Value::Array { element_type, .. } => *element_type,
            _ => {
                // For scalar types, compute a type hash based on the variant
                let type_name = match self {
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
                    Value::Null => "null",
                    _ => unreachable!(),
                };
                TypeHash::new(ContentHash::new(type_name.as_bytes()))
            }
        }
    }
}

/// Serialized value with metadata for lossless serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedValue {
    pub type_hash: TypeHash,
    pub schema_version: SchemaVersion,
    pub data: Vec<u8>,
    pub metadata: SerializationMetadata,
}



/// Metadata for serialization context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializationMetadata {
    pub timestamp: u64,
    pub serializer_version: String,
    pub compression: Option<CompressionType>,
    pub custom_attributes: HashMap<String, String>,
}

/// Compression types for serialized data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// Universal value operations trait
pub trait UniversalValueOperations {
    /// Serialize the value losslessly with full type information
    fn lossless_serialize(&self) -> Result<SerializedValue, SerializationError>;
    
    /// Compute a stable, deterministic hash for the value
    fn stable_hash(&self) -> ContentHash;
    
    /// Compare values structurally for equality
    fn structural_equals(&self, other: &Value) -> bool;
    
    /// Pretty-print the value with type annotations
    fn pretty_print(&self) -> String;
    
    /// Deep clone the value
    fn deep_clone(&self) -> Value;
    
    /// Get the type hash for this value
    fn type_hash(&self) -> TypeHash;
    
    /// Get the schema version for this value
    fn schema_version(&self) -> SchemaVersion;
}

/// Errors that can occur during serialization
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationError {
    TypeNotFound(TypeHash),
    InvalidData(String),
    CompressionFailed(String),
    DecompressionFailed(String),
    VersionMismatch { expected: SchemaVersion, found: SchemaVersion },
    CustomError(String),
}
impl UniversalValueOperations for Value {
    fn lossless_serialize(&self) -> Result<SerializedValue, SerializationError> {
        // Serialize the value to JSON bytes
        let data = serde_json::to_vec(self)
            .map_err(|e| SerializationError::InvalidData(e.to_string()))?;
        
        let metadata = SerializationMetadata {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            serializer_version: "0.1.0".to_string(),
            compression: None,
            custom_attributes: HashMap::new(),
        };
        
        Ok(SerializedValue {
            type_hash: self.type_hash(),
            schema_version: self.schema_version(),
            data,
            metadata,
        })
    }
    
    fn stable_hash(&self) -> ContentHash {
        // Create a deterministic representation for hashing
        let serialized = match self.lossless_serialize() {
            Ok(s) => s.data,
            Err(_) => {
                // Fallback to simple string representation
                format!("{:?}", self).into_bytes()
            }
        };
        
        ContentHash::new(&serialized)
    }
    
    fn structural_equals(&self, other: &Value) -> bool {
        match (self, other) {
            // Scalar types - direct comparison
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::I128(a), Value::I128(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            (Value::U128(a), Value::U128(b)) => a == b,
            (Value::F32(a), Value::F32(b)) => {
                // Handle NaN and infinity correctly
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            },
            (Value::F64(a), Value::F64(b)) => {
                // Handle NaN and infinity correctly
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            },
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null, Value::Null) => true,
            
            // Composite types - structural comparison
            (Value::Struct { type_hash: th1, fields: f1 }, 
             Value::Struct { type_hash: th2, fields: f2 }) => {
                th1 == th2 && f1.len() == f2.len() && 
                f1.iter().zip(f2.iter()).all(|((n1, v1), (n2, v2))| {
                    n1 == n2 && v1.structural_equals(v2)
                })
            },
            (Value::Array { element_type: et1, elements: e1 },
             Value::Array { element_type: et2, elements: e2 }) => {
                et1 == et2 && e1.len() == e2.len() &&
                e1.iter().zip(e2.iter()).all(|(v1, v2)| v1.structural_equals(v2))
            },
            
            // Function types - compare by ID and type
            (Value::Function { type_hash: th1, function_id: id1 },
             Value::Function { type_hash: th2, function_id: id2 }) => {
                th1 == th2 && id1 == id2
            },
            (Value::Closure { type_hash: th1, closure_id: id1 },
             Value::Closure { type_hash: th2, closure_id: id2 }) => {
                th1 == th2 && id1 == id2
            },
            (Value::Continuation { type_hash: th1, continuation_id: id1 },
             Value::Continuation { type_hash: th2, continuation_id: id2 }) => {
                th1 == th2 && id1 == id2
            },
            
            // Different types are never equal
            _ => false,
        }
    }
    
    fn pretty_print(&self) -> String {
        match self {
            Value::I32(v) => format!("{}i32", v),
            Value::I64(v) => format!("{}i64", v),
            Value::I128(v) => format!("{}i128", v),
            Value::U32(v) => format!("{}u32", v),
            Value::U64(v) => format!("{}u64", v),
            Value::U128(v) => format!("{}u128", v),
            Value::F32(v) => {
                if v.is_nan() {
                    "NaN_f32".to_string()
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        "∞_f32".to_string()
                    } else {
                        "-∞_f32".to_string()
                    }
                } else {
                    format!("{}f32", v)
                }
            },
            Value::F64(v) => {
                if v.is_nan() {
                    "NaN_f64".to_string()
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        "∞_f64".to_string()
                    } else {
                        "-∞_f64".to_string()
                    }
                } else {
                    format!("{}f64", v)
                }
            },
            Value::Bool(v) => format!("{}:bool", v),
            Value::String(v) => format!("\"{}\":string", v),
            Value::Null => "null".to_string(),
            
            Value::Struct { type_hash, fields } => {
                let fields_str = fields.iter()
                    .map(|(name, value)| format!("{}: {}", name, value.pretty_print()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("struct@{} {{ {} }}", type_hash, fields_str)
            },
            
            Value::Array { element_type, elements } => {
                let elements_str = elements.iter()
                    .map(|v| v.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("array<{}>@{} [{}]", element_type, element_type, elements_str)
            },
            
            Value::Function { type_hash, function_id } => {
                format!("function@{}#{}", type_hash, function_id)
            },
            
            Value::Closure { type_hash, closure_id } => {
                format!("closure@{}#{}", type_hash, closure_id)
            },
            
            Value::Continuation { type_hash, continuation_id } => {
                format!("continuation@{}#{}", type_hash, continuation_id)
            },
        }
    }
    
    fn deep_clone(&self) -> Value {
        // Since Value is already Clone, we can use clone
        // For more complex types with shared references, this would need custom logic
        self.clone()
    }
    
    fn type_hash(&self) -> TypeHash {
        // Delegate to the direct method to avoid duplication
        self.type_hash()
    }
    
    fn schema_version(&self) -> SchemaVersion {
        // For now, use the type hash as the schema version
        // In a full implementation, this would track actual schema evolution
        SchemaVersion(self.type_hash().hash())
    }
}

impl Hashable for Value {
    fn content_hash(&self) -> ContentHash {
        self.stable_hash()
    }
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationError::TypeNotFound(hash) => {
                write!(f, "Type not found: {}", hash)
            },
            SerializationError::InvalidData(msg) => {
                write!(f, "Invalid data: {}", msg)
            },
            SerializationError::CompressionFailed(msg) => {
                write!(f, "Compression failed: {}", msg)
            },
            SerializationError::DecompressionFailed(msg) => {
                write!(f, "Decompression failed: {}", msg)
            },
            SerializationError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {:?}, found {:?}", expected, found)
            },
            SerializationError::CustomError(msg) => {
                write!(f, "Custom error: {}", msg)
            },
        }
    }
}

impl std::error::Error for SerializationError {}#[
cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lossless_serialization() {
        let value = Value::I32(42);
        let serialized = value.lossless_serialize().unwrap();
        
        // Verify the serialized value contains the correct type information
        assert_eq!(serialized.type_hash, value.type_hash());
        assert_eq!(serialized.schema_version, value.schema_version());
        assert!(!serialized.data.is_empty());
        
        // Verify we can deserialize back to the same value
        let deserialized: Value = serde_json::from_slice(&serialized.data).unwrap();
        assert!(value.structural_equals(&deserialized));
    }
    
    #[test]
    fn test_stable_hash_deterministic() {
        let value1 = Value::String("hello world".to_string());
        let value2 = Value::String("hello world".to_string());
        
        // Same values should have same hash
        assert_eq!(value1.stable_hash(), value2.stable_hash());
        
        let value3 = Value::String("different".to_string());
        // Different values should have different hashes
        assert_ne!(value1.stable_hash(), value3.stable_hash());
    }
    
    #[test]
    fn test_structural_equals() {
        // Test scalar equality
        assert!(Value::I32(42).structural_equals(&Value::I32(42)));
        assert!(!Value::I32(42).structural_equals(&Value::I32(43)));
        assert!(!Value::I32(42).structural_equals(&Value::I64(42)));
        
        // Test string equality
        let s1 = Value::String("test".to_string());
        let s2 = Value::String("test".to_string());
        let s3 = Value::String("different".to_string());
        assert!(s1.structural_equals(&s2));
        assert!(!s1.structural_equals(&s3));
        
        // Test float special values
        let nan1 = Value::F64(f64::NAN);
        let nan2 = Value::F64(f64::NAN);
        assert!(nan1.structural_equals(&nan2)); // NaN should equal NaN
        
        let inf1 = Value::F64(f64::INFINITY);
        let inf2 = Value::F64(f64::INFINITY);
        assert!(inf1.structural_equals(&inf2));
        
        let neg_inf = Value::F64(f64::NEG_INFINITY);
        assert!(!inf1.structural_equals(&neg_inf));
    }
    
    #[test]
    fn test_pretty_print() {
        // Test scalar types
        assert_eq!(Value::I32(42).pretty_print(), "42i32");
        assert_eq!(Value::Bool(true).pretty_print(), "true:bool");
        assert_eq!(Value::String("hello".to_string()).pretty_print(), "\"hello\":string");
        assert_eq!(Value::Null.pretty_print(), "null");
        
        // Test special float values
        assert_eq!(Value::F64(f64::NAN).pretty_print(), "NaN_f64");
        assert_eq!(Value::F64(f64::INFINITY).pretty_print(), "∞_f64");
        assert_eq!(Value::F64(f64::NEG_INFINITY).pretty_print(), "-∞_f64");
        
        // Test composite types
        let struct_value = Value::Struct {
            type_hash: TypeHash::new(ContentHash::new(b"test_struct")),
            fields: vec![
                ("name".to_string(), Value::String("test".to_string())),
                ("age".to_string(), Value::I32(25)),
            ],
        };
        let pretty = struct_value.pretty_print();
        assert!(pretty.contains("struct@"));
        assert!(pretty.contains("name: \"test\":string"));
        assert!(pretty.contains("age: 25i32"));
    }
    
    #[test]
    fn test_array_operations() {
        let array = Value::Array {
            element_type: TypeHash::new(ContentHash::new(b"i32")),
            elements: vec![Value::I32(1), Value::I32(2), Value::I32(3)],
        };
        
        // Test structural equality
        let array2 = Value::Array {
            element_type: TypeHash::new(ContentHash::new(b"i32")),
            elements: vec![Value::I32(1), Value::I32(2), Value::I32(3)],
        };
        assert!(array.structural_equals(&array2));
        
        // Test different arrays
        let array3 = Value::Array {
            element_type: TypeHash::new(ContentHash::new(b"i32")),
            elements: vec![Value::I32(1), Value::I32(2), Value::I32(4)],
        };
        assert!(!array.structural_equals(&array3));
        
        // Test pretty printing
        let pretty = array.pretty_print();
        assert!(pretty.contains("array<"));
        assert!(pretty.contains("1i32, 2i32, 3i32"));
    }
    
    #[test]
    fn test_deep_clone() {
        let original = Value::Struct {
            type_hash: TypeHash::new(ContentHash::new(b"test")),
            fields: vec![
                ("field1".to_string(), Value::I32(42)),
                ("field2".to_string(), Value::String("test".to_string())),
            ],
        };
        
        let cloned = original.deep_clone();
        assert!(original.structural_equals(&cloned));
        
        // Verify they have the same hash
        assert_eq!(original.stable_hash(), cloned.stable_hash());
    }
    
    #[test]
    fn test_serialization_metadata() {
        let value = Value::Bool(true);
        let serialized = value.lossless_serialize().unwrap();
        
        assert_eq!(serialized.metadata.serializer_version, "0.1.0");
        assert_eq!(serialized.metadata.compression, None);
        assert!(serialized.metadata.timestamp > 0);
        assert!(serialized.metadata.custom_attributes.is_empty());
    }
    
    #[test]
    fn test_type_hash_different_types() {
        // Test the ContentHash directly first
        let i32_content_hash = ContentHash::new(b"i32");
        let string_content_hash = ContentHash::new(b"string");
        
        println!("Direct i32 content hash: {}", i32_content_hash);
        println!("Direct string content hash: {}", string_content_hash);
        
        assert_ne!(i32_content_hash, string_content_hash);
        
        // Now test the TypeHash
        let i32_type_hash = TypeHash::new(i32_content_hash);
        let string_type_hash = TypeHash::new(string_content_hash);
        
        println!("i32 type hash: {}", i32_type_hash);
        println!("string type hash: {}", string_type_hash);
        
        assert_ne!(i32_type_hash, string_type_hash);
        
        // Now test the Value type_hash method
        let i32_value = Value::I32(42);
        let string_value = Value::String("hello".to_string());
        
        let i32_hash = i32_value.type_hash();
        let string_hash = string_value.type_hash();
        
        println!("Value i32 type hash: {}", i32_hash);
        println!("Value string type hash: {}", string_hash);
        
        println!("i32 hash bytes: {:?}", i32_hash.as_content_hash().as_bytes());
        println!("string hash bytes: {:?}", string_hash.as_content_hash().as_bytes());
        
        assert_ne!(i32_hash, string_hash);
    }
}