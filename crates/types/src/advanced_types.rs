//! Advanced type implementations
//! 
//! Provides implementations for advanced types like enums, regex, and resources.

use crate::{Value, TypeHash, ContentHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enum type with Rust-like variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumType {
    pub variant_name: String,
    pub variant_data: Option<Value>,
    pub enum_definition: EnumDefinition,
}

/// Enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: HashMap<String, VariantDefinition>,
    pub schema_hash: ContentHash,
}

/// Variant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDefinition {
    pub tag: u32,
    pub data_type: Option<TypeHash>,
    pub discriminant: Option<i64>,
}

/// Enum operations trait
pub trait EnumOperations {
    fn get_variant_name(&self) -> &str;
}

/// Regular expression type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexType {
    pub pattern: String,
    pub flags: RegexFlags,
}

/// Regex flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexFlags {
    pub case_insensitive: bool,
    pub multiline: bool,
    pub dot_matches_newline: bool,
    pub unicode: bool,
}

/// Regex operations trait
pub trait RegexOperations {
    fn matches(&self, text: &str) -> bool;
}

/// Match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Resource type for external resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceType {
    pub resource_id: ResourceId,
    pub resource_type: ResourceTypeDefinition,
    pub handle: ResourceHandle,
}

/// Resource ID
pub type ResourceId = u64;

/// Resource type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTypeDefinition {
    pub name: String,
    pub capabilities: Vec<String>,
}

/// Resource handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHandle {
    pub id: u64,
    pub is_valid: bool,
}

/// Resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub name: String,
    pub size: Option<u64>,
    pub created_at: u64,
}

/// Resource operations trait
pub trait ResourceOperations {
    fn is_acquired(&self) -> bool;
}

/// Finalizer function type
pub type FinalizerFunction = Box<dyn Fn() + Send + Sync>;

/// Enum errors
#[derive(Debug, thiserror::Error)]
pub enum EnumError {
    #[error("Invalid variant: {0}")]
    InvalidVariant(String),
}

/// Regex errors
#[derive(Debug, thiserror::Error)]
pub enum RegexError {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Resource errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(ResourceId),
    #[error("Resource already acquired")]
    AlreadyAcquired,
}