//! Comprehensive unit tests for the type system operations
//! 
//! Tests all core type system functionality including:
//! - Content-addressable hashing
//! - Type descriptors and RTTI
//! - Universal value operations
//! - Type registry operations

use mir_types::*;
use mir_types::type_descriptor::{BasicTypeDescriptor, TypeMetadata, FieldMetadata};
use std::collections::HashMap;

#[test]
fn test_type_registry_basic_operations() {
    let mut registry = TypeRegistry::new();
    
    // Test empty registry
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    
    // Register a basic type
    let i32_hash = registry.register_scalar_type("i32", 4);
    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
    
    // Test retrieval
    assert!(registry.contains(i32_hash));
    let retrieved = registry.get(i32_hash);
    assert!(retrieved.is_some());
}

#[test]
fn test_type_registry_multiple_types() {
    let mut registry = TypeRegistry::new();
    
    // Register multiple types
    let i32_hash = registry.register_scalar_type("i32", 4);
    let i64_hash = registry.register_scalar_type("i64", 8);
    let string_hash = registry.register_scalar_type("string", 0);
    
    assert_eq!(registry.len(), 3);
    assert_ne!(i32_hash, i64_hash);
    assert_ne!(i32_hash, string_hash);
    assert_ne!(i64_hash, string_hash);
    
    // All should be retrievable
    assert!(registry.contains(i32_hash));
    assert!(registry.contains(i64_hash));
    assert!(registry.contains(string_hash));
}

#[test]
fn test_type_registry_composite_types() {
    let mut registry = TypeRegistry::new();
    
    // Register scalar types first
    let i32_hash = registry.register_scalar_type("i32", 4);
    let string_hash = registry.register_scalar_type("string", 0);
    
    // Register composite types
    let array_hash = registry.register_array_type(i32_hash);
    let struct_fields = vec![
        ("id".to_string(), i32_hash),
        ("name".to_string(), string_hash),
    ];
    let struct_hash = registry.register_struct_type("Person", struct_fields);
    
    assert_eq!(registry.len(), 4);
    assert!(registry.contains(array_hash));
    assert!(registry.contains(struct_hash));
}

#[test]
fn test_content_hash_consistency() {
    // Test that content hashes are consistent across runs
    let data = b"test data for hashing";
    let hash1 = ContentHash::new(data);
    let hash2 = ContentHash::new(data);
    
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.to_hex(), hash2.to_hex());
}

#[test]
fn test_content_hash_different_data() {
    let hash1 = ContentHash::new(b"data1");
    let hash2 = ContentHash::new(b"data2");
    
    assert_ne!(hash1, hash2);
    assert_ne!(hash1.to_hex(), hash2.to_hex());
}

#[test]
fn test_value_operations_scalar_types() {
    // Test all scalar value operations
    let values = vec![
        Value::I32(42),
        Value::I64(1234567890),
        Value::U32(42),
        Value::F32(3.14),
        Value::F64(2.718281828),
        Value::Bool(true),
        Value::String("test string".to_string()),
    ];
    
    for value in values {
        // Test basic properties
        assert!(value.is_scalar());
        assert!(!value.is_composite());
        
        // Test universal operations
        let hash1 = value.stable_hash();
        let hash2 = value.stable_hash();
        assert_eq!(hash1, hash2);
        
        let cloned = value.deep_clone();
        assert!(value.structural_equals(&cloned));
        
        let pretty = value.pretty_print();
        assert!(!pretty.is_empty());
        assert!(pretty.contains(value.type_name()));
        
        // Test serialization
        let serialized = value.lossless_serialize().unwrap();
        assert!(!serialized.data.is_empty());
        assert_eq!(serialized.type_hash, value.type_hash());
    }
}

#[test]
fn test_value_operations_composite_types() {
    // Test array operations
    let array = Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)]);
    assert!(!array.is_scalar());
    assert!(array.is_composite());
    assert_eq!(array.type_name(), "array");
    
    // Test struct operations
    let mut map = HashMap::new();
    map.insert("id".to_string(), Value::I32(42));
    map.insert("name".to_string(), Value::String("test".to_string()));
    let struct_val = Value::Struct(map);
    
    assert!(!struct_val.is_scalar());
    assert!(struct_val.is_composite());
    assert_eq!(struct_val.type_name(), "struct");
    
    // Test universal operations on composite types
    let hash = struct_val.stable_hash();
    let cloned = struct_val.deep_clone();
    assert!(struct_val.structural_equals(&cloned));
    assert_eq!(struct_val.stable_hash(), hash);
}

