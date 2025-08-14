//! Function types and closures for the MIR type system

use crate::{TypeHash, Value, GCHeader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionId(u64);

impl FunctionId {
    pub fn new(id: u64) -> Self {
        FunctionId(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Function signature with parameter and return types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub parameter_types: Vec<TypeHash>,
    pub return_type: TypeHash,
    pub is_pure: bool,
    pub is_async: bool,
}

impl FunctionSignature {
    pub fn new(
        parameter_types: Vec<TypeHash>,
        return_type: TypeHash,
        is_pure: bool,
        is_async: bool,
    ) -> Self {
        Self {
            parameter_types,
            return_type,
            is_pure,
            is_async,
        }
    }
    
    pub fn parameter_count(&self) -> usize {
        self.parameter_types.len()
    }
    
    pub fn is_compatible_with(&self, arg_types: &[TypeHash]) -> bool {
        self.parameter_types.len() == arg_types.len() &&
        self.parameter_types.iter().zip(arg_types.iter()).all(|(expected, actual)| expected == actual)
    }
}

/// Function implementation variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionImplementation {
    /// Native Rust function
    Native {
        name: String,
        // Function pointer would be stored separately in runtime
    },
    /// MIR IR function
    IR {
        instructions: Vec<u8>, // Placeholder for IR instructions
        locals: Vec<TypeHash>,
    },
    /// Foreign function interface binding
    FFI {
        module_name: String,
        function_name: String,
        calling_convention: CallingConvention,
    },
}

/// Calling conventions for FFI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallingConvention {
    C,
    Stdcall,
    Fastcall,
    Vectorcall,
}

/// First-class function type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionType {
    pub id: FunctionId,
    pub signature: FunctionSignature,
    pub implementation: FunctionImplementation,
    pub captures: CaptureEnvironment,
    pub gc_header: GCHeader,
    pub auto_span: bool, // Automatic OpenTelemetry spanning
    pub source_map: Option<SourceMap>,
}

impl FunctionType {
    pub fn new(
        id: FunctionId,
        signature: FunctionSignature,
        implementation: FunctionImplementation,
    ) -> Self {
        Self {
            id,
            signature,
            implementation,
            captures: CaptureEnvironment::new(),
            gc_header: GCHeader::new(std::mem::size_of::<FunctionType>()),
            auto_span: false,
            source_map: None,
        }
    }
    
    pub fn with_captures(mut self, captures: CaptureEnvironment) -> Self {
        self.captures = captures;
        self
    }
    
    pub fn with_auto_span(mut self, auto_span: bool) -> Self {
        self.auto_span = auto_span;
        self
    }
    
    pub fn with_source_map(mut self, source_map: SourceMap) -> Self {
        self.source_map = Some(source_map);
        self
    }
}

/// Capture environment for closures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureEnvironment {
    captured_values: HashMap<String, CapturedValue>,
}

impl CaptureEnvironment {
    pub fn new() -> Self {
        Self {
            captured_values: HashMap::new(),
        }
    }
    
    pub fn capture(&mut self, name: String, value: Value, mode: CaptureMode) {
        self.captured_values.insert(name, CapturedValue { value, mode });
    }
    
    pub fn get_captured(&self, name: &str) -> Option<&CapturedValue> {
        self.captured_values.get(name)
    }
    
    pub fn get_captured_mut(&mut self, name: &str) -> Option<&mut CapturedValue> {
        self.captured_values.get_mut(name)
    }
    
    pub fn captured_names(&self) -> impl Iterator<Item = &String> {
        self.captured_values.keys()
    }
    
    pub fn is_empty(&self) -> bool {
        self.captured_values.is_empty()
    }
}

impl Default for CaptureEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Captured value with its capture mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedValue {
    pub value: Value,
    pub mode: CaptureMode,
}

/// Capture modes for closures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureMode {
    /// Capture by value (move semantics)
    ByValue,
    /// Capture by immutable reference
    ByReference,
    /// Capture by mutable reference
    ByMutableReference,
}

/// Source map information for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceMap {
    pub file_path: String,
    pub line_mappings: Vec<LineMapping>,
}

/// Line mapping for source maps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineMapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub source_line: u32,
    pub source_column: u32,
}

/// Function metadata for runtime introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub id: FunctionId,
    pub name: Option<String>,
    pub signature: FunctionSignature,
    pub is_closure: bool,
    pub capture_count: usize,
    pub source_location: Option<SourceLocation>,
}

/// Source location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Function operations trait
pub trait FunctionOperations {
    fn call(&self, args: &[Value]) -> Result<Value, FunctionError>;
    fn partial_apply(&self, args: &[Value]) -> Result<FunctionType, FunctionError>;
    fn get_signature(&self) -> &FunctionSignature;
    fn is_callable_with(&self, arg_types: &[TypeHash]) -> bool;
    fn compose(&self, other: &FunctionType) -> Result<FunctionType, FunctionError>;
}

