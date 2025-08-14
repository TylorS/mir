//! Core AST node definitions

use mir_types::{ContentHash};
use serde::{Deserialize, Serialize};

/// AST node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }
}

/// Base AST node trait
pub trait ASTNode {
    fn node_id(&self) -> NodeId;
    fn content_hash(&self) -> ContentHash;
}