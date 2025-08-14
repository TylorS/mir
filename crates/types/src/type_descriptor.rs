//! Type descriptor definitions for the MIR type system

use crate::{ContentHash, Value};
use serde::{Deserialize, Serialize};

/// Core type descriptor trait
pub trait TypeDescriptor {
    fn schema_hash(&self) -> ContentHash;
    fn pretty_print(&self, value: &Value) -> String;
    fn structural_equals(&self, a: &Value, b: &Value) -> bool;
    fn stable_hash(&self, value: &Value) -> ContentHash;
}

/// Type hash identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeHash(ContentHash);

impl TypeHash {
    pub fn new(hash: ContentHash) -> Self {
        TypeHash(hash)
    }
    
    pub fn as_content_hash(&self) -> ContentHash {
        self.0
    }
    
    pub fn hash(&self) -> ContentHash {
        self.0
    }
}

impl std::fmt::Display for TypeHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}