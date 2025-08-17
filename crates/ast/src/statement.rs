//! Statement AST nodes

use crate::{ASTNode, NodeId, Expression};
use mir_types::{ContentHash, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    // Explicit OpenTelemetry statements
    StartSpan {
        id: NodeId,
        name: String,
        attributes: HashMap<String, Value>,
        body: Vec<Statement>,
    },
    EndSpan {
        id: NodeId,
        span: Expression,
    },
    WithSpan {
        id: NodeId,
        span: Expression,
        body: Vec<Statement>,
    },
    RecordMetric {
        id: NodeId,
        metric_type: MetricType,
        name: String,
        value: Expression,
        labels: HashMap<String, Value>,
    },
    LogEvent {
        id: NodeId,
        level: LogLevel,
        message: String,
        attributes: HashMap<String, Value>,
    },
}

/// Types of metrics that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Log levels for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ASTNode for Statement {
    fn node_id(&self) -> NodeId {
        match self {
            Statement::Expression { id, .. } => *id,
            Statement::VariableDeclaration { id, .. } => *id,
            Statement::FunctionDeclaration { id, .. } => *id,
            Statement::StartSpan { id, .. } => *id,
            Statement::EndSpan { id, .. } => *id,
            Statement::WithSpan { id, .. } => *id,
            Statement::RecordMetric { id, .. } => *id,
            Statement::LogEvent { id, .. } => *id,
        }
    }
    
    fn content_hash(&self) -> ContentHash {
        // Placeholder implementation
        ContentHash::new(b"statement")
    }
}