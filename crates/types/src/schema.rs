//! Schema definitions for versioned data structures

use crate::{ContentHash, Value, TypeHash, SerializationError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Schema version identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaVersion(pub ContentHash);

impl SchemaVersion {
    pub fn new(hash: ContentHash) -> Self {
        SchemaVersion(hash)
    }

    pub fn as_content_hash(&self) -> ContentHash {
        self.0
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Encoder trait for serializing values according to a schema
pub trait Encoder: Send + Sync {
    fn encode(&self, value: &Value) -> Result<Vec<u8>, SerializationError>;
    fn encode_with_context(&self, value: &Value, context: &EncodingContext) -> Result<Vec<u8>, SerializationError>;
    fn schema_hash(&self) -> ContentHash;
}

/// Decoder trait for deserializing values according to a schema
pub trait Decoder: Send + Sync {
    fn decode(&self, data: &[u8]) -> Result<Value, SerializationError>;
    fn decode_with_context(&self, data: &[u8], context: &DecodingContext) -> Result<Value, SerializationError>;
    fn schema_hash(&self) -> ContentHash;
}

/// Encoding context for additional serialization information
#[derive(Debug, Clone)]
pub struct EncodingContext {
    pub compression: Option<CompressionType>,
    pub include_metadata: bool,
    pub custom_attributes: HashMap<String, String>,
}

/// Decoding context for additional deserialization information
#[derive(Debug, Clone)]
pub struct DecodingContext {
    pub expected_schema: Option<ContentHash>,
    pub allow_migration: bool,
    pub custom_attributes: HashMap<String, String>,
}

/// Compression types for encoded data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// Migration function type
pub type MigrationFunction = Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>;

/// Migration error
#[derive(Debug, Clone)]
pub struct MigrationError {
    pub message: String,
    pub source_version: Option<SchemaVersion>,
    pub target_version: Option<SchemaVersion>,
    pub value_type: Option<TypeHash>,
}

/// Migration chain for multi-step migrations
#[derive(Clone)]
pub struct MigrationChain {
    pub steps: Vec<MigrationStep>,
    pub total_cost: u32,
}

/// Single migration step
#[derive(Clone)]
pub struct MigrationStep {
    pub from_version: SchemaVersion,
    pub to_version: SchemaVersion,
    pub migration_function: MigrationFunction,
    pub cost: u32,
}

impl std::fmt::Debug for MigrationChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationChain")
            .field("steps", &format!("{} steps", self.steps.len()))
            .field("total_cost", &self.total_cost)
            .finish()
    }
}

impl std::fmt::Debug for MigrationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationStep")
            .field("from_version", &self.from_version)
            .field("to_version", &self.to_version)
            .field("migration_function", &"<function>")
            .field("cost", &self.cost)
            .finish()
    }
}

impl PartialEq for MigrationChain {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost == other.total_cost && self.steps.len() == other.steps.len()
    }
}

impl Eq for MigrationChain {}

impl PartialEq for MigrationStep {
    fn eq(&self, other: &Self) -> bool {
        self.from_version == other.from_version 
            && self.to_version == other.to_version 
            && self.cost == other.cost
    }
}

impl Eq for MigrationStep {}

/// Schema compatibility levels
#[derive(Debug, Clone)]
pub enum CompatibilityResult {
    FullyCompatible,
    BackwardCompatible,
    ForwardCompatible,
    RequiresMigration(MigrationChain),
    Incompatible(String),
}

