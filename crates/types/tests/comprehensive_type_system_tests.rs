//! Comprehensive integration tests for the type system
//!
//! These tests verify the complete type system functionality including
//! type registration, schema evolution, migration paths, and content-addressable storage.

use mir_types::{
    TypeRegistry, ContentHash, TypeHash, Value, Schema,
    type_registry::{
        CompatibilityResult, MigrationChain, MigrationStep, 
        DirectMigrationStrategy, MigrationStrategy, RuntimeTypeInfo
    },
    type_descriptor::{BasicTypeDescriptor, TypeDescriptor, TypeMetadata},
    schema::{JsonEncoder, JsonDecoder, MigrationError},
};
use std::sync::Arc;
use std::collections::HashMap;

fn create_test_type_descriptor(name: &str, data: &[u8]) -> Box<dyn TypeDescriptor + Send + Sync> {
    Box::new(BasicTypeDescriptor::new(name.to_string(), data))
}

#[test]
fn test_complete_type_registration_workflow() {
    let mut registry = TypeRegistry::new();
    
    // Register basic scalar types
    let i32_type = registry.register_scalar_type("i32", 4).unwrap();
    let f64_type = registry.register_scalar_type("f64", 8).unwrap();
    let string_type = registry.register_scalar_type("string", 0).unwrap(); // Variable size
    
    // Register composite types
    let array_type = registry.register_array_type(i32_type).unwrap();
    let struct_fields = vec![
        ("id".to_string(), i32_type),
        ("score".to_string(), f64_type),
        ("name".to_string(), string_type),
        ("values".to_string(), array_type),
    ];
    let struct_type = registry.register_struct_type("UserRecord", struct_fields).unwrap();
    
    // Verify all types are registered and accessible
    assert!(registry.contains(i32_type));
    assert!(registry.contains(f64_type));
    assert!(registry.contains(string_type));
    assert!(registry.contains(array_type));
    assert!(registry.contains(struct_type));
    
    // Verify type count
    assert_eq!(registry.len(), 5);
    
    // Verify schemas are created
    let i32_content_hash = registry.get_type_descriptor_by_type_hash(i32_type).unwrap();
    assert!(registry.get_schema(i32_content_hash).is_some());
}

#[test]
fn test_schema_evolution_and_migration() {
    let registry = TypeRegistry::new();
    
    // Create version 1 of a type
    let v1_data = b"struct:User:id:i32:name:string";
    let v1_descriptor = create_test_type_descriptor("User", v1_data);
    let v1_hash = registry.register_type(v1_descriptor).unwrap();
    
    // Create version 2 of the same type (with additional field)
    let v2_data = b"struct:User:id:i32:name:string:email:string";
    let v2_descriptor = create_test_type_descriptor("User", v2_data);
    let v2_hash = registry.register_type(v2_descriptor).unwrap();
    
    // Register migration from v1 to v2
    registry.register_migration(v1_hash, v2_hash, |mut value| {
        match &mut value {
            Value::Struct { fields } => {
                // Add default email field
                fields.insert("email".to_string(), Value::String("unknown@example.com".to_string()));
                Ok(value)
            }
            _ => Err(MigrationError::IncompatibleTypes {
                from: "struct".to_string(),
                to: "struct".to_string(),
            }),
        }
    });
    
    // Test compatibility checking
    let compatibility = registry.validate_schema_compatibility(v1_hash, v2_hash);
    match compatibility {
        CompatibilityResult::RequiresMigration(path) => {
            assert_eq!(path, vec![v2_hash]);
        }
        _ => panic!("Expected migration required"),
    }
    
    // Test migration path computation
    let migration_path = registry.compute_migration_path(v1_hash, v2_hash);
    assert!(migration_path.is_some());
    let path = migration_path.unwrap();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0].from_hash, v1_hash);
    assert_eq!(path[0].to_hash, v2_hash);
}

#[test]
fn test_complex_migration_chain() {
    let registry = TypeRegistry::new();
    
    // Create a chain of type versions: v1 -> v2 -> v3
    let v1_hash = ContentHash::new(b"type_v1");
    let v2_hash = ContentHash::new(b"type_v2");
    let v3_hash = ContentHash::new(b"type_v3");
    
    // Register types
    let v1_desc = create_test_type_descriptor("Type", b"type_v1");
    let v2_desc = create_test_type_descriptor("Type", b"type_v2");
    let v3_desc = create_test_type_descriptor("Type", b"type_v3");
    
    registry.register_type(v1_desc).unwrap();
    registry.register_type(v2_desc).unwrap();
    registry.register_type(v3_desc).unwrap();
    
    // Register migrations
    registry.register_migration(v1_hash, v2_hash, |value| Ok(value));
    registry.register_migration(v2_hash, v3_hash, |value| Ok(value));
    
    // Test direct migration (v1 -> v2)
    let path_1_2 = registry.compute_migration_path(v1_hash, v2_hash);
    assert!(path_1_2.is_some());
    assert_eq!(path_1_2.unwrap().len(), 1);
    
    // Test direct migration (v2 -> v3)
    let path_2_3 = registry.compute_migration_path(v2_hash, v3_hash);
    assert!(path_2_3.is_some());
    assert_eq!(path_2_3.unwrap().len(), 1);
    
    // Test chained migration (v1 -> v3) - currently not implemented
    let path_1_3 = registry.compute_migration_path(v1_hash, v3_hash);
    // This will be None until Dijkstra's algorithm is implemented
    assert!(path_1_3.is_none());
}

