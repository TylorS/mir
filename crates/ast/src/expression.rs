//! Expression AST nodes

use crate::{ASTNode, NodeId};
use mir_types::{ContentHash, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    // Explicit OpenTelemetry operations
    CreateSpan {
        id: NodeId,
        name: String,
        attributes: HashMap<String, Value>,
        parent_span: Option<Box<Expression>>,
    },
    SetSpanAttribute {
        id: NodeId,
        span: Box<Expression>,
        key: String,
        value: Value,
    },
    AddSpanEvent {
        id: NodeId,
        span: Box<Expression>,
        name: String,
        attributes: HashMap<String, Value>,
    },
    GetTraceContext {
        id: NodeId,
    },
    SetTraceContext {
        id: NodeId,
        context: Box<Expression>,
    },
}

impl ASTNode for Expression {
    fn node_id(&self) -> NodeId {
        match self {
            Expression::Literal { id, .. } => *id,
            Expression::Identifier { id, .. } => *id,
            Expression::FunctionCall { id, .. } => *id,
            Expression::CreateSpan { id, .. } => *id,
            Expression::SetSpanAttribute { id, .. } => *id,
            Expression::AddSpanEvent { id, .. } => *id,
            Expression::GetTraceContext { id } => *id,
            Expression::SetTraceContext { id, .. } => *id,
        }
    }

    fn content_hash(&self) -> ContentHash {
        // Placeholder implementation
        ContentHash::new(b"expression")
    }
}
