//! Unit tests for serialization and deserialization
//! 
//! Tests all serialization functionality including:
//! - Lossless serialization of all value types
//! - Schema-driven serialization with versioning
//! - JSON IR serialization with metadata
//! - Error handling and edge cases

use mir_types::*;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_basic_value_serialization() {
    let test_values = vec![
        Value::I32(42),
        Value::I64(-1234567890),
        Value::U32(4294967295),
        Value::U64(18446744073709551615),
        Value::F32(3.14159),
        Value::F64(2.718281828459045),
        Value::Bool(true),
        Value::Bool(false),
        Value::String("Hello, World!".to_string()),
        Value::Null,
        Value::Undefined,
    ];

    for original_value in test_values {
        // Test lossless serialization
        let serialized = original_value.lossless_serialize().unwrap();
        
        // Verify metadata
        assert_eq!(serialized.metadata.version, 1);
        assert_eq!(serialized.metadata.encoding, "json");
        assert!(serialized.metadata.compression.is_none());
        
        // Verify checksum
        let expected_checksum = ContentHash::new(&serialized.data);
        assert_eq!(serialized.metadata.checksum, expected_checksum);
        
        // Test roundtrip
        let json_str = String::from_utf8(serialized.data).unwrap();
        let deserialized: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original_value, deserialized);
    }
}

#[test]
fn test_composite_value_serialization() {
    // Test array serialization
    let array = Value::Array(vec![
        Value::I32(1),
        Value::I32(2),
        Value::I32(3),
        Value::String("test".to_string()),
    ]);
    
    let serialized = array.lossless_serialize().unwrap();
    let json_str = String::from_utf8(serialized.data).unwrap();
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(array, deserialized);
    
    // Test struct serialization
    let mut map = HashMap::new();
    map.insert("id".to_string(), Value::I32(123));
    map.insert("name".to_string(), Value::String("Alice".to_string()));
    map.insert("active".to_string(), Value::Bool(true));
    
    let struct_val = Value::Struct(map);
    let serialized = struct_val.lossless_serialize().unwrap();
    let json_str = String::from_utf8(serialized.data).unwrap();
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(struct_val, deserialized);
}

#[test]
fn test_nested_structure_serialization() {
    // Create deeply nested structure
    let mut inner_map = HashMap::new();
    inner_map.insert("value".to_string(), Value::I32(42));
    inner_map.insert("flag".to_string(), Value::Bool(true));
    
    let inner_struct = Value::Struct(inner_map);
    let array_with_struct = Value::Array(vec![inner_struct.clone(), inner_struct]);
    
    let mut outer_map = HashMap::new();
    outer_map.insert("data".to_string(), array_with_struct);
    outer_map.insert("count".to_string(), Value::I32(2));
    
    let complex_value = Value::Struct(outer_map);
    
    // Test serialization
    let serialized = complex_value.lossless_serialize().unwrap();
    assert!(!serialized.data.is_empty());
    
    // Test roundtrip
    let json_str = String::from_utf8(serialized.data).unwrap();
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(complex_value, deserialized);
}

#[test]
fn test_union_value_serialization() {
    let union_val = Value::Union {
        tag: 1,
        value: Box::new(Value::String("test_union".to_string())),
    };
    
    let serialized = union_val.lossless_serialize().unwrap();
    let json_str = String::from_utf8(serialized.data).unwrap();
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(union_val, deserialized);
}

#[test]
fn test_record_value_serialization() {
    let record = Value::Record(vec![
        ("x".to_string(), Value::F64(10.5)),
        ("y".to_string(), Value::F64(20.3)),
        ("label".to_string(), Value::String("point".to_string())),
    ]);
    
    let serialized = record.lossless_serialize().unwrap();
    let json_str = String::from_utf8(serialized.data).unwrap();
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(record, deserialized);
}

