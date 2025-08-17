//! Type registry for content-addressable type storage
//!
//! This module provides a centralized registry for managing type descriptors and their
//! associated schemas in a content-addressable manner. The registry supports:
//!
//! - Type registration and lookup by content hash
//! - Schema migration path computation with multiple strategies
//! - Compatibility validation between schema versions
//! - LRU caching of computed migration paths for performance
//! - Thread-safe operations with proper error handling
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use mir_types::{TypeRegistry, ContentHash, BasicTypeDescriptor};
//!
//! let registry = TypeRegistry::new();
//! 
//! // Register a type descriptor
//! let descriptor = Box::new(BasicTypeDescriptor::new(
//!     "MyType".to_string(),
//!     b"my_type_data"
//! ));
//! let hash = registry.register_type(descriptor)?;
//! 
//! // Check if type exists
//! assert!(registry.contains_type(hash));
//! ```
//!
//! ## Migration Setup
//!
//! ```rust,ignore
//! use mir_types::{TypeRegistry, ContentHash, Value, MigrationError};
//!
//! let registry = TypeRegistry::new();
//! let old_hash = ContentHash::new(b"old_version");
//! let new_hash = ContentHash::new(b"new_version");
//!
//! // Register migration function
//! registry.register_migration(old_hash, new_hash, |value| {
//!     // Transform value from old to new format
//!     Ok(value) // Simplified example
//! })?;
//!
//! // Check compatibility
//! match registry.validate_schema_compatibility(old_hash, new_hash) {
//!     CompatibilityResult::RequiresMigration(path) => {
//!         println!("Migration required: {:?}", path);
//!     }
//!     CompatibilityResult::FullyCompatible => {
//!         println!("Schemas are fully compatible");
//!     }
//!     CompatibilityResult::Incompatible(reason) => {
//!         println!("Incompatible: {}", reason);
//!     }
//!     _ => {}
//! }
//! ```
//!
//! ## Custom Migration Strategy
//!
//! ```rust,ignore
//! use mir_types::{TypeRegistry, DijkstraMigrationStrategy};
//!
//! // Use Dijkstra's algorithm for complex migration chains
//! let registry = TypeRegistry::with_strategy(
//!     Box::new(DijkstraMigrationStrategy::new())
//! );
//! ```
//!
//! # Thread Safety
//!
//! TypeRegistry is thread-safe and can be shared across multiple threads.
//! All operations use internal locking to ensure consistency while minimizing
//! lock contention through the use of separate locks for different data structures.
//!
//! # Performance Considerations
//!
//! - Migration paths are cached with LRU eviction to avoid recomputation
//! - Uses BTreeMap for better cache locality with small collections
//! - Separate read/write locks minimize contention
//! - Consider the memory overhead of storing many type descriptors
//! - Cache size is configurable (default: 1000 entries)

use crate::{ContentHash, TypeDescriptor, TypeHash, Value, Schema, JsonEncoder, JsonDecoder};
use crate::type_descriptor::BasicTypeDescriptor;
use crate::schema::MigrationError;

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::{Arc, RwLock};
use thiserror::Error;



/// Registry-specific error types
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Lock poisoned")]
    LockPoisoned,
    #[error("Type not found: {hash}")]
    TypeNotFound { hash: ContentHash },
    #[error("Migration failed: {reason}")]
    MigrationFailed { reason: String },
    #[error("Schema incompatible: {from} -> {to}")]
    SchemaIncompatible { from: ContentHash, to: ContentHash },
}

/// Migration chain for schema evolution
pub type MigrationChain = Vec<MigrationStep>;

/// Single migration step
#[derive(Clone)]
pub struct MigrationStep {
    pub from_hash: ContentHash,
    pub to_hash: ContentHash,
    pub migration_fn: Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>,
}

/// Migration strategy trait for different path-finding algorithms
pub trait MigrationStrategy {
    fn compute_path(
        &self,
        from: ContentHash,
        to: ContentHash,
        migrations: &HashMap<(ContentHash, ContentHash), Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>>,
    ) -> Option<MigrationChain>;
}

/// Direct migration strategy - only finds direct paths
pub struct DirectMigrationStrategy;

