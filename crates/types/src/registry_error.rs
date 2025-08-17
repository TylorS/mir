//! Error types for type registry operations

use crate::{ContentHash, TypeHash};
use std::fmt;

/// Errors that can occur during type registry operations
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRegistryError {
    /// Type with given hash not found
    TypeNotFound(ContentHash),
    /// Type hash not found in registry
    TypeHashNotFound(TypeHash),
    /// Migration path could not be computed
    MigrationPathNotFound { from: ContentHash, to: ContentHash },
    /// Schema is incompatible and cannot be migrated
    IncompatibleSchema { 
        from: ContentHash, 
        to: ContentHash, 
        reason: String 
    },
    /// Internal error during registry operation
    Internal(String),
}

impl fmt::Display for TypeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRegistryError::TypeNotFound(hash) => {
                write!(f, "Type not found for hash: {:?}", hash)
            }
            TypeRegistryError::TypeHashNotFound(type_hash) => {
                write!(f, "Type hash not found: {:?}", type_hash)
            }
            TypeRegistryError::MigrationPathNotFound { from, to } => {
                write!(f, "No migration path from {:?} to {:?}", from, to)
            }
            TypeRegistryError::IncompatibleSchema { from, to, reason } => {
                write!(f, "Incompatible schema migration from {:?} to {:?}: {}", from, to, reason)
            }
            TypeRegistryError::Internal(msg) => {
                write!(f, "Internal registry error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TypeRegistryError {}

/// Result type for type registry operations
pub type TypeRegistryResult<T> = Result<T, TypeRegistryError>;