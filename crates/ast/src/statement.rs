//! Statement AST nodes

use crate::{ASTNode, NodeId, Expression};
use mir_types::ContentHash;
use serde::{Deserialize, Serialize};

/// Statement AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Expression {
        id: NodeId,
        expression: Expression,
    },
    VariableDeclaration {
        id: NodeId,
        name: String,
        value: Option<Expression>,
    },
    FunctionDeclaration {
        id: NodeId,
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
}

impl ASTNode for Statement {
    fn node_id(&self) -> NodeId {
        match self {
            Statement::Expression { id, .. } => *id,
            Statement::VariableDeclaration { id, .. } => *id,
            Statement::FunctionDeclaration { id, .. } => *id,
        }
    }
    
    fn content_hash(&self) -> ContentHash {
        // Placeholder implementation
        ContentHash::new(b"statement")
    }
}