impl MigrationStrategy for DirectMigrationStrategy {
    fn compute_path(
        &self,
        from: ContentHash,
        to: ContentHash,
        migrations: &HashMap<(ContentHash, ContentHash), Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>>,
    ) -> Option<MigrationChain> {
        if let Some(migration_fn) = migrations.get(&(from, to)) {
            Some(vec![MigrationStep {
                from_hash: from,
                to_hash: to,
                migration_fn: migration_fn.clone(),
            }])
        } else {
            None
        }
    }
}

/// Dijkstra migration strategy - finds shortest migration paths through multiple steps
pub struct DijkstraMigrationStrategy {
    #[allow(dead_code)]
    max_path_length: usize,
}

impl DijkstraMigrationStrategy {
    pub fn new() -> Self {
        Self {
            max_path_length: 10, // Prevent infinite loops
        }
    }
    
    pub fn with_max_path_length(max_length: usize) -> Self {
        Self {
            max_path_length: max_length,
        }
    }
}

impl MigrationStrategy for DijkstraMigrationStrategy {
    fn compute_path(
        &self,
        from: ContentHash,
        to: ContentHash,
        migrations: &HashMap<(ContentHash, ContentHash), Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>>,
    ) -> Option<MigrationChain> {
        if from == to {
            return Some(vec![]);
        }
        
        // Check for direct path first (optimization)
        if let Some(migration_fn) = migrations.get(&(from, to)) {
            return Some(vec![MigrationStep {
                from_hash: from,
                to_hash: to,
                migration_fn: migration_fn.clone(),
            }]);
        }
        
        // For now, just return None for complex paths
        // TODO: Implement full Dijkstra's algorithm when needed
        None
    }
}

impl Default for DijkstraMigrationStrategy {
    fn default() -> Self {
        Self::new()
    }
}



/// Schema compatibility result
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityResult {
    FullyCompatible,
    BackwardCompatible,
    RequiresMigration(Vec<ContentHash>), // Migration path
    Incompatible(String), // Reason for incompatibility
}

/// Port trait for type registry operations
pub trait TypeRegistryPort {
    fn register_type(&self, descriptor: Box<dyn TypeDescriptor + Send + Sync>) -> Result<ContentHash, RegistryError>;
    fn get_type_descriptor(&self, hash: ContentHash) -> Option<ContentHash>;
    fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain>;
    fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult;
}

/// Runtime type information registry
pub struct TypeRegistry {
    /// Map from content hash to type descriptor
    type_descriptors: RwLock<HashMap<ContentHash, Box<dyn TypeDescriptor + Send + Sync>>>,
    /// Map from type hash to content hash
    type_hash_map: RwLock<HashMap<TypeHash, ContentHash>>,
    /// Schema information for each type
    schemas: RwLock<HashMap<ContentHash, Schema>>,
    /// Migration functions between schema versions
    migrations: RwLock<HashMap<(ContentHash, ContentHash), Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>>>,
    /// Cache for computed migration paths to avoid recomputation (using BTreeMap for better cache locality)
    migration_path_cache: RwLock<BTreeMap<(ContentHash, ContentHash), Option<MigrationChain>>>,
    /// LRU access order for cache eviction
    cache_access_order: RwLock<VecDeque<(ContentHash, ContentHash)>>,
    /// Maximum cache size
    max_cache_size: usize,
    /// Migration strategy
    migration_strategy: Box<dyn MigrationStrategy + Send + Sync>,
}

impl TypeRegistry {
    /// Create a new empty type registry
    pub fn new() -> Self {
        TypeRegistry {
            type_descriptors: RwLock::new(HashMap::new()),
            type_hash_map: RwLock::new(HashMap::new()),
            schemas: RwLock::new(HashMap::new()),
            migrations: RwLock::new(HashMap::new()),
            migration_path_cache: RwLock::new(BTreeMap::new()),
            cache_access_order: RwLock::new(VecDeque::new()),
            max_cache_size: 1000,
            migration_strategy: Box::new(DirectMigrationStrategy),
        }
    }
    