#[test]
fn test_schema_encoder_decoder() {
    let schema_hash = ContentHash::new(b"test_schema");
    let encoder = JsonEncoder::new(schema_hash);
    let decoder = JsonDecoder::new(schema_hash);
    
    let test_value = Value::String("schema_test".to_string());
    
    // Test encoding
    let encoded = encoder.encode(&test_value).unwrap();
    assert!(!encoded.is_empty());
    
    // Test decoding
    let decoded = decoder.decode(&encoded).unwrap();
    assert_eq!(test_value, decoded);
}

#[test]
fn test_schema_with_context() {
    let schema_hash = ContentHash::new(b"context_test_schema");
    let encoder = JsonEncoder::new(schema_hash);
    let decoder = JsonDecoder::new(schema_hash);
    
    let test_value = Value::I32(999);
    
    // Test encoding with context
    let encoding_context = EncodingContext {
        version: 1,
        compression: None,
    };
    let encoded = encoder.encode_with_context(&test_value, &encoding_context).unwrap();
    
    // Test decoding with context
    let decoding_context = DecodingContext {
        expected_version: 1,
        allow_version_mismatch: false,
    };
    let decoded = decoder.decode_with_context(&encoded, &decoding_context).unwrap();
    assert_eq!(test_value, decoded);
}

#[test]
fn test_version_mismatch_handling() {
    let schema_hash = ContentHash::new(b"version_test_schema");
    let decoder = JsonDecoder::new(schema_hash);
    
    let test_value = Value::Bool(true);
    let encoded = serde_json::to_vec(&test_value).unwrap();
    
    // Test with version mismatch not allowed
    let strict_context = DecodingContext {
        expected_version: 2, // Different from encoder version (1)
        allow_version_mismatch: false,
    };
    
    let result = decoder.decode_with_context(&encoded, &strict_context);
    assert!(matches!(result, Err(SerializationError::VersionMismatch { .. })));
    
    // Test with version mismatch allowed
    let lenient_context = DecodingContext {
        expected_version: 2,
        allow_version_mismatch: true,
    };
    
    let result = decoder.decode_with_context(&encoded, &lenient_context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);
}

#[test]
fn test_schema_registry() {
    let mut registry = SchemaRegistry::new();
    
    let schema_hash = ContentHash::new(b"registry_test_schema");
    let encoder = Arc::new(JsonEncoder::new(schema_hash));
    let decoder = Arc::new(JsonDecoder::new(schema_hash));
    
    let schema = Schema::new(schema_hash, encoder, decoder);
    let registered_hash = registry.register(schema);
    
    assert_eq!(registered_hash, schema_hash);
    assert!(registry.get(schema_hash).is_some());
}

#[test]
fn test_serialization_error_handling() {
    // Test various error conditions
    
    // Test invalid JSON data
    let schema_hash = ContentHash::new(b"error_test_schema");
    let decoder = JsonDecoder::new(schema_hash);
    
    let invalid_json = b"{ invalid json }";
    let result = decoder.decode(invalid_json);
    assert!(matches!(result, Err(SerializationError::InvalidData(_))));
}

#[test]
fn test_large_value_serialization() {
    // Test serialization of large values
    let large_array = Value::Array((0..10000).map(Value::I32).collect());
    
    let serialized = large_array.lossless_serialize().unwrap();
    assert!(!serialized.data.is_empty());
    
    // Verify checksum is correct for large data
    let expected_checksum = ContentHash::new(&serialized.data);
    assert_eq!(serialized.metadata.checksum, expected_checksum);
    
    // Test roundtrip
    let json_str = String::from_utf8(serialized.data).unwrap();
    let deserialized: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(large_array, deserialized);
}

#[test]
fn test_special_float_values() {
    // Test finite special values that JSON can handle
    let special_floats = vec![
        Value::F32(f32::MAX),
        Value::F32(f32::MIN),
        Value::F64(f64::MAX),
        Value::F64(f64::MIN),
    ];
    
    for float_val in special_floats {
        let serialized = float_val.lossless_serialize().unwrap();
        let json_str = String::from_utf8(serialized.data).unwrap();
        let deserialized: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(float_val, deserialized);
    }
    
    // Test that infinity and NaN values can be serialized (even if they become null in JSON)
    let infinite_values = vec![
        Value::F32(f32::INFINITY),
        Value::F32(f32::NEG_INFINITY),
        Value::F64(f64::INFINITY),
        Value::F64(f64::NEG_INFINITY),
        Value::F64(f64::NAN),
    ];
    
    for infinite_val in infinite_values {
        // Should be able to serialize without panicking
        let serialized = infinite_val.lossless_serialize();
        assert!(serialized.is_ok());
        
        // JSON representation might not preserve the exact value for infinity/NaN
        // but the serialization process should not fail
    }
}