/// Function-related errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FunctionError {
    /// Wrong number of arguments provided
    ArityMismatch {
        expected: usize,
        provided: usize,
    },
    /// Argument type mismatch
    TypeMismatch {
        parameter_index: usize,
        expected: TypeHash,
        provided: TypeHash,
    },
    /// Runtime execution error
    ExecutionError(String),
    /// Capture environment error
    CaptureError(String),
    /// Composition error
    CompositionError(String),
    /// FFI call error
    FFIError(String),
}

impl fmt::Display for FunctionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionError::ArityMismatch { expected, provided } => {
                write!(f, "Arity mismatch: expected {} arguments, got {}", expected, provided)
            }
            FunctionError::TypeMismatch { parameter_index, expected, provided } => {
                write!(f, "Type mismatch at parameter {}: expected {:?}, got {:?}", 
                       parameter_index, expected, provided)
            }
            FunctionError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            FunctionError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            FunctionError::CompositionError(msg) => write!(f, "Composition error: {}", msg),
            FunctionError::FFIError(msg) => write!(f, "FFI error: {}", msg),
        }
    }
}

impl std::error::Error for FunctionError {}

/// Closure type with capture environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureType {
    pub function: FunctionType,
    pub captured_values: HashMap<String, Value>,
    pub capture_mode: CaptureMode,
    pub gc_header: GCHeader,
}

impl ClosureType {
    pub fn new(function: FunctionType, capture_mode: CaptureMode) -> Self {
        Self {
            function,
            captured_values: HashMap::new(),
            capture_mode,
            gc_header: GCHeader::new(std::mem::size_of::<ClosureType>()),
        }
    }
    
    pub fn with_captures(mut self, captures: HashMap<String, Value>) -> Self {
        self.captured_values = captures;
        self
    }
    
    pub fn capture(&mut self, name: String, value: Value) {
        self.captured_values.insert(name, value);
    }
    
    pub fn get_captured(&self, name: &str) -> Option<&Value> {
        self.captured_values.get(name)
    }
    
    pub fn get_captured_mut(&mut self, name: &str) -> Option<&mut Value> {
        if matches!(self.capture_mode, CaptureMode::ByMutableReference) {
            self.captured_values.get_mut(name)
        } else {
            None
        }
    }
    
    pub fn captured_names(&self) -> impl Iterator<Item = &String> {
        self.captured_values.keys()
    }
    
    pub fn capture_count(&self) -> usize {
        self.captured_values.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.captured_values.is_empty()
    }
}

/// Closure operations trait
pub trait ClosureOperations {
    fn call(&self, args: &[Value]) -> Result<Value, FunctionError>;
    fn get_captured(&self, name: &str) -> Option<&Value>;
    fn update_captured(&mut self, name: &str, value: Value) -> Result<(), FunctionError>;
    fn clone_with_captures(&self, new_captures: HashMap<String, Value>) -> Self;
    fn get_function(&self) -> &FunctionType;
    fn get_capture_mode(&self) -> CaptureMode;
}

impl ClosureOperations for ClosureType {
    fn call(&self, args: &[Value]) -> Result<Value, FunctionError> {
        // Validate argument count and types using the underlying function
        if args.len() != self.function.signature.parameter_count() {
            return Err(FunctionError::ArityMismatch {
                expected: self.function.signature.parameter_count(),
                provided: args.len(),
            });
        }
        
        // Validate argument types
        for (i, (expected_type, arg)) in self.function.signature.parameter_types.iter().zip(args.iter()).enumerate() {
            let actual_type = arg.type_hash();
            if *expected_type != actual_type {
                return Err(FunctionError::TypeMismatch {
                    parameter_index: i,
                    expected: *expected_type,
                    provided: actual_type,
                });
            }
        }
        
        // Execute closure with access to captured values
        match &self.function.implementation {
            FunctionImplementation::Native { name } => {
                // In a real implementation, this would execute the native function
                // with access to captured values
                Err(FunctionError::ExecutionError(format!(
                    "Native closure '{}' execution not implemented (captures: {})", 
                    name, 
                    self.capture_count()
                )))
            }
            FunctionImplementation::IR { instructions: _, locals: _ } => {
                // In a real implementation, this would execute the IR instructions
                // with captured values available in the execution context
                Err(FunctionError::ExecutionError(format!(
                    "IR closure execution not implemented (captures: {})", 
                    self.capture_count()
                )))
            }
            FunctionImplementation::FFI { module_name, function_name, calling_convention: _ } => {
                // FFI closures would need special handling for captured values
                Err(FunctionError::FFIError(format!(
                    "FFI closure {}::{} not implemented (captures: {})", 
                    module_name, 
                    function_name,
                    self.capture_count()
                )))
            }
        }
    }
    
    fn get_captured(&self, name: &str) -> Option<&Value> {
        self.captured_values.get(name)
    }
    