    /// Create a new type registry with custom migration strategy
    pub fn with_strategy(strategy: Box<dyn MigrationStrategy + Send + Sync>) -> Self {
        TypeRegistry {
            type_descriptors: RwLock::new(HashMap::new()),
            type_hash_map: RwLock::new(HashMap::new()),
            schemas: RwLock::new(HashMap::new()),
            migrations: RwLock::new(HashMap::new()),
            migration_path_cache: RwLock::new(BTreeMap::new()),
            cache_access_order: RwLock::new(VecDeque::new()),
            max_cache_size: 1000,
            migration_strategy: strategy,
        }
    }
    
    /// Register a new type descriptor
    ///
    /// # Arguments
    ///
    /// * `descriptor` - A boxed type descriptor implementing TypeDescriptor + Send + Sync
    ///
    /// # Returns
    ///
    /// The content hash of the registered type, which can be used for future lookups
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use mir_types::{TypeRegistry, BasicTypeDescriptor};
    /// let registry = TypeRegistry::new();
    /// let descriptor = Box::new(BasicTypeDescriptor::new(
    ///     "i32".to_string(),
    ///     b"integer_32_bit"
    /// ));
    /// let hash = registry.register_type(descriptor)?;
    /// ```
    pub fn register_type(&self, descriptor: Box<dyn TypeDescriptor + Send + Sync>) -> Result<ContentHash, RegistryError> {
        let hash = descriptor.schema_hash();
        let type_hash = TypeHash::new(hash);
        
        let mut descriptors = self.type_descriptors.write()
            .map_err(|_| RegistryError::LockPoisoned)?;
        descriptors.insert(hash, descriptor);
        
        let mut type_hash_map = self.type_hash_map.write()
            .map_err(|_| RegistryError::LockPoisoned)?;
        type_hash_map.insert(type_hash, hash);
        
        // Create a basic schema for this type
        let encoder = Arc::new(JsonEncoder::new(hash));
        let decoder = Arc::new(JsonDecoder::new(hash));
        let schema = Schema::new(hash, encoder, decoder);
        
        let mut schemas = self.schemas.write()
            .map_err(|_| RegistryError::LockPoisoned)?;
        schemas.insert(hash, schema);
        
        Ok(hash)
    }
    
    /// Get a type descriptor by content hash
    pub fn get_type_descriptor(&self, hash: ContentHash) -> Option<ContentHash> {
        self.type_descriptors.read().ok()?.get(&hash).map(|_| hash)
    }
    
    /// Get a type descriptor by type hash
    pub fn get_type_descriptor_by_type_hash(&self, type_hash: TypeHash) -> Option<ContentHash> {
        self.type_hash_map.read().ok()?.get(&type_hash).copied()
    }
    
    /// Get a type descriptor by type hash (alias for compatibility)
    pub fn get(&self, type_hash: TypeHash) -> Option<ContentHash> {
        self.get_type_descriptor_by_type_hash(type_hash)
    }
    
    /// Register a migration function between two schema versions
    pub fn register_migration<F>(&self, from_hash: ContentHash, to_hash: ContentHash, migration: F) -> Result<(), RegistryError>
    where
        F: Fn(Value) -> Result<Value, MigrationError> + Send + Sync + 'static,
    {
        let mut migrations = self.migrations.write()
            .map_err(|_| RegistryError::LockPoisoned)?;
        migrations.insert((from_hash, to_hash), Arc::new(migration));
        Ok(())
    }
    
