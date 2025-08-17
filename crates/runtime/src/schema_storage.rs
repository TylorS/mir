//! Schema-aware storage that integrates with the type system

use mir_types::{ContentHash, Value, Schema, SchemaVersion, UniversalValueOperations};
use crate::storage::{ContentAddressableStore, StorageStats};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Schema-aware storage that handles versioned data
pub struct SchemaAwareStore<S: ContentAddressableStore> {
    /// Underlying content-addressable storage
    storage: S,
    /// Schema registry for type information
    schemas: HashMap<ContentHash, Schema>,
    /// Version mapping for schema evolution
    version_map: HashMap<ContentHash, Vec<ContentHash>>, // old -> [new versions]
    /// Reverse version mapping
    reverse_version_map: HashMap<ContentHash, Vec<ContentHash>>, // new -> [old versions]
    /// Cached migration paths
    migration_cache: HashMap<(ContentHash, ContentHash), MigrationPath>,
}

/// Migration path between schema versions
#[derive(Debug, Clone)]
pub struct MigrationPath {
    pub steps: Vec<MigrationStep>,
    pub is_lossy: bool,
}

/// Single migration step
#[derive(Debug, Clone)]
pub struct MigrationStep {
    pub from_schema: ContentHash,
    pub to_schema: ContentHash,
    pub migration_type: MigrationType,
}

/// Type of migration required
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationType {
    /// No migration needed (fully compatible)
    None,
    /// Automatic migration (backward compatible)
    Automatic,
    /// Custom migration function required
    Custom,
    /// Lossy migration (data may be lost)
    Lossy,
}

/// Versioned value with schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedValue {
    pub value: Value,
    pub schema_hash: ContentHash,
    pub schema_version: SchemaVersion,
    pub stored_at: u64,
    pub metadata: ValueMetadata,
}

/// Metadata for stored values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueMetadata {
    pub content_type: String,
    pub compression: Option<CompressionType>,
    pub checksum: ContentHash,
    pub tags: HashMap<String, String>,
}

/// Compression types for stored values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// Storage result with version information
#[derive(Debug, Clone)]
pub struct StorageResult {
    pub content_hash: ContentHash,
    pub schema_hash: ContentHash,
    pub was_deduplicated: bool,
    pub migration_applied: Option<MigrationPath>,
}

/// Query parameters for retrieving versioned data
#[derive(Debug, Clone)]
pub struct QueryParams {
    pub target_schema: Option<ContentHash>,
    pub allow_migration: bool,
    pub prefer_latest: bool,
    pub include_metadata: bool,
}

impl<S: ContentAddressableStore> SchemaAwareStore<S> {
    /// Create a new schema-aware store
    pub fn new(storage: S) -> Self {
        SchemaAwareStore {
            storage,
            schemas: HashMap::new(),
            version_map: HashMap::new(),
            reverse_version_map: HashMap::new(),
            migration_cache: HashMap::new(),
        }
    }
    
    /// Register a schema with the store
    pub fn register_schema(&mut self, schema: Schema) -> ContentHash {
        let hash = schema.version_hash;
        self.schemas.insert(hash, schema);
        hash
    }
    
    /// Register a schema evolution path
    pub fn register_schema_evolution(&mut self, old_schema: ContentHash, new_schema: ContentHash) {
        self.version_map.entry(old_schema).or_default().push(new_schema);
        self.reverse_version_map.entry(new_schema).or_default().push(old_schema);
        
        // Clear migration cache for affected schemas
        self.migration_cache.retain(|(from, to), _| {
            *from != old_schema && *to != old_schema && *from != new_schema && *to != new_schema
        });
    }
    