impl PartialEq for CompatibilityResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CompatibilityResult::FullyCompatible, CompatibilityResult::FullyCompatible) => true,
            (CompatibilityResult::BackwardCompatible, CompatibilityResult::BackwardCompatible) => true,
            (CompatibilityResult::ForwardCompatible, CompatibilityResult::ForwardCompatible) => true,
            (CompatibilityResult::RequiresMigration(a), CompatibilityResult::RequiresMigration(b)) => a == b,
            (CompatibilityResult::Incompatible(a), CompatibilityResult::Incompatible(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for CompatibilityResult {}

/// Schema with encoder/decoder pairs and versioning
pub struct Schema {
    pub version_hash: ContentHash,
    pub encoder: Arc<dyn Encoder>,
    pub decoder: Arc<dyn Decoder>,
    pub migration_functions: HashMap<SchemaVersion, MigrationFunction>,
    pub compatibility_rules: HashMap<SchemaVersion, CompatibilityResult>,
}

impl Schema {
    pub fn new(
        version_hash: ContentHash,
        encoder: Arc<dyn Encoder>,
        decoder: Arc<dyn Decoder>,
    ) -> Self {
        Schema {
            version_hash,
            encoder,
            decoder,
            migration_functions: HashMap::new(),
            compatibility_rules: HashMap::new(),
        }
    }

    /// Add a migration function from a specific version
    pub fn add_migration(&mut self, from_version: SchemaVersion, migration: MigrationFunction) {
        self.migration_functions.insert(from_version, migration);
    }

    /// Add compatibility rule for a specific version
    pub fn add_compatibility_rule(&mut self, version: SchemaVersion, result: CompatibilityResult) {
        self.compatibility_rules.insert(version, result);
    }

    /// Get the current schema version
    pub fn version(&self) -> SchemaVersion {
        SchemaVersion::new(self.version_hash)
    }

    /// Encode a value using this schema
    pub fn encode(&self, value: &Value) -> Result<Vec<u8>, SerializationError> {
        self.encoder.encode(value)
    }

    /// Decode data using this schema
    pub fn decode(&self, data: &[u8]) -> Result<Value, SerializationError> {
        self.decoder.decode(data)
    }

    /// Encode with context
    pub fn encode_with_context(&self, value: &Value, context: &EncodingContext) -> Result<Vec<u8>, SerializationError> {
        self.encoder.encode_with_context(value, context)
    }

    /// Decode with context
    pub fn decode_with_context(&self, data: &[u8], context: &DecodingContext) -> Result<Value, SerializationError> {
        self.decoder.decode_with_context(data, context)
    }
}

/// Schema registry for managing multiple schema versions
pub struct SchemaRegistry {
    schemas: HashMap<SchemaVersion, Arc<Schema>>,
    migration_graph: HashMap<SchemaVersion, Vec<(SchemaVersion, u32)>>, // version -> [(target, cost)]
}

impl SchemaRegistry {
    pub fn new() -> Self {
        SchemaRegistry {
            schemas: HashMap::new(),
            migration_graph: HashMap::new(),
        }
    }

    /// Register a new schema version
    pub fn register_schema(&mut self, schema: Schema) -> SchemaVersion {
        let version = schema.version();
        let schema_arc = Arc::new(schema);
        self.schemas.insert(version, schema_arc);
        version
    }

    /// Get a schema by version
    pub fn get_schema(&self, version: SchemaVersion) -> Option<Arc<Schema>> {
        self.schemas.get(&version).cloned()
    }

    /// Add a migration edge between two schema versions
    pub fn add_migration_edge(&mut self, from: SchemaVersion, to: SchemaVersion, cost: u32) {
        self.migration_graph
            .entry(from)
            .or_insert_with(Vec::new)
            .push((to, cost));
    }

    /// Check compatibility between two schema versions
    pub fn check_compatibility(&self, from: SchemaVersion, to: SchemaVersion) -> CompatibilityResult {
        if from == to {
            return CompatibilityResult::FullyCompatible;
        }

        // Check if we have a direct compatibility rule
        if let Some(schema) = self.schemas.get(&to) {
            if let Some(result) = schema.compatibility_rules.get(&from) {
                return result.clone();
            }
        }

        // Try to compute a migration path
        match self.compute_migration_path(from, to) {
            Some(chain) => CompatibilityResult::RequiresMigration(chain),
            None => CompatibilityResult::Incompatible(
                format!("No migration path from {} to {}", from, to)
            ),
        }
    }

    /// Compute automatic migration path between schema versions
    pub fn compute_migration_path(&self, from: SchemaVersion, to: SchemaVersion) -> Option<MigrationChain> {
        if from == to {
            return Some(MigrationChain {
                steps: Vec::new(),
                total_cost: 0,
            });
        }

        // Use Dijkstra's algorithm to find the shortest migration path
        let mut distances: HashMap<SchemaVersion, u32> = HashMap::new();
        let mut previous: HashMap<SchemaVersion, SchemaVersion> = HashMap::new();
        let mut queue = VecDeque::new();

        distances.insert(from, 0);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                break;
            }

            let current_distance = distances[&current];

            if let Some(neighbors) = self.migration_graph.get(&current) {
                for &(neighbor, edge_cost) in neighbors {
                    let new_distance = current_distance + edge_cost;
                    
                    if !distances.contains_key(&neighbor) || new_distance < distances[&neighbor] {
                        distances.insert(neighbor, new_distance);
                        previous.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // Reconstruct the path
        if !distances.contains_key(&to) {
            return None;
        }

        let mut path = Vec::new();
        let mut current = to;
        
        while let Some(&prev) = previous.get(&current) {
            // Find the migration function
            if let Some(schema) = self.schemas.get(&current) {
                if let Some(migration_fn) = schema.migration_functions.get(&prev) {
                    path.push(MigrationStep {
                        from_version: prev,
                        to_version: current,
                        migration_function: migration_fn.clone(),
                        cost: self.get_edge_cost(prev, current).unwrap_or(1),
                    });
                }
            }
            current = prev;
        }

        path.reverse();

        Some(MigrationChain {
            total_cost: distances[&to],
            steps: path,
        })
    }

    /// Get the cost of migrating between two adjacent schema versions
    fn get_edge_cost(&self, from: SchemaVersion, to: SchemaVersion) -> Option<u32> {
        self.migration_graph
            .get(&from)?
            .iter()
            .find(|(version, _)| *version == to)
            .map(|(_, cost)| *cost)
    }

    /// Apply a migration chain to transform a value
    pub fn apply_migration_chain(&self, value: Value, chain: &MigrationChain) -> Result<Value, MigrationError> {
        let mut current_value = value;
        
        for step in &chain.steps {
            current_value = (step.migration_function)(current_value)?;
        }
        
        Ok(current_value)
    }

    /// Migrate a value from one schema version to another
    pub fn migrate_value(&self, value: Value, from: SchemaVersion, to: SchemaVersion) -> Result<Value, MigrationError> {
        match self.compute_migration_path(from, to) {
            Some(chain) => self.apply_migration_chain(value, &chain),
            None => Err(MigrationError {
                message: format!("No migration path from {} to {}", from, to),
                source_version: Some(from),
                target_version: Some(to),
                value_type: None,
            }),
        }
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Migration error: {}", self.message)?;
        if let (Some(source), Some(target)) = (&self.source_version, &self.target_version) {
            write!(f, " (from {} to {})", source, target)?;
        }
        Ok(())
    }
}

impl std::error::Error for MigrationError {}
/// Default JSON encoder implementation
pub struct JsonEncoder {
    schema_hash: ContentHash,
}

impl JsonEncoder {
    pub fn new(schema_hash: ContentHash) -> Self {
        JsonEncoder { schema_hash }
    }
}

impl Encoder for JsonEncoder {
    fn encode(&self, value: &Value) -> Result<Vec<u8>, SerializationError> {
        serde_json::to_vec(value)
            .map_err(|e| SerializationError::InvalidData(e.to_string()))
    }

    fn encode_with_context(&self, value: &Value, context: &EncodingContext) -> Result<Vec<u8>, SerializationError> {
        let mut data = self.encode(value)?;
        
        // Apply compression if requested
        if let Some(compression) = &context.compression {
            data = match compression {
                CompressionType::None => data,
                CompressionType::Gzip => {
                    // Placeholder for gzip compression
                    // In a real implementation, you'd use a compression library
                    data
                },
                CompressionType::Lz4 => {
                    // Placeholder for lz4 compression
                    data
                },
                CompressionType::Zstd => {
                    // Placeholder for zstd compression
                    data
                },
            };
        }
        
        Ok(data)
    }

    fn schema_hash(&self) -> ContentHash {
        self.schema_hash
    }
}

/// Default JSON decoder implementation
pub struct JsonDecoder {
    schema_hash: ContentHash,
}

impl JsonDecoder {
    pub fn new(schema_hash: ContentHash) -> Self {
        JsonDecoder { schema_hash }
    }
}

impl Decoder for JsonDecoder {
    fn decode(&self, data: &[u8]) -> Result<Value, SerializationError> {
        serde_json::from_slice(data)
            .map_err(|e| SerializationError::InvalidData(e.to_string()))
    }

    fn decode_with_context(&self, data: &[u8], context: &DecodingContext) -> Result<Value, SerializationError> {
        // Handle decompression if needed (placeholder)
        let decompressed_data = data;
        
        let value = self.decode(decompressed_data)?;
        
        // Validate schema if expected
        if let Some(expected_schema) = context.expected_schema {
            if expected_schema != self.schema_hash {
                return Err(SerializationError::VersionMismatch {
                    expected: crate::SchemaVersion(expected_schema),
                    found: crate::SchemaVersion(self.schema_hash),
                });
            }
        }
        
        Ok(value)
    }

    fn schema_hash(&self) -> ContentHash {
        self.schema_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UniversalValueOperations;
    use std::sync::Arc;

    #[test]
    fn test_schema_creation() {
        let schema_hash = ContentHash::new(b"test_schema_v1");
        let encoder = Arc::new(JsonEncoder::new(schema_hash));
        let decoder = Arc::new(JsonDecoder::new(schema_hash));
        
        let schema = Schema::new(schema_hash, encoder, decoder);
        assert_eq!(schema.version().as_content_hash(), schema_hash);
    }

    #[test]
    fn test_schema_registry() {
        let mut registry = SchemaRegistry::new();
        
        // Create two schema versions
        let v1_hash = ContentHash::new(b"schema_v1");
        let v2_hash = ContentHash::new(b"schema_v2");
        
        let v1_encoder = Arc::new(JsonEncoder::new(v1_hash));
        let v1_decoder = Arc::new(JsonDecoder::new(v1_hash));
        let schema_v1 = Schema::new(v1_hash, v1_encoder, v1_decoder);
        
        let v2_encoder = Arc::new(JsonEncoder::new(v2_hash));
        let v2_decoder = Arc::new(JsonDecoder::new(v2_hash));
        let schema_v2 = Schema::new(v2_hash, v2_encoder, v2_decoder);
        
        let v1 = registry.register_schema(schema_v1);
        let v2 = registry.register_schema(schema_v2);
        
        // Test schema retrieval
        assert!(registry.get_schema(v1).is_some());
        assert!(registry.get_schema(v2).is_some());
        
        // Test compatibility (should be incompatible without migration)
        match registry.check_compatibility(v1, v2) {
            CompatibilityResult::Incompatible(_) => {}, // Expected
            other => panic!("Expected incompatible, got {:?}", other),
        }
    }

    #[test]
    fn test_migration_path_computation() {
        let mut registry = SchemaRegistry::new();
        
        // Create three schema versions: v1 -> v2 -> v3
        let v1_hash = ContentHash::new(b"schema_v1");
        let v2_hash = ContentHash::new(b"schema_v2");
        let v3_hash = ContentHash::new(b"schema_v3");
        
        // Create schemas with migration functions
        let v1_encoder = Arc::new(JsonEncoder::new(v1_hash));
        let v1_decoder = Arc::new(JsonDecoder::new(v1_hash));
        let mut schema_v1 = Schema::new(v1_hash, v1_encoder, v1_decoder);
        
        let v2_encoder = Arc::new(JsonEncoder::new(v2_hash));
        let v2_decoder = Arc::new(JsonDecoder::new(v2_hash));
        let mut schema_v2 = Schema::new(v2_hash, v2_encoder, v2_decoder);
        
        let v3_encoder = Arc::new(JsonEncoder::new(v3_hash));
        let v3_decoder = Arc::new(JsonDecoder::new(v3_hash));
        let mut schema_v3 = Schema::new(v3_hash, v3_encoder, v3_decoder);
        
        // Add migration functions
        let v1_to_v2_migration = Arc::new(|value: Value| -> Result<Value, MigrationError> {
            Ok(value) // Identity migration for test
        });
        let v2_to_v3_migration = Arc::new(|value: Value| -> Result<Value, MigrationError> {
            Ok(value) // Identity migration for test
        });
        
        schema_v2.add_migration(SchemaVersion::new(v1_hash), v1_to_v2_migration);
        schema_v3.add_migration(SchemaVersion::new(v2_hash), v2_to_v3_migration);
        
        let v1 = registry.register_schema(schema_v1);
        let v2 = registry.register_schema(schema_v2);
        let v3 = registry.register_schema(schema_v3);
        
        // Add migration edges
        registry.add_migration_edge(v1, v2, 1);
        registry.add_migration_edge(v2, v3, 1);
        
        // Test direct migration
        let path = registry.compute_migration_path(v1, v2);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.steps.len(), 1);
        assert_eq!(path.total_cost, 1);
        
        // Test multi-step migration
        let path = registry.compute_migration_path(v1, v3);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.steps.len(), 2);
        assert_eq!(path.total_cost, 2);
        
        // Test no path
        let v4 = SchemaVersion::new(ContentHash::new(b"schema_v4"));
        let path = registry.compute_migration_path(v1, v4);
        assert!(path.is_none());
    }

    #[test]
    fn test_json_encoder_decoder() {
        let schema_hash = ContentHash::new(b"test_json_schema");
        let encoder = JsonEncoder::new(schema_hash);
        let decoder = JsonDecoder::new(schema_hash);
        
        let value = Value::String("test value".to_string());
        
        // Test encoding
        let encoded = encoder.encode(&value).unwrap();
        assert!(!encoded.is_empty());
        
        // Test decoding
        let decoded = decoder.decode(&encoded).unwrap();
        assert!(value.structural_equals(&decoded));
    }

    #[test]
    fn test_encoding_context() {
        let schema_hash = ContentHash::new(b"test_context_schema");
        let encoder = JsonEncoder::new(schema_hash);
        
        let value = Value::I32(42);
        let context = EncodingContext {
            compression: Some(CompressionType::None),
            include_metadata: true,
            custom_attributes: HashMap::new(),
        };
        
        let encoded = encoder.encode_with_context(&value, &context).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_decoding_context() {
        let schema_hash = ContentHash::new(b"test_decode_context");
        let decoder = JsonDecoder::new(schema_hash);
        
        let value = Value::Bool(true);
        let encoded = serde_json::to_vec(&value).unwrap();
        
        let context = DecodingContext {
            expected_schema: Some(schema_hash),
            allow_migration: false,
            custom_attributes: HashMap::new(),
        };
        
        let decoded = decoder.decode_with_context(&encoded, &context).unwrap();
        assert!(value.structural_equals(&decoded));
    }

    #[test]
    fn test_schema_version_mismatch() {
        let schema_hash1 = ContentHash::new(b"schema1");
        let schema_hash2 = ContentHash::new(b"schema2");
        let decoder = JsonDecoder::new(schema_hash1);
        
        let value = Value::String("test".to_string());
        let encoded = serde_json::to_vec(&value).unwrap();
        
        let context = DecodingContext {
            expected_schema: Some(schema_hash2), // Different schema
            allow_migration: false,
            custom_attributes: HashMap::new(),
        };
        
        let result = decoder.decode_with_context(&encoded, &context);
        assert!(matches!(result, Err(SerializationError::VersionMismatch { .. })));
    }
}