    fn update_captured(&mut self, name: &str, value: Value) -> Result<(), FunctionError> {
        match self.capture_mode {
            CaptureMode::ByValue => {
                // Cannot update captured values when captured by value
                Err(FunctionError::CaptureError(
                    "Cannot update captured value when captured by value".to_string()
                ))
            }
            CaptureMode::ByReference => {
                // Cannot update captured values when captured by immutable reference
                Err(FunctionError::CaptureError(
                    "Cannot update captured value when captured by immutable reference".to_string()
                ))
            }
            CaptureMode::ByMutableReference => {
                // Can update when captured by mutable reference
                if self.captured_values.contains_key(name) {
                    self.captured_values.insert(name.to_string(), value);
                    Ok(())
                } else {
                    Err(FunctionError::CaptureError(format!(
                        "Captured value '{}' not found", name
                    )))
                }
            }
        }
    }
    
    fn clone_with_captures(&self, new_captures: HashMap<String, Value>) -> Self {
        ClosureType {
            function: self.function.clone(),
            captured_values: new_captures,
            capture_mode: self.capture_mode.clone(),
            gc_header: GCHeader::new(std::mem::size_of::<ClosureType>()),
        }
    }
    
    fn get_function(&self) -> &FunctionType {
        &self.function
    }
    
    fn get_capture_mode(&self) -> CaptureMode {
        self.capture_mode.clone()
    }
}

/// Unique identifier for continuations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContinuationId(u64);

impl ContinuationId {
    pub fn new(id: u64) -> Self {
        ContinuationId(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Prompt tag for delimiting continuations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromptTag {
    id: u64,
    name: Option<String>,
}

impl PromptTag {
    pub fn new(id: u64) -> Self {
        PromptTag { id, name: None }
    }
    
    pub fn with_name(id: u64, name: String) -> Self {
        PromptTag { id, name: Some(name) }
    }
    
    pub fn id(&self) -> u64 {
        self.id
    }
    
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Stack frame for continuation capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_id: FunctionId,
    pub instruction_pointer: usize,
    pub local_variables: HashMap<String, Value>,
    pub operand_stack: Vec<Value>,
}

impl StackFrame {
    pub fn new(function_id: FunctionId, instruction_pointer: usize) -> Self {
        Self {
            function_id,
            instruction_pointer,
            local_variables: HashMap::new(),
            operand_stack: Vec::new(),
        }
    }
    
    pub fn with_locals(mut self, locals: HashMap<String, Value>) -> Self {
        self.local_variables = locals;
        self
    }
    
    pub fn with_operands(mut self, operands: Vec<Value>) -> Self {
        self.operand_stack = operands;
        self
    }
    
    pub fn set_local(&mut self, name: String, value: Value) {
        self.local_variables.insert(name, value);
    }
    
    pub fn get_local(&self, name: &str) -> Option<&Value> {
        self.local_variables.get(name)
    }
    
    pub fn push_operand(&mut self, value: Value) {
        self.operand_stack.push(value);
    }
    
    pub fn pop_operand(&mut self) -> Option<Value> {
        self.operand_stack.pop()
    }
}

/// Continuation type for delimited continuations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationType {
    pub continuation_id: ContinuationId,
    pub stack_frames: Vec<StackFrame>,
    pub prompt_tag: PromptTag,
    pub is_delimited: bool,
    pub gc_header: GCHeader,
}

impl ContinuationType {
    pub fn new(
        continuation_id: ContinuationId,
        stack_frames: Vec<StackFrame>,
        prompt_tag: PromptTag,
    ) -> Self {
        Self {
            continuation_id,
            stack_frames,
            prompt_tag,
            is_delimited: true,
            gc_header: GCHeader::new(std::mem::size_of::<ContinuationType>()),
        }
    }
    
    pub fn undelimited(
        continuation_id: ContinuationId,
        stack_frames: Vec<StackFrame>,
    ) -> Self {
        Self {
            continuation_id,
            stack_frames,
            prompt_tag: PromptTag::new(0), // Default prompt tag for undelimited continuations
            is_delimited: false,
            gc_header: GCHeader::new(std::mem::size_of::<ContinuationType>()),
        }
    }
    
    pub fn frame_count(&self) -> usize {
        self.stack_frames.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.stack_frames.is_empty()
    }
    
    pub fn top_frame(&self) -> Option<&StackFrame> {
        self.stack_frames.last()
    }
    
    pub fn top_frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.stack_frames.last_mut()
    }
}

/// Continuation operations trait
pub trait ContinuationOperations {
    fn resume(&self, value: Value) -> Result<Value, ContinuationError>;
    fn abort(&self, value: Value) -> Result<Value, ContinuationError>;
    fn compose(&self, other: &ContinuationType) -> Result<ContinuationType, ContinuationError>;
    fn is_delimited(&self) -> bool;
    fn get_prompt_tag(&self) -> &PromptTag;
    fn capture_current_continuation(prompt_tag: PromptTag) -> Result<ContinuationType, ContinuationError>;
}

/// Continuation-related errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContinuationError {
    /// Empty continuation cannot be resumed
    EmptyContinuation,
    /// Prompt tag mismatch
    PromptTagMismatch {
        expected: PromptTag,
        found: PromptTag,
    },
    /// Invalid continuation state
    InvalidState(String),
    /// Execution error during resume/abort
    ExecutionError(String),
    /// Composition error
    CompositionError(String),
}

