//! Function type implementations
//! 
//! Provides implementations for function types, closures, and continuations.

use crate::{Value, TypeHash, ContentHash, UniversalValueOperations};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Function type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionType {
    pub signature: FunctionSignature,
    pub implementation: FunctionImplementation,
}

/// Function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub parameter_types: Vec<TypeHash>,
    pub return_type: TypeHash,
    pub is_pure: bool,
    pub is_async: bool,
}

/// Function implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionImplementation {
    Native,
    IR,
    FFI,
}

/// Function ID
pub type FunctionId = u64;

/// Capture environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEnvironment {
    pub captures: HashMap<String, CapturedValue>,
}

/// Captured value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedValue {
    pub value: Value,
    pub mode: CaptureMode,
}

/// Capture mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureMode {
    ByValue,
    ByReference,
    ByMutableReference,
}

/// Calling convention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallingConvention {
    Standard,
    FastCall,
    SystemV,
}

/// Function operations trait
pub trait FunctionOperations {
    fn call(&self, args: &[Value]) -> Result<Value, FunctionError>;
}

/// Function metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub name: String,
    pub source_location: Option<SourceLocation>,
}

/// Source map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub mappings: Vec<SourceMapping>,
}

/// Source mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub source_line: u32,
    pub source_column: u32,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Closure type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureType {
    pub function: FunctionType,
    pub captured_values: HashMap<String, Value>,
    pub capture_mode: CaptureMode,
}

/// Closure operations trait
pub trait ClosureOperations {
    fn call(&self, args: &[Value]) -> Result<Value, FunctionError>;
}

/// Continuation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationType {
    pub continuation_id: ContinuationId,
    pub prompt_tag: PromptTag,
}

/// Continuation ID
pub type ContinuationId = u64;

/// Prompt tag
pub type PromptTag = u64;

/// Stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_id: FunctionId,
    pub locals: HashMap<String, Value>,
}

/// Continuation operations trait
pub trait ContinuationOperations {
    fn resume(&self, value: Value) -> Result<Value, ContinuationError>;
}

/// Prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub tag: PromptTag,
    pub handler: FunctionId,
}

/// Control operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlOperators {
    Shift,
    Reset,
    Call,
}

/// Function errors
#[derive(Debug, thiserror::Error)]
pub enum FunctionError {
    #[error("Invalid arguments")]
    InvalidArguments,
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Continuation errors
#[derive(Debug, thiserror::Error)]
pub enum ContinuationError {
    #[error("Invalid continuation")]
    InvalidContinuation,
    #[error("Continuation already consumed")]
    AlreadyConsumed,
}

// Helper function to create type hash from bytes
#[allow(dead_code)]
fn create_type_hash_from_bytes(bytes: &[u8]) -> TypeHash {
    TypeHash::new(ContentHash::new(bytes))
}

impl FunctionType {
    /// Check if this function is callable with the given argument types
    pub fn is_callable_with(&self, arg_types: &[TypeHash]) -> bool {
        if arg_types.len() != self.signature.parameter_types.len() {
            return false;
        }
        
        for (expected, actual) in self.signature.parameter_types.iter().zip(arg_types.iter()) {
            if expected != actual {
                return false;
            }
        }
        
        true
    }
}

impl ClosureType {
    /// Check if this closure is callable with the given argument types
    pub fn is_callable_with(&self, args: &[Value]) -> bool {
        let arg_types: Vec<TypeHash> = args.iter().map(|arg| arg.type_hash()).collect();
        self.function.is_callable_with(&arg_types)
    }
}