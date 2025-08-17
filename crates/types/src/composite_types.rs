//! Composite type implementations
//! 
//! Provides implementations for composite data structures.

use crate::{Value, TypeHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GC header for managed types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCHeader {
    pub mark: bool,
    pub generation: u32,
}

/// Struct type with named fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructType {
    pub fields: HashMap<String, Value>,
    pub gc_header: GCHeader,
}

/// Struct operations trait
pub trait StructOperations {
    fn get_field(&self, name: &str) -> Option<&Value>;
}

/// Array type with homogeneous elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayType {
    pub elements: Vec<Value>,
    pub element_type: TypeHash,
    pub gc_header: GCHeader,
}

/// Array operations trait
pub trait ArrayOperations {
    fn get(&self, index: usize) -> Option<&Value>;
    fn len(&self) -> usize;
}

/// Record type with heterogeneous fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordType {
    pub fields: Vec<(String, Value)>,
    pub gc_header: GCHeader,
}

/// Record operations trait
pub trait RecordOperations {
    fn get_by_name(&self, name: &str) -> Option<&Value>;
}

/// Union type with RTTI support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionType {
    pub variant_tag: u32,
    pub variant_data: Value,
    pub possible_types: Vec<TypeHash>,
    pub gc_header: GCHeader,
}

/// Union operations trait
pub trait UnionOperations {
    fn get_variant_tag(&self) -> u32;
}

/// Struct errors
#[derive(Debug, thiserror::Error)]
pub enum StructError {
    #[error("Field not found: {0}")]
    FieldNotFound(String),
}

/// Array errors
#[derive(Debug, thiserror::Error)]
pub enum ArrayError {
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}

/// Record errors
#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("Field not found: {0}")]
    FieldNotFound(String),
}

/// Union errors
#[derive(Debug, thiserror::Error)]
pub enum UnionError {
    #[error("Invalid variant: {0}")]
    InvalidVariant(u32),
}