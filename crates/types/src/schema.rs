//! Schema definitions and serialization support

use crate::{ContentHash, Value, SerializationError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub type SchemaVersion = u32;

#[derive(Debug, Clone)]
pub struct EncodingContext {
    pub version: SchemaVersion,
    pub compression: Option<CompressionType>,
}

#[derive(Debug, Clone)]
pub struct DecodingContext {
    pub expected_version: SchemaVersion,
    pub allow_version_mismatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
}

pub trait Encoder: Send + Sync {
    fn encode(&self, value: &Value) -> Result<Vec<u8>, SerializationError>;
    fn encode_with_context(&self, value: &Value, context: &EncodingContext) -> Result<Vec<u8>, SerializationError>;
}

pub trait Decoder: Send + Sync {
    fn decode(&self, data: &[u8]) -> Result<Value, SerializationError>;
    fn decode_with_context(&self, data: &[u8], context: &DecodingContext) -> Result<Value, SerializationError>;
}

pub struct JsonEncoder {
    #[allow(dead_code)]
    schema_hash: ContentHash,
}

impl JsonEncoder {
    pub fn new(schema_hash: ContentHash) -> Self {
        Self { schema_hash }
    }
}

impl Encoder for JsonEncoder {
    fn encode(&self, value: &Value) -> Result<Vec<u8>, SerializationError> {
        serde_json::to_vec(value)
            .map_err(|e| SerializationError::InvalidData(e.to_string()))
    }

    fn encode_with_context(&self, value: &Value, _context: &EncodingContext) -> Result<Vec<u8>, SerializationError> {
        self.encode(value)
    }
}

pub struct JsonDecoder {
    #[allow(dead_code)]
    schema_hash: ContentHash,
}

impl JsonDecoder {
    pub fn new(schema_hash: ContentHash) -> Self {
        Self { schema_hash }
    }
}

impl Decoder for JsonDecoder {
    fn decode(&self, data: &[u8]) -> Result<Value, SerializationError> {
        serde_json::from_slice(data)
            .map_err(|e| SerializationError::InvalidData(e.to_string()))
    }

    fn decode_with_context(&self, data: &[u8], context: &DecodingContext) -> Result<Value, SerializationError> {
        if context.expected_version != 1 && !context.allow_version_mismatch {
            return Err(SerializationError::VersionMismatch {
                expected: context.expected_version,
                actual: 1,
            });
        }
        
        self.decode(data)
    }
}

#[derive(Clone)]
pub struct Schema {
    pub version_hash: ContentHash,
    pub encoder: Arc<dyn Encoder>,
    pub decoder: Arc<dyn Decoder>,
    pub migration_functions: HashMap<ContentHash, MigrationFunction>,
}

impl Schema {
    pub fn new(version_hash: ContentHash, encoder: Arc<dyn Encoder>, decoder: Arc<dyn Decoder>) -> Self {
        Self {
            version_hash,
            encoder,
            decoder,
            migration_functions: HashMap::new(),
        }
    }
}

pub type MigrationFunction = Arc<dyn Fn(Value) -> Result<Value, MigrationError> + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Incompatible schema versions")]
    IncompatibleVersions,
    
    #[error("Missing migration function")]
    MissingMigration,
}

pub type MigrationChain = Vec<MigrationStep>;

#[derive(Clone)]
pub struct MigrationStep {
    pub from_hash: ContentHash,
    pub to_hash: ContentHash,
    pub migration_fn: MigrationFunction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityResult {
    FullyCompatible,
    BackwardCompatible,
    RequiresMigration(Vec<ContentHash>),
    Incompatible(String),
}

pub struct SchemaRegistry {
    schemas: HashMap<ContentHash, Schema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register(&mut self, schema: Schema) -> ContentHash {
        let hash = schema.version_hash;
        self.schemas.insert(hash, schema);
        hash
    }

    pub fn get(&self, hash: ContentHash) -> Option<&Schema> {
        self.schemas.get(&hash)
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}