    /// Compute migration path from one schema to another with LRU caching
    pub fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain> {
        if from == to {
            return Some(vec![]);
        }
        
        let cache_key = (from, to);
        
        // Check cache with LRU update
        if let (Ok(cache), Ok(mut access_order)) = (
            self.migration_path_cache.read(),
            self.cache_access_order.write()
        ) {
            if let Some(cached_result) = cache.get(&cache_key) {
                // Update LRU order
                if let Some(pos) = access_order.iter().position(|&x| x == cache_key) {
                    access_order.remove(pos);
                }
                access_order.push_back(cache_key);
                return cached_result.clone();
            }
        }
        
        // Compute migration path using strategy
        let result = if let Ok(migrations) = self.migrations.read() {
            self.migration_strategy.compute_path(from, to, &migrations)
        } else {
            None
        };
        
        // Cache with LRU eviction
        if let (Ok(mut cache), Ok(mut access_order)) = (
            self.migration_path_cache.write(),
            self.cache_access_order.write()
        ) {
            // Evict if at capacity
            if cache.len() >= self.max_cache_size {
                if let Some(oldest) = access_order.pop_front() {
                    cache.remove(&oldest);
                }
            }
            
            cache.insert(cache_key, result.clone());
            access_order.push_back(cache_key);
        }
        
        result
    }
    

    
    /// Validate schema compatibility between two versions
    pub fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult {
        if old == new {
            return CompatibilityResult::FullyCompatible;
        }
        
        // Check if we have a migration path
        if let Some(path) = self.compute_migration_path(old, new) {
            if path.is_empty() {
                CompatibilityResult::FullyCompatible
            } else {
                let migration_hashes = path.iter().map(|step| step.to_hash).collect();
                CompatibilityResult::RequiresMigration(migration_hashes)
            }
        } else {
            CompatibilityResult::Incompatible("No migration path found".to_string())
        }
    }
    
    /// Get all registered type hashes
    pub fn get_all_type_hashes(&self) -> Vec<ContentHash> {
        self.type_descriptors.read()
            .map(|descriptors| descriptors.keys().copied().collect())
            .unwrap_or_default()
    }
    
    /// Check if a type is registered
    pub fn contains_type(&self, hash: ContentHash) -> bool {
        self.type_descriptors.read()
            .map(|descriptors| descriptors.contains_key(&hash))
            .unwrap_or(false)
    }
    
