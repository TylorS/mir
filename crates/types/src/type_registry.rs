//! Type registry for content-addressable type storage

use crate::{ContentHash, TypeDescriptor, TypeHash, Value, Schema, JsonEncoder, JsonDecoder, SchemaVersion};
use crate::schema::MigrationError;

use std::collections::HashMap;
use std::sync::Arc;

/// Migration chain for schema evolution
pub type MigrationChain = Vec<MigrationStep>;

/// Single migration step
pub struct MigrationStep {
    pub from_hash: ContentHash,
    pub to_hash: ContentHash,
    pub migration_fn: Box<dyn Fn(Value) -> Result<Value, MigrationError>>,
}



/// Schema compatibility result
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityResult {
    FullyCompatible,
    BackwardCompatible,
    RequiresMigration(Vec<ContentHash>), // Migration path
    Incompatible(String), // Reason for incompatibility
}

/// Runtime type information registry
pub struct TypeRegistry {
    /// Map from content hash to type descriptor
    type_descriptors: HashMap<ContentHash, Box<dyn TypeDescriptor>>,
    /// Map from type hash to content hash
    type_hash_map: HashMap<TypeHash, ContentHash>,
    /// Schema information for each type
    schemas: HashMap<ContentHash, Schema>,
    /// Migration functions between schema versions
    migrations: HashMap<(ContentHash, ContentHash), Box<dyn Fn(Value) -> Result<Value, MigrationError>>>,
}

impl TypeRegistry {
    /// Create a new empty type registry
    pub fn new() -> Self {
        TypeRegistry {
            type_descriptors: HashMap::new(),
            type_hash_map: HashMap::new(),
            schemas: HashMap::new(),
            migrations: HashMap::new(),
        }
    }
    
    /// Register a new type descriptor
    pub fn register_type(&mut self, descriptor: Box<dyn TypeDescriptor>) -> ContentHash {
        let hash = descriptor.schema_hash();
        let type_hash = TypeHash::new(hash);
        
        self.type_descriptors.insert(hash, descriptor);
        self.type_hash_map.insert(type_hash, hash);
        
        // Create a basic schema for this type
        let encoder = Arc::new(JsonEncoder::new(hash));
        let decoder = Arc::new(JsonDecoder::new(hash));
        let schema = Schema::new(hash, encoder, decoder);
        self.schemas.insert(hash, schema);
        
        hash
    }
    
    /// Get a type descriptor by content hash
    pub fn get_type_descriptor(&self, hash: ContentHash) -> Option<&dyn TypeDescriptor> {
        self.type_descriptors.get(&hash).map(|desc| desc.as_ref())
    }
    
    /// Get a type descriptor by type hash
    pub fn get_type_descriptor_by_type_hash(&self, type_hash: TypeHash) -> Option<&dyn TypeDescriptor> {
        self.type_hash_map.get(&type_hash)
            .and_then(|content_hash| self.get_type_descriptor(*content_hash))
    }
    
    /// Register a migration function between two schema versions
    pub fn register_migration<F>(&mut self, from_hash: ContentHash, to_hash: ContentHash, migration: F)
    where
        F: Fn(Value) -> Result<Value, MigrationError> + 'static,
    {
        self.migrations.insert((from_hash, to_hash), Box::new(migration));
    }
    
    /// Compute migration path from one schema to another
    pub fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain> {
        if from == to {
            return Some(vec![]);
        }
        
        // Simple direct migration check for now
        // TODO: Implement proper path finding algorithm for complex migration chains
        if self.migrations.contains_key(&(from, to)) {
            return Some(vec![MigrationStep {
                from_hash: from,
                to_hash: to,
                migration_fn: Box::new(move |_value| {
                    // This is a placeholder - in real implementation, we'd need to clone the function
                    Err(MigrationError {
                        message: "Migration not implemented".to_string(),
                        source_version: Some(SchemaVersion::new(from)),
                        target_version: Some(SchemaVersion::new(to)),
                        value_type: None,
                    })
                }),
            }]);
        }
        
        None
    }
    
    /// Validate schema compatibility between two versions
    pub fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult {
        if old == new {
            return CompatibilityResult::FullyCompatible;
        }
        
        // Check if we have a direct migration path
        if self.migrations.contains_key(&(old, new)) {
            return CompatibilityResult::RequiresMigration(vec![new]);
        }
        
        // For now, assume incompatible if no migration exists
        // TODO: Implement proper compatibility analysis based on type structure
        CompatibilityResult::Incompatible("No migration path found".to_string())
    }
    
    /// Get all registered type hashes
    pub fn get_all_type_hashes(&self) -> Vec<ContentHash> {
        self.type_descriptors.keys().copied().collect()
    }
    
    /// Check if a type is registered
    pub fn contains_type(&self, hash: ContentHash) -> bool {
        self.type_descriptors.contains_key(&hash)
    }
    
    /// Get schema for a type
    pub fn get_schema(&self, hash: ContentHash) -> Option<&Schema> {
        self.schemas.get(&hash)
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime type information trait
pub trait RuntimeTypeInfo {
    fn get_type_descriptor(&self, hash: ContentHash) -> Option<&dyn TypeDescriptor>;
    fn register_type(&mut self, descriptor: Box<dyn TypeDescriptor>) -> ContentHash;
    fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain>;
    fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult;
}

impl RuntimeTypeInfo for TypeRegistry {
    fn get_type_descriptor(&self, hash: ContentHash) -> Option<&dyn TypeDescriptor> {
        self.get_type_descriptor(hash)
    }
    
    fn register_type(&mut self, descriptor: Box<dyn TypeDescriptor>) -> ContentHash {
        self.register_type(descriptor)
    }
    
    fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain> {
        self.compute_migration_path(from, to)
    }
    
    fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult {
        self.validate_schema_compatibility(old, new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock type descriptor for testing
    struct MockTypeDescriptor {
        hash: ContentHash,
    }
    
    impl TypeDescriptor for MockTypeDescriptor {
        fn schema_hash(&self) -> ContentHash {
            self.hash
        }
        
        fn pretty_print(&self, _value: &Value) -> String {
            "mock_value".to_string()
        }
        
        fn structural_equals(&self, _a: &Value, _b: &Value) -> bool {
            true
        }
        
        fn stable_hash(&self, _value: &Value) -> ContentHash {
            self.hash
        }
    }
    
    #[test]
    fn test_type_registry_basic_operations() {
        let mut registry = TypeRegistry::new();
        
        let hash = ContentHash::new(b"test_type");
        let descriptor = Box::new(MockTypeDescriptor { hash });
        
        let registered_hash = registry.register_type(descriptor);
        assert_eq!(registered_hash, hash);
        
        assert!(registry.contains_type(hash));
        assert!(registry.get_type_descriptor(hash).is_some());
    }
    
    #[test]
    fn test_schema_compatibility() {
        let registry = TypeRegistry::new();
        
        let hash1 = ContentHash::new(b"type1");
        let hash2 = ContentHash::new(b"type2");
        
        // Same hash should be fully compatible
        assert_eq!(
            registry.validate_schema_compatibility(hash1, hash1),
            CompatibilityResult::FullyCompatible
        );
        
        // Different hashes without migration should be incompatible
        match registry.validate_schema_compatibility(hash1, hash2) {
            CompatibilityResult::Incompatible(_) => {},
            _ => panic!("Expected incompatible result"),
        }
    }
}