#[test]
fn test_value_serialization_roundtrip() {
    let test_values = vec![
        Value::I32(42),
        Value::String("test".to_string()),
        Value::Array(vec![Value::I32(1), Value::I32(2)]),
        Value::Bool(true),
        Value::Null,
    ];
    
    for original in test_values {
        let serialized = original.lossless_serialize().unwrap();
        
        // Verify metadata
        assert_eq!(serialized.metadata.version, 1);
        assert_eq!(serialized.metadata.encoding, "json");
        assert_eq!(serialized.metadata.checksum, ContentHash::new(&serialized.data));
        
        // Verify we can deserialize back to the same value
        let json_str = String::from_utf8(serialized.data).unwrap();
        let deserialized: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }
}

#[test]
fn test_type_hash_operations() {
    let content_hash1 = ContentHash::new(b"type1");
    let content_hash2 = ContentHash::new(b"type2");
    
    let type_hash1 = TypeHash::new(content_hash1);
    let type_hash2 = TypeHash::new(content_hash2);
    
    assert_ne!(type_hash1, type_hash2);
    assert_eq!(type_hash1.content_hash(), content_hash1);
    assert_eq!(type_hash2.content_hash(), content_hash2);
    
    // Test display
    let display = format!("{}", type_hash1);
    assert!(display.contains("TypeHash"));
    assert!(display.contains(&content_hash1.to_hex()));
}

#[test]
fn test_type_descriptor_compatibility() {
    let desc1 = BasicTypeDescriptor::new("TestType".to_string(), b"schema_v1");
    let desc2 = BasicTypeDescriptor::new("TestType".to_string(), b"schema_v1");
    let desc3 = BasicTypeDescriptor::new("TestType".to_string(), b"schema_v2");
    
    // Same schema should be compatible
    assert!(desc1.is_compatible_with(&desc2));
    assert!(desc2.is_compatible_with(&desc1));
    
    // Different schemas should not be compatible
    assert!(!desc1.is_compatible_with(&desc3));
    assert!(!desc3.is_compatible_with(&desc1));
}

#[test]
fn test_type_descriptor_metadata() {
    let metadata = TypeMetadata {
        name: "TestStruct".to_string(),
        size_hint: Some(64),
        alignment: Some(8),
        is_copy: false,
        is_send: true,
        is_sync: true,
        fields: vec![
            FieldMetadata {
                name: "id".to_string(),
                type_hash: TypeHash::new(ContentHash::new(b"i32")),
                offset: Some(0),
                is_optional: false,
            },
            FieldMetadata {
                name: "name".to_string(),
                type_hash: TypeHash::new(ContentHash::new(b"string")),
                offset: Some(8),
                is_optional: true,
            },
        ],
    };
    
    let descriptor = BasicTypeDescriptor::new("TestStruct".to_string(), b"struct_schema")
        .with_metadata(metadata.clone());
    
    let retrieved_metadata = descriptor.metadata();
    assert_eq!(retrieved_metadata.name, metadata.name);
    assert_eq!(retrieved_metadata.size_hint, metadata.size_hint);
    assert_eq!(retrieved_metadata.fields.len(), 2);
    assert_eq!(retrieved_metadata.fields[0].name, "id");
    assert_eq!(retrieved_metadata.fields[1].name, "name");
    assert!(!retrieved_metadata.fields[0].is_optional);
    assert!(retrieved_metadata.fields[1].is_optional);
}

#[test]
fn test_value_size_hints() {
    assert_eq!(Value::I32(42).size_hint(), 4);
    assert_eq!(Value::I64(42).size_hint(), 8);
    assert_eq!(Value::Bool(true).size_hint(), 1);
    assert_eq!(Value::String("hello".to_string()).size_hint(), 5);
    
    let array = Value::Array(vec![Value::I32(1), Value::I32(2)]);
    assert_eq!(array.size_hint(), 4 + 4 + 8); // two i32s + overhead
    
    let union = Value::Union {
        tag: 1,
        value: Box::new(Value::I32(42)),
    };
    assert_eq!(union.size_hint(), 4 + 4); // i32 + tag
}