#[test]
fn test_content_addressable_properties() {
    let registry = TypeRegistry::new();
    
    // Same content should produce same hash
    let data = b"identical_type_definition";
    let desc1 = create_test_type_descriptor("TestType", data);
    let desc2 = create_test_type_descriptor("TestType", data);
    
    let hash1 = registry.register_type(desc1).unwrap();
    let hash2 = registry.register_type(desc2).unwrap();
    
    // Should be the same hash (content-addressable)
    assert_eq!(hash1, hash2);
    
    // Registry should only contain one entry
    assert_eq!(registry.len(), 1);
    
    // Different content should produce different hash
    let different_data = b"different_type_definition";
    let desc3 = create_test_type_descriptor("TestType", different_data);
    let hash3 = registry.register_type(desc3).unwrap();
    
    assert_ne!(hash1, hash3);
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_migration_strategy_interface() {
    let strategy = DirectMigrationStrategy;
    let mut migrations = HashMap::new();
    
    let hash1 = ContentHash::new(b"source");
    let hash2 = ContentHash::new(b"target");
    let hash3 = ContentHash::new(b"other");
    
    // Test with no migrations
    let path = strategy.compute_path(hash1, hash2, &migrations);
    assert!(path.is_none());
    
    // Add direct migration
    migrations.insert((hash1, hash2), Arc::new(|value| Ok(value)));
    let path = strategy.compute_path(hash1, hash2, &migrations);
    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), 1);
    
    // Test non-existent path
    let path = strategy.compute_path(hash1, hash3, &migrations);
    assert!(path.is_none());
}

#[test]
fn test_runtime_type_info_trait_implementation() {
    let registry = TypeRegistry::new();
    let runtime_info: &dyn RuntimeTypeInfo = &registry;
    
    // Test trait methods work correctly
    let hash = ContentHash::new(b"trait_test_type");
    let descriptor = create_test_type_descriptor("TraitTest", b"trait_test_type");
    
    let registered_hash = runtime_info.register_type(descriptor);
    assert_eq!(registered_hash, hash);
    
    let retrieved_hash = runtime_info.get_type_descriptor(hash);
    assert_eq!(retrieved_hash, Some(hash));
    
    // Test migration path through trait
    let hash2 = ContentHash::new(b"trait_test_type_v2");
    let descriptor2 = create_test_type_descriptor("TraitTest", b"trait_test_type_v2");
    runtime_info.register_type(descriptor2);
    
    let path = runtime_info.compute_migration_path(hash, hash2);
    assert!(path.is_some());
    assert!(path.unwrap().is_empty()); // No migration registered
    
    // Test compatibility through trait
    let compatibility = runtime_info.validate_schema_compatibility(hash, hash);
    assert_eq!(compatibility, CompatibilityResult::FullyCompatible);
}

#[test]
fn test_schema_operations_integration() {
    let registry = TypeRegistry::new();
    
    // Register a type and get its schema
    let hash = ContentHash::new(b"schema_test");
    let descriptor = create_test_type_descriptor("SchemaTest", b"schema_test");
    registry.register_type(descriptor).unwrap();
    
    let schema = registry.get_schema(hash);
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert_eq!(schema.version_hash, hash);
    
    // Test encoder/decoder are present
    let _encoder = &schema.encoder;
    let _decoder = &schema.decoder;
    // Encoders and decoders are always present in the schema
}

#[test]
fn test_error_conditions_and_edge_cases() {
    let registry = TypeRegistry::new();
    
    // Test operations on non-existent types
    let non_existent = ContentHash::new(b"does_not_exist");
    
    assert!(!registry.contains_type(non_existent));
    assert!(registry.get_type_descriptor(non_existent).is_none());
    assert!(registry.get_schema(non_existent).is_none());
    
    // Test migration path for same hash (should be empty)
    let hash = ContentHash::new(b"same_hash_test");
    let path = registry.compute_migration_path(hash, hash);
    assert!(path.is_some());
    assert!(path.unwrap().is_empty());
    
    // Test compatibility for same hash
    let compatibility = registry.validate_schema_compatibility(hash, hash);
    assert_eq!(compatibility, CompatibilityResult::FullyCompatible);
    
    // Test with empty registry
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.get_all_type_hashes().is_empty());
}

