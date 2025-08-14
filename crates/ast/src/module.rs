//! Module definitions

use crate::{ASTNode, NodeId, Statement};
use mir_types::ContentHash;
use serde::{Deserialize, Serialize};

/// Module representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: NodeId,
    pub name: String,
    pub imports: Vec<ImportDeclaration>,
    pub exports: Vec<ExportDeclaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclaration {
    pub module_path: String,
    pub imported_items: Vec<ImportItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub is_type: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDeclaration {
    pub name: String,
    pub is_type: bool,
}

impl ASTNode for Module {
    fn node_id(&self) -> NodeId {
        self.id
    }
    
    fn content_hash(&self) -> ContentHash {
        // Placeholder implementation
        ContentHash::new(self.name.as_bytes())
    }
}