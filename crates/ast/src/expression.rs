//! Expression AST nodes

use crate::{ASTNode, NodeId};
use mir_types::{ContentHash, Value};
use serde::{Deserialize, Serialize};

/// Expression AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal {
        id: NodeId,
        value: Value,
    },
    Identifier {
        id: NodeId,
        name: String,
    },
    FunctionCall {
        id: NodeId,
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

impl ASTNode for Expression {
    fn node_id(&self) -> NodeId {
        match self {
            Expression::Literal { id, .. } => *id,
            Expression::Identifier { id, .. } => *id,
            Expression::FunctionCall { id, .. } => *id,
        }
    }
    
    fn content_hash(&self) -> ContentHash {
        // Placeholder implementation
        ContentHash::new(b"expression")
    }
}