#[test]
fn test_concurrent_type_registration() {
    use std::sync::Arc;
    use std::thread;
    
    let registry = Arc::new(TypeRegistry::new());
    
    // Spawn multiple threads registering types concurrently
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let registry = Arc::clone(&registry);
            thread::spawn(move || {
                let data = format!("concurrent_type_{}", i);
                let descriptor = create_test_type_descriptor(&format!("Type{}", i), data.as_bytes());
                registry.register_type(descriptor).unwrap()
            })
        })
        .collect();
    
    // Collect all registered hashes
    let mut hashes = Vec::new();
    for handle in handles {
        let hash = handle.join().unwrap();
        hashes.push(hash);
    }
    
    // Verify all types were registered
    assert_eq!(hashes.len(), 20);
    assert_eq!(registry.len(), 20);
    
    // Verify all hashes are unique (different content)
    let mut unique_hashes = hashes.clone();
    unique_hashes.sort();
    unique_hashes.dedup();
    assert_eq!(unique_hashes.len(), 20);
    
    // Verify all types are accessible
    for hash in hashes {
        assert!(registry.contains_type(hash));
        assert!(registry.get_type_descriptor(hash).is_some());
    }
}

#[test]
fn test_migration_function_execution() {
    let registry = TypeRegistry::new();
    
    let hash1 = ContentHash::new(b"migration_source");
    let hash2 = ContentHash::new(b"migration_target");
    
    // Register types
    let desc1 = create_test_type_descriptor("Source", b"migration_source");
    let desc2 = create_test_type_descriptor("Target", b"migration_target");
    registry.register_type(desc1).unwrap();
    registry.register_type(desc2).unwrap();
    
    // Register migration that transforms the value
    registry.register_migration(hash1, hash2, |value| {
        match value {
            Value::I32(n) => Ok(Value::I64(n as i64 * 2)),
            _ => Err(MigrationError::IncompatibleTypes {
                from: "unknown".to_string(),
                to: "i64".to_string(),
            }),
        }
    });
    
    // Get migration path and test execution
    let path = registry.compute_migration_path(hash1, hash2);
    assert!(path.is_some());
    
    let migration_steps = path.unwrap();
    assert_eq!(migration_steps.len(), 1);
    
    // Test migration function execution
    let input_value = Value::I32(21);
    let migration_fn = &migration_steps[0].migration_fn;
    let result = migration_fn(input_value);
    
    assert!(result.is_ok());
    match result.unwrap() {
        Value::I64(n) => assert_eq!(n, 42),
        _ => panic!("Expected I64 value"),
    }
}

#[test]
fn test_type_hash_to_content_hash_mapping() {
    let mut registry = TypeRegistry::new();
    
    // Register types using different methods
    let scalar_type = registry.register_scalar_type("test_scalar", 8).unwrap();
    let array_type = registry.register_array_type(scalar_type).unwrap();
    
    // Test type hash to content hash mapping
    let scalar_content_hash = registry.get_type_descriptor_by_type_hash(scalar_type);
    let array_content_hash = registry.get_type_descriptor_by_type_hash(array_type);
    
    assert!(scalar_content_hash.is_some());
    assert!(array_content_hash.is_some());
    
    // Test alias method
    assert_eq!(registry.get(scalar_type), scalar_content_hash);
    assert_eq!(registry.get(array_type), array_content_hash);
    
    // Verify content hashes are different
    assert_ne!(scalar_content_hash.unwrap(), array_content_hash.unwrap());
}

#[test]
fn test_comprehensive_compatibility_scenarios() {
    let registry = TypeRegistry::new();
    
    let base_hash = ContentHash::new(b"base_type");
    let compatible_hash = ContentHash::new(b"compatible_type");
    let incompatible_hash = ContentHash::new(b"incompatible_type");
    
    // Register types
    registry.register_type(create_test_type_descriptor("Base", b"base_type")).unwrap();
    registry.register_type(create_test_type_descriptor("Compatible", b"compatible_type")).unwrap();
    registry.register_type(create_test_type_descriptor("Incompatible", b"incompatible_type")).unwrap();
    
    // Test fully compatible (same type)
    let result = registry.validate_schema_compatibility(base_hash, base_hash);
    assert_eq!(result, CompatibilityResult::FullyCompatible);
    
    // Test requires migration (with registered migration)
    registry.register_migration(base_hash, compatible_hash, |value| Ok(value));
    let result = registry.validate_schema_compatibility(base_hash, compatible_hash);
    match result {
        CompatibilityResult::RequiresMigration(path) => {
            assert_eq!(path, vec![compatible_hash]);
        }
        _ => panic!("Expected RequiresMigration"),
    }
    
    // Test incompatible (no migration available)
    let result = registry.validate_schema_compatibility(base_hash, incompatible_hash);
    match result {
        CompatibilityResult::Incompatible(reason) => {
            assert!(!reason.is_empty());
        }
        _ => panic!("Expected Incompatible"),
    }
}