impl fmt::Display for ContinuationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContinuationError::EmptyContinuation => {
                write!(f, "Cannot resume empty continuation")
            }
            ContinuationError::PromptTagMismatch { expected, found } => {
                write!(f, "Prompt tag mismatch: expected {:?}, found {:?}", expected, found)
            }
            ContinuationError::InvalidState(msg) => {
                write!(f, "Invalid continuation state: {}", msg)
            }
            ContinuationError::ExecutionError(msg) => {
                write!(f, "Continuation execution error: {}", msg)
            }
            ContinuationError::CompositionError(msg) => {
                write!(f, "Continuation composition error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ContinuationError {}

impl ContinuationOperations for ContinuationType {
    fn resume(&self, _value: Value) -> Result<Value, ContinuationError> {
        if self.is_empty() {
            return Err(ContinuationError::EmptyContinuation);
        }
        
        // In a real implementation, this would:
        // 1. Restore the stack frames
        // 2. Push the resume value onto the operand stack
        // 3. Continue execution from the saved instruction pointer
        // 4. Execute until the prompt tag is reached or the continuation completes
        
        Err(ContinuationError::ExecutionError(
            "Continuation resume not implemented".to_string()
        ))
    }
    
    fn abort(&self, value: Value) -> Result<Value, ContinuationError> {
        if !self.is_delimited {
            return Err(ContinuationError::InvalidState(
                "Cannot abort undelimited continuation".to_string()
            ));
        }
        
        // In a real implementation, this would:
        // 1. Discard all stack frames up to the prompt tag
        // 2. Return the abort value to the prompt handler
        // 3. Continue execution after the prompt
        
        Ok(value) // For now, just return the abort value
    }
    
    fn compose(&self, other: &ContinuationType) -> Result<ContinuationType, ContinuationError> {
        if self.prompt_tag != other.prompt_tag {
            return Err(ContinuationError::PromptTagMismatch {
                expected: self.prompt_tag.clone(),
                found: other.prompt_tag.clone(),
            });
        }
        
        // Compose continuations by concatenating stack frames
        let mut composed_frames = self.stack_frames.clone();
        composed_frames.extend(other.stack_frames.clone());
        
        Ok(ContinuationType::new(
            ContinuationId::new(self.continuation_id.as_u64() * 1000 + other.continuation_id.as_u64()),
            composed_frames,
            self.prompt_tag.clone(),
        ))
    }
    
    fn is_delimited(&self) -> bool {
        self.is_delimited
    }
    
    fn get_prompt_tag(&self) -> &PromptTag {
        &self.prompt_tag
    }
    
    fn capture_current_continuation(prompt_tag: PromptTag) -> Result<ContinuationType, ContinuationError> {
        // In a real implementation, this would:
        // 1. Walk up the call stack until the prompt tag is found
        // 2. Capture all stack frames between current position and the prompt
        // 3. Create a continuation with the captured frames
        
        // For now, create an empty continuation
        Ok(ContinuationType::new(
            ContinuationId::new(1), // Generate unique ID
            Vec::new(),
            prompt_tag,
        ))
    }
}

/// Prompt operations for delimited control
pub struct Prompt {
    tag: PromptTag,
}

impl Prompt {
    pub fn new(tag: PromptTag) -> Self {
        Self { tag }
    }
    
    pub fn tag(&self) -> &PromptTag {
        &self.tag
    }
    
    /// Execute a computation with this prompt
    pub fn with_prompt<F>(&self, computation: F) -> Result<Value, ContinuationError>
    where
        F: FnOnce() -> Result<Value, ContinuationError>,
    {
        // In a real implementation, this would:
        // 1. Install the prompt tag in the execution context
        // 2. Execute the computation
        // 3. Handle any continuations that escape to this prompt
        // 4. Remove the prompt tag when done
        
        computation()
    }
}

/// Control operators for delimited continuations
pub struct ControlOperators;

impl ControlOperators {
    /// Capture the current continuation up to the given prompt tag
    pub fn shift(prompt_tag: PromptTag) -> Result<ContinuationType, ContinuationError> {
        ContinuationType::capture_current_continuation(prompt_tag)
    }
    
    /// Reset the computation with a new prompt
    pub fn reset<F>(prompt_tag: PromptTag, computation: F) -> Result<Value, ContinuationError>
    where
        F: FnOnce() -> Result<Value, ContinuationError>,
    {
        let prompt = Prompt::new(prompt_tag);
        prompt.with_prompt(computation)
    }
    
    /// Call with current continuation (call/cc)
    pub fn call_cc<F>(f: F) -> Result<Value, ContinuationError>
    where
        F: FnOnce(ContinuationType) -> Result<Value, ContinuationError>,
    {
        // Capture the current continuation (undelimited)
        let continuation = ContinuationType::undelimited(
            ContinuationId::new(1), // Generate unique ID
            Vec::new(), // Would capture actual stack frames
        );
        
        f(continuation)
    }
}

impl FunctionOperations for FunctionType {
    fn call(&self, args: &[Value]) -> Result<Value, FunctionError> {
        // Validate argument count
        if args.len() != self.signature.parameter_count() {
            return Err(FunctionError::ArityMismatch {
                expected: self.signature.parameter_count(),
                provided: args.len(),
            });
        }
        
        // Validate argument types
        for (i, (expected_type, arg)) in self.signature.parameter_types.iter().zip(args.iter()).enumerate() {
            let actual_type = arg.type_hash();
            if *expected_type != actual_type {
                return Err(FunctionError::TypeMismatch {
                    parameter_index: i,
                    expected: *expected_type,
                    provided: actual_type,
                });
            }
        }
        
        // Execute function based on implementation type
        match &self.implementation {
            FunctionImplementation::Native { name } => {
                // In a real implementation, this would look up and call the native function
                Err(FunctionError::ExecutionError(format!("Native function '{}' execution not implemented", name)))
            }
            FunctionImplementation::IR { instructions: _, locals: _ } => {
                // In a real implementation, this would execute the IR instructions
                Err(FunctionError::ExecutionError("IR execution not implemented".to_string()))
            }
            FunctionImplementation::FFI { module_name, function_name, calling_convention: _ } => {
                // In a real implementation, this would call the FFI function
                Err(FunctionError::FFIError(format!("FFI call to {}::{} not implemented", module_name, function_name)))
            }
        }
    }
    
    fn partial_apply(&self, args: &[Value]) -> Result<FunctionType, FunctionError> {
        if args.len() >= self.signature.parameter_count() {
            return Err(FunctionError::ArityMismatch {
                expected: self.signature.parameter_count() - 1,
                provided: args.len(),
            });
        }
        
        // Create new signature with remaining parameters
        let remaining_params = self.signature.parameter_types[args.len()..].to_vec();
        let new_signature = FunctionSignature::new(
            remaining_params,
            self.signature.return_type,
            self.signature.is_pure,
            self.signature.is_async,
        );
        
        // Create new capture environment with partially applied arguments
        let mut new_captures = self.captures.clone();
        for (i, arg) in args.iter().enumerate() {
            new_captures.capture(format!("__partial_arg_{}", i), arg.clone(), CaptureMode::ByValue);
        }
        
        // Create new function with updated signature and captures
        Ok(FunctionType {
            id: FunctionId::new(self.id.as_u64() + 1), // Generate new ID
            signature: new_signature,
            implementation: self.implementation.clone(),
            captures: new_captures,
            gc_header: GCHeader::new(std::mem::size_of::<FunctionType>()),
            auto_span: self.auto_span,
            source_map: self.source_map.clone(),
        })
    }
    
    fn get_signature(&self) -> &FunctionSignature {
        &self.signature
    }
    
    fn is_callable_with(&self, arg_types: &[TypeHash]) -> bool {
        self.signature.is_compatible_with(arg_types)
    }
    
    fn compose(&self, other: &FunctionType) -> Result<FunctionType, FunctionError> {
        // Check if functions can be composed (return type of self matches parameter type of other)
        if other.signature.parameter_count() != 1 {
            return Err(FunctionError::CompositionError(
                "Can only compose with unary functions".to_string()
            ));
        }
        
        if self.signature.return_type != other.signature.parameter_types[0] {
            return Err(FunctionError::CompositionError(
                "Return type of first function must match parameter type of second function".to_string()
            ));
        }
        
        // Create composed function signature
        let composed_signature = FunctionSignature::new(
            self.signature.parameter_types.clone(),
            other.signature.return_type,
            self.signature.is_pure && other.signature.is_pure,
            self.signature.is_async || other.signature.is_async,
        );
        
        // Create composed implementation (placeholder)
        let composed_impl = FunctionImplementation::Native {
            name: format!("composed_{}_{}", 
                         self.id.as_u64(), 
                         other.id.as_u64())
        };
        
        // Merge capture environments
        let mut merged_captures = self.captures.clone();
        for name in other.captures.captured_names() {
            if let Some(captured) = other.captures.get_captured(name) {
                merged_captures.capture(
                    format!("__other_{}", name),
                    captured.value.clone(),
                    captured.mode.clone()
                );
            }
        }
        
        Ok(FunctionType {
            id: FunctionId::new(self.id.as_u64() * 1000 + other.id.as_u64()), // Generate composed ID
            signature: composed_signature,
            implementation: composed_impl,
            captures: merged_captures,
            gc_header: GCHeader::new(std::mem::size_of::<FunctionType>()),
            auto_span: self.auto_span || other.auto_span,
            source_map: None, // Composed functions lose source map info
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContentHash;

    fn create_test_type_hash(id: u8) -> TypeHash {
        let mut bytes = [0u8; 32];
        bytes[0] = id;
        TypeHash::new(ContentHash::from_bytes(bytes))
    }

    #[test]
    fn test_function_signature_creation() {
        let param_types = vec![create_test_type_hash(1), create_test_type_hash(2)];
        let return_type = create_test_type_hash(3);
        
        let signature = FunctionSignature::new(param_types.clone(), return_type, true, false);
        
        assert_eq!(signature.parameter_types, param_types);
        assert_eq!(signature.return_type, return_type);
        assert!(signature.is_pure);
        assert!(!signature.is_async);
        assert_eq!(signature.parameter_count(), 2);
    }

    #[test]
    fn test_function_signature_compatibility() {
        let param_types = vec![create_test_type_hash(1), create_test_type_hash(2)];
        let return_type = create_test_type_hash(3);
        let signature = FunctionSignature::new(param_types, return_type, true, false);
        
        // Compatible arguments
        let compatible_args = vec![create_test_type_hash(1), create_test_type_hash(2)];
        assert!(signature.is_compatible_with(&compatible_args));
        
        // Wrong number of arguments
        let wrong_arity = vec![create_test_type_hash(1)];
        assert!(!signature.is_compatible_with(&wrong_arity));
        
        // Wrong types
        let wrong_types = vec![create_test_type_hash(1), create_test_type_hash(99)];
        assert!(!signature.is_compatible_with(&wrong_types));
    }

    #[test]
    fn test_capture_environment() {
        let mut env = CaptureEnvironment::new();
        assert!(env.is_empty());
        
        env.capture("x".to_string(), Value::I32(42), CaptureMode::ByValue);
        env.capture("y".to_string(), Value::String("hello".to_string()), CaptureMode::ByReference);
        
        assert!(!env.is_empty());
        assert_eq!(env.captured_names().count(), 2);
        
        let captured_x = env.get_captured("x").unwrap();
        assert_eq!(captured_x.value, Value::I32(42));
        assert_eq!(captured_x.mode, CaptureMode::ByValue);
        
        let captured_y = env.get_captured("y").unwrap();
        assert_eq!(captured_y.value, Value::String("hello".to_string()));
        assert_eq!(captured_y.mode, CaptureMode::ByReference);
    }

    #[test]
    fn test_function_type_creation() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1)],
            create_test_type_hash(2),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_function".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        
        assert_eq!(function.id, FunctionId::new(1));
        assert_eq!(function.signature.parameter_count(), 1);
        assert!(function.captures.is_empty());
        assert!(!function.auto_span);
    }

    #[test]
    fn test_closure_creation() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1)],
            create_test_type_hash(2),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_closure".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        let closure = ClosureType::new(function, CaptureMode::ByValue);
        
        assert_eq!(closure.capture_count(), 0);
        assert!(closure.is_empty());
        assert_eq!(closure.get_capture_mode(), CaptureMode::ByValue);
    }

    #[test]
    fn test_closure_with_captures() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1)],
            create_test_type_hash(2),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_closure".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        let mut closure = ClosureType::new(function, CaptureMode::ByMutableReference);
        
        // Add captures
        closure.capture("x".to_string(), Value::I32(42));
        closure.capture("y".to_string(), Value::String("hello".to_string()));
        
        assert_eq!(closure.capture_count(), 2);
        assert!(!closure.is_empty());
        
        // Test getting captured values
        assert_eq!(closure.get_captured("x"), Some(&Value::I32(42)));
        assert_eq!(closure.get_captured("y"), Some(&Value::String("hello".to_string())));
        assert_eq!(closure.get_captured("z"), None);
        
        // Test captured names
        let names: Vec<_> = closure.captured_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"x".to_string()));
        assert!(names.contains(&&"y".to_string()));
    }

    #[test]
    fn test_closure_capture_modes() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1)],
            create_test_type_hash(2),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_closure".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        
        // Test by-value capture (cannot update)
        let mut closure_by_value = ClosureType::new(function.clone(), CaptureMode::ByValue);
        closure_by_value.capture("x".to_string(), Value::I32(42));
        
        let result = closure_by_value.update_captured("x", Value::I32(100));
        assert!(matches!(result, Err(FunctionError::CaptureError(_))));
        
        // Test by-reference capture (cannot update)
        let mut closure_by_ref = ClosureType::new(function.clone(), CaptureMode::ByReference);
        closure_by_ref.capture("x".to_string(), Value::I32(42));
        
        let result = closure_by_ref.update_captured("x", Value::I32(100));
        assert!(matches!(result, Err(FunctionError::CaptureError(_))));
        
        // Test by-mutable-reference capture (can update)
        let mut closure_by_mut_ref = ClosureType::new(function, CaptureMode::ByMutableReference);
        closure_by_mut_ref.capture("x".to_string(), Value::I32(42));
        
        let result = closure_by_mut_ref.update_captured("x", Value::I32(100));
        assert!(result.is_ok());
        assert_eq!(closure_by_mut_ref.get_captured("x"), Some(&Value::I32(100)));
        
        // Test updating non-existent capture
        let result = closure_by_mut_ref.update_captured("y", Value::I32(200));
        assert!(matches!(result, Err(FunctionError::CaptureError(_))));
    }

    #[test]
    fn test_closure_clone_with_captures() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1)],
            create_test_type_hash(2),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_closure".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        let mut original_closure = ClosureType::new(function, CaptureMode::ByValue);
        original_closure.capture("x".to_string(), Value::I32(42));
        
        // Clone with new captures
        let mut new_captures = HashMap::new();
        new_captures.insert("y".to_string(), Value::String("world".to_string()));
        new_captures.insert("z".to_string(), Value::Bool(true));
        
        let cloned_closure = original_closure.clone_with_captures(new_captures);
        
        // Original should still have "x"
        assert_eq!(original_closure.get_captured("x"), Some(&Value::I32(42)));
        assert_eq!(original_closure.capture_count(), 1);
        
        // Clone should have new captures
        assert_eq!(cloned_closure.get_captured("y"), Some(&Value::String("world".to_string())));
        assert_eq!(cloned_closure.get_captured("z"), Some(&Value::Bool(true)));
        assert_eq!(cloned_closure.get_captured("x"), None);
        assert_eq!(cloned_closure.capture_count(), 2);
    }

    #[test]
    fn test_closure_call_validation() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1), create_test_type_hash(2)],
            create_test_type_hash(3),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_closure".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        let mut closure = ClosureType::new(function, CaptureMode::ByValue);
        closure.capture("captured_var".to_string(), Value::String("captured".to_string()));
        
        // Test arity mismatch
        let wrong_arity_args = vec![Value::I32(42)];
        let result = closure.call(&wrong_arity_args);
        assert!(matches!(result, Err(FunctionError::ArityMismatch { expected: 2, provided: 1 })));
        
        // Test correct arity (would fail on type checking since Value::type_hash() returns placeholder)
        let correct_args = vec![Value::I32(42), Value::I64(100)];
        let result = closure.call(&correct_args);
        // Since Value::type_hash() returns a placeholder, this will fail on type mismatch
        assert!(matches!(result, Err(FunctionError::TypeMismatch { .. })));
    }

    #[test]
    fn test_prompt_tag_creation() {
        let tag1 = PromptTag::new(1);
        assert_eq!(tag1.id(), 1);
        assert_eq!(tag1.name(), None);
        
        let tag2 = PromptTag::with_name(2, "test_prompt".to_string());
        assert_eq!(tag2.id(), 2);
        assert_eq!(tag2.name(), Some("test_prompt"));
    }

    #[test]
    fn test_stack_frame_creation() {
        let function_id = FunctionId::new(42);
        let mut frame = StackFrame::new(function_id, 100);
        
        assert_eq!(frame.function_id, function_id);
        assert_eq!(frame.instruction_pointer, 100);
        assert!(frame.local_variables.is_empty());
        assert!(frame.operand_stack.is_empty());
        
        // Test adding locals and operands
        frame.set_local("x".to_string(), Value::I32(42));
        frame.push_operand(Value::String("test".to_string()));
        
        assert_eq!(frame.get_local("x"), Some(&Value::I32(42)));
        assert_eq!(frame.operand_stack.len(), 1);
        
        let popped = frame.pop_operand();
        assert_eq!(popped, Some(Value::String("test".to_string())));
        assert!(frame.operand_stack.is_empty());
    }

    #[test]
    fn test_continuation_creation() {
        let continuation_id = ContinuationId::new(1);
        let prompt_tag = PromptTag::new(100);
        let stack_frames = vec![
            StackFrame::new(FunctionId::new(1), 0),
            StackFrame::new(FunctionId::new(2), 50),
        ];
        
        let continuation = ContinuationType::new(continuation_id, stack_frames.clone(), prompt_tag.clone());
        
        assert_eq!(continuation.continuation_id, continuation_id);
        assert_eq!(continuation.prompt_tag, prompt_tag);
        assert!(continuation.is_delimited());
        assert_eq!(continuation.frame_count(), 2);
        assert!(!continuation.is_empty());
        
        // Test top frame access
        let top_frame = continuation.top_frame().unwrap();
        assert_eq!(top_frame.function_id, FunctionId::new(2));
        assert_eq!(top_frame.instruction_pointer, 50);
    }

    #[test]
    fn test_undelimited_continuation() {
        let continuation_id = ContinuationId::new(2);
        let stack_frames = vec![StackFrame::new(FunctionId::new(1), 0)];
        
        let continuation = ContinuationType::undelimited(continuation_id, stack_frames);
        
        assert_eq!(continuation.continuation_id, continuation_id);
        assert!(!continuation.is_delimited());
        assert_eq!(continuation.frame_count(), 1);
    }

    #[test]
    fn test_continuation_abort() {
        let continuation_id = ContinuationId::new(1);
        let prompt_tag = PromptTag::new(100);
        let stack_frames = vec![StackFrame::new(FunctionId::new(1), 0)];
        
        let continuation = ContinuationType::new(continuation_id, stack_frames, prompt_tag);
        
        // Test abort with delimited continuation
        let abort_value = Value::String("aborted".to_string());
        let result = continuation.abort(abort_value.clone());
        assert_eq!(result, Ok(abort_value));
        
        // Test abort with undelimited continuation (should fail)
        let undelimited = ContinuationType::undelimited(continuation_id, vec![]);
        let result = undelimited.abort(Value::I32(42));
        assert!(matches!(result, Err(ContinuationError::InvalidState(_))));
    }

    #[test]
    fn test_continuation_resume() {
        let continuation_id = ContinuationId::new(1);
        let prompt_tag = PromptTag::new(100);
        let stack_frames = vec![StackFrame::new(FunctionId::new(1), 0)];
        
        let continuation = ContinuationType::new(continuation_id, stack_frames, prompt_tag);
        
        // Test resume (should fail since execution is not implemented)
        let resume_value = Value::I32(42);
        let result = continuation.resume(resume_value);
        assert!(matches!(result, Err(ContinuationError::ExecutionError(_))));
        
        // Test resume with empty continuation
        let empty_continuation = ContinuationType::new(
            continuation_id,
            vec![],
            PromptTag::new(100)
        );
        let result = empty_continuation.resume(Value::I32(42));
        assert!(matches!(result, Err(ContinuationError::EmptyContinuation)));
    }

    #[test]
    fn test_continuation_composition() {
        let prompt_tag = PromptTag::new(100);
        
        let cont1 = ContinuationType::new(
            ContinuationId::new(1),
            vec![StackFrame::new(FunctionId::new(1), 0)],
            prompt_tag.clone()
        );
        
        let cont2 = ContinuationType::new(
            ContinuationId::new(2),
            vec![StackFrame::new(FunctionId::new(2), 50)],
            prompt_tag.clone()
        );
        
        // Test successful composition
        let composed = cont1.compose(&cont2).unwrap();
        assert_eq!(composed.frame_count(), 2);
        assert_eq!(composed.prompt_tag, prompt_tag);
        
        // Test composition with different prompt tags (should fail)
        let cont3 = ContinuationType::new(
            ContinuationId::new(3),
            vec![StackFrame::new(FunctionId::new(3), 75)],
            PromptTag::new(200) // Different prompt tag
        );
        
        let result = cont1.compose(&cont3);
        assert!(matches!(result, Err(ContinuationError::PromptTagMismatch { .. })));
    }

    #[test]
    fn test_control_operators() {
        let prompt_tag = PromptTag::new(100);
        
        // Test shift (capture continuation)
        let continuation = ControlOperators::shift(prompt_tag.clone()).unwrap();
        assert_eq!(continuation.prompt_tag, prompt_tag);
        assert!(continuation.is_empty()); // Empty since we're not in a real execution context
        
        // Test reset (execute with prompt)
        let result = ControlOperators::reset(prompt_tag, || {
            Ok(Value::String("reset_result".to_string()))
        }).unwrap();
        assert_eq!(result, Value::String("reset_result".to_string()));
        
        // Test call/cc
        let result = ControlOperators::call_cc(|continuation| {
            assert!(!continuation.is_delimited()); // call/cc creates undelimited continuations
            Ok(Value::I32(42))
        }).unwrap();
        assert_eq!(result, Value::I32(42));
    }

    #[test]
    fn test_prompt_operations() {
        let prompt_tag = PromptTag::with_name(1, "test_prompt".to_string());
        let prompt = Prompt::new(prompt_tag.clone());
        
        assert_eq!(prompt.tag(), &prompt_tag);
        
        // Test with_prompt
        let result = prompt.with_prompt(|| {
            Ok(Value::Bool(true))
        }).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_function_call_validation() {
        let signature = FunctionSignature::new(
            vec![create_test_type_hash(1), create_test_type_hash(2)],
            create_test_type_hash(3),
            true,
            false
        );
        
        let implementation = FunctionImplementation::Native {
            name: "test_function".to_string(),
        };
        
        let function = FunctionType::new(FunctionId::new(1), signature, implementation);
        
        // Test arity mismatch
        let wrong_arity_args = vec![Value::I32(42)];
        let result = function.call(&wrong_arity_args);
        assert!(matches!(result, Err(FunctionError::ArityMismatch { expected: 2, provided: 1 })));
        
        // Test correct arity (would fail on type checking since Value::type_hash() returns placeholder)
        let correct_args = vec![Value::I32(42), Value::I64(100)];
        let result = function.call(&correct_args);
        // Since Value::type_hash() returns a placeholder, this will fail on type mismatch
        assert!(matches!(result, Err(FunctionError::TypeMismatch { .. })));
    }
}