#[test]
fn test_value_display_formatting() {
    assert_eq!(format!("{}", Value::I32(42)), "42");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::String("test".to_string())), "\"test\"");
    assert_eq!(format!("{}", Value::Null), "null");
    
    let array = Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)]);
    assert_eq!(format!("{}", array), "[1, 2, 3]");
    
    let record = Value::Record(vec![
        ("x".to_string(), Value::I32(10)),
        ("y".to_string(), Value::I32(20)),
    ]);
    assert_eq!(format!("{}", record), "(x: 10, y: 20)");
}

#[test]
fn test_hashable_trait_implementations() {
    let test_string = "test data";
    let test_bytes = b"test data";
    let test_string_owned = test_string.to_string();
    
    let hash1 = test_string.content_hash();
    let hash2 = test_bytes.content_hash();
    let hash3 = test_string_owned.content_hash();
    
    assert_eq!(hash1, hash2);
    assert_eq!(hash1, hash3);
    assert_eq!(hash2, hash3);
}

#[test]
fn test_serialization_error_handling() {
    // Test that serialization handles edge cases
    let values = vec![
        Value::I128(i128::MAX),
        Value::I128(i128::MIN),
        Value::U128(u128::MAX),
        Value::F64(f64::INFINITY),
        Value::F64(f64::NEG_INFINITY),
        Value::F64(f64::NAN),
    ];
    
    for value in values {
        let result = value.lossless_serialize();
        // Should either succeed or fail gracefully
        match result {
            Ok(serialized) => {
                assert!(!serialized.data.is_empty());
                assert_eq!(serialized.type_hash, value.type_hash());
            }
            Err(_) => {
                // Acceptable for edge cases like NaN
            }
        }
    }
}

#[test]
fn test_complex_nested_structures() {
    // Create a complex nested structure
    let mut inner_struct = HashMap::new();
    inner_struct.insert("value".to_string(), Value::I32(42));
    
    let nested = Value::Struct(inner_struct);
    let array_of_structs = Value::Array(vec![nested.clone(), nested.clone()]);
    
    let mut outer_struct = HashMap::new();
    outer_struct.insert("data".to_string(), array_of_structs);
    outer_struct.insert("count".to_string(), Value::I32(2));
    
    let complex_value = Value::Struct(outer_struct);
    
    // Test operations on complex structure
    assert!(complex_value.is_composite());
    assert!(!complex_value.is_scalar());
    
    let hash = complex_value.stable_hash();
    let cloned = complex_value.deep_clone();
    assert_eq!(complex_value.stable_hash(), hash);
    assert!(complex_value.structural_equals(&cloned));
    
    let serialized = complex_value.lossless_serialize().unwrap();
    assert!(!serialized.data.is_empty());
}

#[test]
fn test_type_registry_concurrent_access() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let registry = Arc::new(Mutex::new(TypeRegistry::new()));
    let mut handles = vec![];
    
    // Spawn multiple threads to register types concurrently
    for i in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = thread::spawn(move || {
            let mut reg = registry_clone.lock().unwrap();
            reg.register_scalar_type(&format!("type_{}", i), i * 4)
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    let mut type_hashes = vec![];
    for handle in handles {
        let hash = handle.join().unwrap();
        type_hashes.push(hash);
    }
    
    // Verify all types were registered
    let reg = registry.lock().unwrap();
    assert_eq!(reg.len(), 10);
    
    for hash in type_hashes {
        assert!(reg.contains(hash));
    }
}

#[test]
fn test_performance_with_large_values() {
    use std::time::Instant;
    
    // Create a large array
    let large_array = Value::Array((0..10000).map(Value::I32).collect());
    
    let start = Instant::now();
    let hash = large_array.stable_hash();
    let hash_duration = start.elapsed();
    
    let start = Instant::now();
    let cloned = large_array.deep_clone();
    let clone_duration = start.elapsed();
    
    let start = Instant::now();
    let _serialized = large_array.lossless_serialize().unwrap();
    let serialize_duration = start.elapsed();
    
    // Operations should complete in reasonable time
    assert!(hash_duration.as_millis() < 1000);
    assert!(clone_duration.as_millis() < 1000);
    assert!(serialize_duration.as_millis() < 1000);
    
    // Verify correctness
    assert_eq!(large_array.stable_hash(), hash);
    assert!(large_array.structural_equals(&cloned));
}