#[test]
fn test_unicode_string_serialization() {
    let unicode_strings = vec![
        "Hello, 世界!",
        "🚀 Rust is awesome! 🦀",
        "Ελληνικά",
        "العربية",
        "हिन्दी",
        "🎵🎶🎼",
    ];
    
    for unicode_str in unicode_strings {
        let value = Value::String(unicode_str.to_string());
        let serialized = value.lossless_serialize().unwrap();
        let json_str = String::from_utf8(serialized.data).unwrap();
        let deserialized: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value, deserialized);
    }
}

#[test]
fn test_empty_collections_serialization() {
    let empty_values = vec![
        Value::Array(vec![]),
        Value::Struct(HashMap::new()),
        Value::Record(vec![]),
        Value::String(String::new()),
    ];
    
    for empty_val in empty_values {
        let serialized = empty_val.lossless_serialize().unwrap();
        let json_str = String::from_utf8(serialized.data).unwrap();
        let deserialized: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(empty_val, deserialized);
    }
}

#[test]
fn test_serialization_metadata_consistency() {
    let test_value = Value::String("metadata_test".to_string());
    
    // Serialize multiple times
    let serialized1 = test_value.lossless_serialize().unwrap();
    let serialized2 = test_value.lossless_serialize().unwrap();
    
    // Metadata should be consistent
    assert_eq!(serialized1.metadata.version, serialized2.metadata.version);
    assert_eq!(serialized1.metadata.encoding, serialized2.metadata.encoding);
    assert_eq!(serialized1.metadata.compression, serialized2.metadata.compression);
    
    // Data should be identical
    assert_eq!(serialized1.data, serialized2.data);
    
    // Checksums should match
    assert_eq!(serialized1.metadata.checksum, serialized2.metadata.checksum);
}

#[test]
fn test_type_hash_consistency_in_serialization() {
    let values = vec![
        Value::I32(42),
        Value::String("test".to_string()),
        Value::Bool(true),
        Value::Array(vec![Value::I32(1)]),
    ];
    
    for value in values {
        let serialized = value.lossless_serialize().unwrap();
        let expected_type_hash = value.type_hash();
        
        assert_eq!(serialized.type_hash, expected_type_hash);
    }
}

#[test]
fn test_serialization_performance() {
    use std::time::Instant;
    
    // Create a moderately complex value
    let mut complex_map = HashMap::new();
    for i in 0..100 {
        complex_map.insert(format!("key_{}", i), Value::I32(i));
    }
    let complex_value = Value::Struct(complex_map);
    
    // Measure serialization time
    let start = Instant::now();
    for _ in 0..100 {
        let _serialized = complex_value.lossless_serialize().unwrap();
    }
    let duration = start.elapsed();
    
    // Should complete in reasonable time (less than 1 second for 100 serializations)
    assert!(duration.as_secs() < 1);
}

#[test]
fn test_concurrent_serialization() {
    use std::sync::Arc;
    use std::thread;
    
    let test_value = Arc::new(Value::String("concurrent_test".to_string()));
    let mut handles = vec![];
    
    // Spawn multiple threads to serialize concurrently
    for _ in 0..10 {
        let value_clone = Arc::clone(&test_value);
        let handle = thread::spawn(move || {
            value_clone.lossless_serialize().unwrap()
        });
        handles.push(handle);
    }
    
    // Wait for all threads and verify results
    for handle in handles {
        let serialized = handle.join().unwrap();
        assert!(!serialized.data.is_empty());
        assert_eq!(serialized.metadata.version, 1);
    }
}