    /// Store a value with schema information
    pub fn store_value(&mut self, value: &Value, schema_hash: ContentHash) -> Result<StorageResult, StorageError> {
        // Get schema for validation
        let _schema = self.schemas.get(&schema_hash)
            .ok_or(StorageError::SchemaNotFound(schema_hash))?;
        
        // Create versioned value
        let versioned_value = VersionedValue {
            value: value.clone(),
            schema_hash,
            schema_version: 1, // Use version 1 as default
            stored_at: current_timestamp(),
            metadata: ValueMetadata {
                content_type: "application/mir-value".to_string(),
                compression: None,
                checksum: value.stable_hash(),
                tags: HashMap::new(),
            },
        };
        
        // Serialize the versioned value
        let serialized = serde_json::to_vec(&versioned_value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        // Store in underlying storage
        let content_hash = self.storage.store(&serialized);
        
        // Check for deduplication (content existed before this store operation)
        let was_deduplicated = false; // For now, assume no deduplication
        
        Ok(StorageResult {
            content_hash,
            schema_hash,
            was_deduplicated,
            migration_applied: None,
        })
    }
    
    /// Retrieve a value with optional schema migration
    pub fn retrieve_value(&self, content_hash: ContentHash, params: QueryParams) -> Result<Option<VersionedValue>, StorageError> {
        // Retrieve from underlying storage
        let data = match self.storage.retrieve(content_hash) {
            Some(data) => data,
            None => return Ok(None),
        };
        
        // Deserialize versioned value
        let mut versioned_value: VersionedValue = serde_json::from_slice(&data)
            .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
        
        // Apply migration if requested and needed
        if let Some(target_schema) = params.target_schema {
            if versioned_value.schema_hash != target_schema && params.allow_migration {
                versioned_value = self.migrate_value(versioned_value, target_schema)?;
            }
        }
        
        Ok(Some(versioned_value))
    }
    
    /// Migrate a value to a different schema version
    fn migrate_value(&self, mut value: VersionedValue, target_schema: ContentHash) -> Result<VersionedValue, StorageError> {
        if value.schema_hash == target_schema {
            return Ok(value);
        }
        
        // Find migration path
        let migration_path = self.find_migration_path(value.schema_hash, target_schema)?;
        
        // Apply migration steps
        for step in &migration_path.steps {
            value = self.apply_migration_step(value, step)?;
        }
        
        Ok(value)
    }
    
    /// Find migration path between two schemas
    fn find_migration_path(&self, from: ContentHash, to: ContentHash) -> Result<MigrationPath, StorageError> {
        // Check cache first
        if let Some(cached_path) = self.migration_cache.get(&(from, to)) {
            return Ok(cached_path.clone());
        }
        
        // Simple direct path for now
        // TODO: Implement proper pathfinding algorithm for complex migration chains
        if let Some(versions) = self.version_map.get(&from) {
            if versions.contains(&to) {
                let path = MigrationPath {
                    steps: vec![MigrationStep {
                        from_schema: from,
                        to_schema: to,
                        migration_type: MigrationType::Automatic,
                    }],
                    is_lossy: false,
                };
                return Ok(path);
            }
        }
        
        Err(StorageError::MigrationPathNotFound(from, to))
    }
    
    /// Apply a single migration step
    fn apply_migration_step(&self, mut value: VersionedValue, step: &MigrationStep) -> Result<VersionedValue, StorageError> {
        match step.migration_type {
            MigrationType::None => Ok(value),
            MigrationType::Automatic => {
                // For automatic migration, just update the schema hash
                // In a real implementation, this would apply schema-specific transformations
                value.schema_hash = step.to_schema;
                value.schema_version = 1; // Use version 1 as default
                Ok(value)
            },
            MigrationType::Custom => {
                // TODO: Apply custom migration function
                Err(StorageError::MigrationError("Custom migrations not implemented".to_string()))
            },
            MigrationType::Lossy => {
                // TODO: Apply lossy migration with warnings
                Err(StorageError::MigrationError("Lossy migrations not implemented".to_string()))
            },
        }
    }
    
    /// Get all values with a specific schema
    pub fn query_by_schema(&self, schema_hash: ContentHash) -> Result<Vec<(ContentHash, VersionedValue)>, StorageError> {
        let mut results = Vec::new();
        
        // This is inefficient - in a real implementation, we'd maintain an index
        for content_hash in self.storage.list_hashes() {
            if let Some(data) = self.storage.retrieve(content_hash) {
                if let Ok(versioned_value) = serde_json::from_slice::<VersionedValue>(&data) {
                    if versioned_value.schema_hash == schema_hash {
                        results.push((content_hash, versioned_value));
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Perform schema-aware garbage collection
    pub fn schema_aware_gc(&mut self, reachable_schemas: &HashSet<ContentHash>) -> Result<GCResult, StorageError> {
        let mut reachable_content = HashSet::new();
        let mut migrated_count = 0;
        let mut removed_count = 0;
        
        // Find all content that uses reachable schemas
        for content_hash in self.storage.list_hashes() {
            if let Some(data) = self.storage.retrieve(content_hash) {
                if let Ok(versioned_value) = serde_json::from_slice::<VersionedValue>(&data) {
                    if reachable_schemas.contains(&versioned_value.schema_hash) {
                        reachable_content.insert(content_hash);
                    } else {
                        // Try to migrate to a reachable schema
                        let mut migrated = false;
                        for &reachable_schema in reachable_schemas {
                            if self.find_migration_path(versioned_value.schema_hash, reachable_schema).is_ok() {
                                // Migration is possible, keep the content
                                reachable_content.insert(content_hash);
                                migrated = true;
                                migrated_count += 1;
                                break;
                            }
                        }
                        if !migrated {
                            removed_count += 1;
                        }
                    }
                }
            }
        }
        
        // Perform garbage collection
        self.storage.garbage_collect(&reachable_content);
        
        Ok(GCResult {
            migrated_count,
            removed_count,
            total_reachable: reachable_content.len(),
        })
    }
    
    /// Get storage statistics including schema information
    pub fn schema_stats(&self) -> SchemaStats {
        let base_stats = self.storage.stats();
        let mut schema_counts = HashMap::new();
        let mut total_values = 0;
        
        // Count values by schema
        for content_hash in self.storage.list_hashes() {
            if let Some(data) = self.storage.retrieve(content_hash) {
                if let Ok(versioned_value) = serde_json::from_slice::<VersionedValue>(&data) {
                    *schema_counts.entry(versioned_value.schema_hash).or_insert(0) += 1;
                    total_values += 1;
                }
            }
        }
        
        SchemaStats {
            base_stats,
            registered_schemas: self.schemas.len(),
            schema_counts,
            total_values,
            migration_cache_size: self.migration_cache.len(),
        }
    }
    
    /// Get underlying storage reference
    pub fn storage(&self) -> &S {
        &self.storage
    }
    
    /// Get mutable underlying storage reference
    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

/// Schema-aware storage statistics
#[derive(Debug, Clone)]
pub struct SchemaStats {
    pub base_stats: StorageStats,
    pub registered_schemas: usize,
    pub schema_counts: HashMap<ContentHash, usize>,
    pub total_values: usize,
    pub migration_cache_size: usize,
}

/// Garbage collection result
#[derive(Debug, Clone)]
pub struct GCResult {
    pub migrated_count: usize,
    pub removed_count: usize,
    pub total_reachable: usize,
}

/// Storage errors
#[derive(Debug, Clone)]
pub enum StorageError {
    SchemaNotFound(ContentHash),
    SerializationError(String),
    DeserializationError(String),
    MigrationPathNotFound(ContentHash, ContentHash),
    MigrationError(String),
    InvalidValue(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::SchemaNotFound(hash) => write!(f, "Schema not found: {hash}"),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            StorageError::DeserializationError(msg) => write!(f, "Deserialization error: {msg}"),
            StorageError::MigrationPathNotFound(from, to) => write!(f, "Migration path not found: {from} -> {to}"),
            StorageError::MigrationError(msg) => write!(f, "Migration error: {msg}"),
            StorageError::InvalidValue(msg) => write!(f, "Invalid value: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStore;
    use mir_types::{JsonEncoder, JsonDecoder};
    use std::sync::Arc;

    fn create_test_schema(id: u8) -> Schema {
        let hash = ContentHash::new(&[id]);
        let encoder = Arc::new(JsonEncoder::new(hash));
        let decoder = Arc::new(JsonDecoder::new(hash));
        Schema::new(hash, encoder, decoder)
    }

    fn create_test_value() -> Value {
        // Create a simple test value
        Value::String("test_value".to_string())
    }

    #[test]
    fn test_schema_aware_storage() {
        let storage = InMemoryStore::new();
        let mut schema_store = SchemaAwareStore::new(storage);
        
        // Register a schema
        let schema = create_test_schema(1);
        let schema_hash = schema_store.register_schema(schema);
        
        // Store a value
        let value = create_test_value();
        let result = schema_store.store_value(&value, schema_hash).unwrap();
        
        assert_eq!(result.schema_hash, schema_hash);
        assert!(!result.was_deduplicated);
        
        // Retrieve the value
        let params = QueryParams {
            target_schema: None,
            allow_migration: false,
            prefer_latest: false,
            include_metadata: true,
        };
        
        let retrieved = schema_store.retrieve_value(result.content_hash, params).unwrap();
        assert!(retrieved.is_some());
        
        let versioned_value = retrieved.unwrap();
        assert_eq!(versioned_value.schema_hash, schema_hash);
        assert_eq!(versioned_value.value, value);
    }
    
    #[test]
    fn test_schema_evolution() {
        let storage = InMemoryStore::new();
        let mut schema_store = SchemaAwareStore::new(storage);
        
        // Register two schema versions
        let schema_v1 = create_test_schema(1);
        let schema_v2 = create_test_schema(2);
        
        let hash_v1 = schema_store.register_schema(schema_v1);
        let hash_v2 = schema_store.register_schema(schema_v2);
        
        // Register evolution path
        schema_store.register_schema_evolution(hash_v1, hash_v2);
        
        // Store value with v1 schema
        let value = create_test_value();
        let result = schema_store.store_value(&value, hash_v1).unwrap();
        
        // Retrieve with v2 schema (should migrate)
        let params = QueryParams {
            target_schema: Some(hash_v2),
            allow_migration: true,
            prefer_latest: false,
            include_metadata: true,
        };
        
        let retrieved = schema_store.retrieve_value(result.content_hash, params).unwrap();
        assert!(retrieved.is_some());
        
        let versioned_value = retrieved.unwrap();
        assert_eq!(versioned_value.schema_hash, hash_v2);
    }
    
    #[test]
    fn test_schema_aware_gc() {
        let storage = InMemoryStore::new();
        let mut schema_store = SchemaAwareStore::new(storage);
        
        // Register schemas
        let schema1 = create_test_schema(1);
        let schema2 = create_test_schema(2);
        
        let hash1 = schema_store.register_schema(schema1);
        let hash2 = schema_store.register_schema(schema2);
        
        // Store values with different schemas
        let value = create_test_value();
        schema_store.store_value(&value, hash1).unwrap();
        schema_store.store_value(&value, hash2).unwrap();
        
        // GC keeping only schema1
        let mut reachable = HashSet::new();
        reachable.insert(hash1);
        
        let gc_result = schema_store.schema_aware_gc(&reachable).unwrap();
        
        // Should have removed content with schema2
        assert_eq!(gc_result.total_reachable, 1);
    }
}