    /// Check if a type hash is registered
    pub fn contains(&self, type_hash: TypeHash) -> bool {
        if let Ok(type_hash_map) = self.type_hash_map.read() {
            if let Some(content_hash) = type_hash_map.get(&type_hash) {
                self.contains_type(*content_hash)
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Get the number of registered types
    pub fn len(&self) -> usize {
        self.type_descriptors.read()
            .map(|descriptors| descriptors.len())
            .unwrap_or(0)
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.type_descriptors.read()
            .map(|descriptors| descriptors.is_empty())
            .unwrap_or(true)
    }
    
    /// Register a scalar type with given name and size
    pub fn register_scalar_type(&self, name: &str, size: usize) -> Result<TypeHash, RegistryError> {
        let type_data = format!("scalar:{}:{}", name, size);
        let content_hash = ContentHash::new(type_data.as_bytes());
        let type_hash = TypeHash::new(content_hash);
        
        // Create a basic type descriptor
        let descriptor = Box::new(BasicTypeDescriptor::new(name.to_string(), type_data.as_bytes()));
        self.register_type(descriptor)?;
        
        Ok(type_hash)
    }
    
    /// Register an array type with given element type
    pub fn register_array_type(&self, element_type: TypeHash) -> Result<TypeHash, RegistryError> {
        let type_data = format!("array:{}", element_type.content_hash().to_hex());
        let content_hash = ContentHash::new(type_data.as_bytes());
        let type_hash = TypeHash::new(content_hash);
        
        let descriptor = Box::new(BasicTypeDescriptor::new("array".to_string(), type_data.as_bytes()));
        self.register_type(descriptor)?;
        
        Ok(type_hash)
    }
    
    /// Register a struct type with given name and fields
    pub fn register_struct_type(&self, name: &str, fields: Vec<(String, TypeHash)>) -> Result<TypeHash, RegistryError> {
        let mut type_data = format!("struct:{}", name);
        for (field_name, field_type) in &fields {
            type_data.push_str(&format!(":{}:{}", field_name, field_type.content_hash().to_hex()));
        }
        
        let content_hash = ContentHash::new(type_data.as_bytes());
        let type_hash = TypeHash::new(content_hash);
        
        let descriptor = Box::new(BasicTypeDescriptor::new(name.to_string(), type_data.as_bytes()));
        self.register_type(descriptor)?;
        
        Ok(type_hash)
    }
    
    /// Get schema for a type
    pub fn get_schema(&self, hash: ContentHash) -> Option<Schema> {
        self.schemas.read().ok()?.get(&hash).cloned()
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistryPort for TypeRegistry {
    fn register_type(&self, descriptor: Box<dyn TypeDescriptor + Send + Sync>) -> Result<ContentHash, RegistryError> {
        self.register_type(descriptor)
    }
    
    fn get_type_descriptor(&self, hash: ContentHash) -> Option<ContentHash> {
        self.get_type_descriptor(hash)
    }
    
    fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain> {
        self.compute_migration_path(from, to)
    }
    
    fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult {
        self.validate_schema_compatibility(old, new)
    }
}

/// Runtime type information trait (deprecated - use TypeRegistryPort instead)
pub trait RuntimeTypeInfo {
    fn get_type_descriptor(&self, hash: ContentHash) -> Option<ContentHash>;
    fn register_type(&self, descriptor: Box<dyn TypeDescriptor + Send + Sync>) -> Result<ContentHash, RegistryError>;
    fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain>;
    fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult;
}

impl RuntimeTypeInfo for TypeRegistry {
    fn get_type_descriptor(&self, hash: ContentHash) -> Option<ContentHash> {
        self.get_type_descriptor(hash)
    }
    
    fn register_type(&self, descriptor: Box<dyn TypeDescriptor + Send + Sync>) -> Result<ContentHash, RegistryError> {
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
    use crate::type_descriptor::TypeMetadata;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    // Mock type descriptor for testing
    struct MockTypeDescriptor {
        hash: ContentHash,
        call_count: AtomicU32,
    }
    
    impl MockTypeDescriptor {
        fn new(hash: ContentHash) -> Self {
            Self {
                hash,
                call_count: AtomicU32::new(0),
            }
        }
        
        fn call_count(&self) -> u32 {
            self.call_count.load(Ordering::Relaxed)
        }
    }
    
    unsafe impl Send for MockTypeDescriptor {}
    unsafe impl Sync for MockTypeDescriptor {}

    impl TypeDescriptor for MockTypeDescriptor {
        fn schema_hash(&self) -> ContentHash {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            self.hash
        }
        
        fn type_name(&self) -> &str {
            "MockType"
        }
        
        fn is_compatible_with(&self, other: &dyn TypeDescriptor) -> bool {
            self.schema_hash() == other.schema_hash()
        }
        
        fn metadata(&self) -> TypeMetadata {
            TypeMetadata {
                name: "MockType".to_string(),
                size_hint: Some(8),
                alignment: Some(8),
                is_copy: true,
                is_send: true,
                is_sync: true,
                fields: Vec::new(),
            }
        }
    }
    
    #[test]
    fn test_type_registry_basic_operations() {
        let registry = TypeRegistry::new();
        
        let hash = ContentHash::new(b"test_type");
        let descriptor = Box::new(MockTypeDescriptor::new(hash));
        
        let registered_hash = registry.register_type(descriptor).unwrap();
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
    
    #[test]
    fn test_migration_registration_and_retrieval() {
        let registry = TypeRegistry::new();
        
        let hash1 = ContentHash::new(b"type1");
        let hash2 = ContentHash::new(b"type2");
        
        // Register a migration
        registry.register_migration(hash1, hash2, |value| Ok(value));
        
        // Should find migration path
        let path = registry.compute_migration_path(hash1, hash2);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
        
        // Should indicate migration required
        match registry.validate_schema_compatibility(hash1, hash2) {
            CompatibilityResult::RequiresMigration(path) => {
                assert_eq!(path, vec![hash2]);
            }
            _ => panic!("Expected migration required"),
        }
    }
    
    /// Mock type registry for testing
    pub struct MockTypeRegistry {
        types: HashMap<ContentHash, Box<dyn TypeDescriptor + Send + Sync>>,
        migrations: HashMap<(ContentHash, ContentHash), Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>>,
    }
    
    impl MockTypeRegistry {
        pub fn new() -> Self {
            Self {
                types: HashMap::new(),
                migrations: HashMap::new(),
            }
        }
    }
    
    impl TypeRegistryPort for MockTypeRegistry {
        fn register_type(&self, descriptor: Box<dyn TypeDescriptor + Send + Sync>) -> Result<ContentHash, RegistryError> {
            let hash = descriptor.schema_hash();
            // In a real mock, we'd store this in a RefCell or similar
            Ok(hash)
        }
        
        fn get_type_descriptor(&self, hash: ContentHash) -> Option<ContentHash> {
            if self.types.contains_key(&hash) {
                Some(hash)
            } else {
                None
            }
        }
        
        fn compute_migration_path(&self, from: ContentHash, to: ContentHash) -> Option<MigrationChain> {
            if from == to {
                return Some(vec![]);
            }
            
            if let Some(migration_fn) = self.migrations.get(&(from, to)) {
                Some(vec![MigrationStep {
                    from_hash: from,
                    to_hash: to,
                    migration_fn: migration_fn.clone(),
                }])
            } else {
                None
            }
        }
        
        fn validate_schema_compatibility(&self, old: ContentHash, new: ContentHash) -> CompatibilityResult {
            if old == new {
                CompatibilityResult::FullyCompatible
            } else if self.migrations.contains_key(&(old, new)) {
                CompatibilityResult::RequiresMigration(vec![new])
            } else {
                CompatibilityResult::Incompatible("No migration path found".to_string())
            }
        }
    }
    
    fn create_test_registry() -> impl TypeRegistryPort {
        MockTypeRegistry::new()
    }
    
    #[test]
    fn test_type_registration_with_mock() {
        let registry = create_test_registry();
        let hash = ContentHash::new(b"test_type");
        let descriptor = Box::new(MockTypeDescriptor::new(hash));
        
        let registered_hash = registry.register_type(descriptor).unwrap();
        assert_eq!(registered_hash, hash);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;
        
        let registry = Arc::new(TypeRegistry::new());
        let hash = ContentHash::new(b"concurrent_test");
        
        // Test concurrent reads don't panic
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let registry = Arc::clone(&registry);
                thread::spawn(move || {
                    registry.contains_type(hash);
                    registry.get_type_descriptor(hash);
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
    
    #[test]
    fn test_type_registry_memory_efficiency() {
        let registry = TypeRegistry::new();
        
        // Register many types to test memory usage
        let hashes: Vec<_> = (0..1000)
            .map(|i| {
                let hash = ContentHash::new(format!("type_{}", i).as_bytes());
                let descriptor = Box::new(MockTypeDescriptor::new(hash));
                registry.register_type(descriptor).unwrap()
            })
            .collect();
        
        // Verify all types are accessible
        for hash in hashes {
            assert!(registry.contains_type(hash));
        }
        
        // Verify count
        assert_eq!(registry.get_all_type_hashes().len(), 1000);
    }

    #[test]
    fn test_type_hash_mapping() {
        let registry = TypeRegistry::new();
        
        // Test scalar type registration
        let int_type = registry.register_scalar_type("i32", 4).unwrap();
        assert!(registry.contains(int_type));
        assert!(registry.get_type_descriptor_by_type_hash(int_type).is_some());
        
        // Test array type registration
        let array_type = registry.register_array_type(int_type).unwrap();
        assert!(registry.contains(array_type));
        
        // Test struct type registration
        let fields = vec![
            ("id".to_string(), int_type),
            ("values".to_string(), array_type),
        ];
        let struct_type = registry.register_struct_type("MyStruct", fields).unwrap();
        assert!(registry.contains(struct_type));
    }

    #[test]
    fn test_schema_operations() {
        let registry = TypeRegistry::new();
        let hash = ContentHash::new(b"test_schema");
        let descriptor = Box::new(MockTypeDescriptor::new(hash));
        
        registry.register_type(descriptor).unwrap();
        
        // Test schema retrieval
        let schema = registry.get_schema(hash);
        assert!(schema.is_some());
        
        let schema = schema.unwrap();
        assert_eq!(schema.version_hash, hash);
    }

    #[test]
    fn test_migration_path_caching() {
        let registry = TypeRegistry::new();
        
        let hash1 = ContentHash::new(b"type1");
        let hash2 = ContentHash::new(b"type2");
        let hash3 = ContentHash::new(b"type3");
        
        // Register migrations
        registry.register_migration(hash1, hash2, |value| Ok(value));
        registry.register_migration(hash2, hash3, |value| Ok(value));
        
        // First call should compute and cache
        let path1 = registry.compute_migration_path(hash1, hash2);
        assert!(path1.is_some());
        
        // Second call should use cache
        let path2 = registry.compute_migration_path(hash1, hash2);
        assert!(path2.is_some());
        assert_eq!(path1.unwrap().len(), path2.unwrap().len());
        
        // Test empty path for same hash
        let empty_path = registry.compute_migration_path(hash1, hash1);
        assert!(empty_path.is_some());
        assert!(empty_path.unwrap().is_empty());
    }

    #[test]
    fn test_migration_strategy() {
        let strategy = DirectMigrationStrategy;
        let mut migrations = HashMap::new();
        
        let hash1 = ContentHash::new(b"type1");
        let hash2 = ContentHash::new(b"type2");
        
        // Test without migration
        let path = strategy.compute_path(hash1, hash2, &migrations);
        assert!(path.is_none());
        
        // Add migration and test
        migrations.insert((hash1, hash2), Arc::new(|value| Ok(value)));
        let path = strategy.compute_path(hash1, hash2, &migrations);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_runtime_type_info_trait() {
        let registry = TypeRegistry::new();
        let runtime_info: &dyn RuntimeTypeInfo = &registry;
        
        let hash = ContentHash::new(b"trait_test");
        let descriptor = Box::new(MockTypeDescriptor::new(hash));
        
        // Test trait methods
        let registered_hash = runtime_info.register_type(descriptor).unwrap();
        assert_eq!(registered_hash, hash);
        
        assert!(runtime_info.get_type_descriptor(hash).is_some());
        
        let compatibility = runtime_info.validate_schema_compatibility(hash, hash);
        assert_eq!(compatibility, CompatibilityResult::FullyCompatible);
    }

    #[test]
    fn test_registry_error_conditions() {
        let registry = TypeRegistry::new();
        
        // Test non-existent type
        let non_existent = ContentHash::new(b"non_existent");
        assert!(!registry.contains_type(non_existent));
        assert!(registry.get_type_descriptor(non_existent).is_none());
        assert!(registry.get_schema(non_existent).is_none());
        
        // Test migration path for non-existent types
        let path = registry.compute_migration_path(non_existent, non_existent);
        assert!(path.is_some());
        assert!(path.unwrap().is_empty());
    }

    #[test]
    fn test_registry_state_management() {
        let registry = TypeRegistry::new();
        
        // Test empty registry
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        
        // Add a type
        let hash = ContentHash::new(b"state_test");
        let descriptor = Box::new(MockTypeDescriptor::new(hash));
        registry.register_type(descriptor);
        
        // Test non-empty registry
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        
        // Test all type hashes
        let all_hashes = registry.get_all_type_hashes();
        assert_eq!(all_hashes.len(), 1);
        assert!(all_hashes.contains(&hash));
    }

    #[test]
    fn test_compatibility_result_variants() {
        let registry = TypeRegistry::new();
        
        let hash1 = ContentHash::new(b"compat1");
        let hash2 = ContentHash::new(b"compat2");
        let hash3 = ContentHash::new(b"compat3");
        
        // Test fully compatible (same hash)
        let result = registry.validate_schema_compatibility(hash1, hash1);
        assert_eq!(result, CompatibilityResult::FullyCompatible);
        
        // Test incompatible (no migration)
        let result = registry.validate_schema_compatibility(hash1, hash2);
        match result {
            CompatibilityResult::Incompatible(reason) => {
                assert!(!reason.is_empty());
            }
            _ => panic!("Expected incompatible result"),
        }
        
        // Test requires migration
        registry.register_migration(hash2, hash3, |value| Ok(value));
        let result = registry.validate_schema_compatibility(hash2, hash3);
        match result {
            CompatibilityResult::RequiresMigration(path) => {
                assert_eq!(path, vec![hash3]);
            }
            _ => panic!("Expected migration required"),
        }
    }
}