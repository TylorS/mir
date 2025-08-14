//! Virtual machine execution engine

use mir_ast::Module;
use mir_runtime::{StateManager, DistributedRuntime, OpenTelemetryCollector};
use mir_types::{ContentHash, Value, TypeRegistry};
use crate::instruction::{Instruction, InstructionSequence, LogLevel};
use crate::call_stack::{CallStack, StackFrame, CallStackError};
use crate::function_registry::{FunctionRegistry, CallError};
use std::collections::HashMap;
use std::sync::Arc;

/// Execution context for the virtual machine with type registry and state management
pub struct ExecutionContext {
    pub type_registry: Arc<TypeRegistry>,
    pub state_manager: StateManager,
    pub distributed_runtime: DistributedRuntime,
    pub telemetry: OpenTelemetryCollector,
    pub function_registry: FunctionRegistry,
    pub global_variables: HashMap<u32, Value>,
    pub memory: Vec<u8>,
}

impl ExecutionContext {
    pub fn new(
        type_registry: Arc<TypeRegistry>,
        state_manager: StateManager,
        distributed_runtime: DistributedRuntime,
        telemetry: OpenTelemetryCollector,
    ) -> Self {
        ExecutionContext {
            type_registry,
            state_manager,
            distributed_runtime,
            telemetry,
            function_registry: FunctionRegistry::new(),
            global_variables: HashMap::new(),
            memory: Vec::new(),
        }
    }
    
    pub fn get_global(&self, index: u32) -> Option<&Value> {
        self.global_variables.get(&index)
    }
    
    pub fn set_global(&mut self, index: u32, value: Value) {
        self.global_variables.insert(index, value);
    }
    
    pub fn allocate_memory(&mut self, size: usize) -> usize {
        let offset = self.memory.len();
        self.memory.resize(offset + size, 0);
        offset
    }
    
    pub fn read_memory(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size <= self.memory.len() {
            Some(&self.memory[offset..offset + size])
        } else {
            None
        }
    }
    
    pub fn write_memory(&mut self, offset: usize, data: &[u8]) -> Result<(), ExecutionError> {
        if offset + data.len() <= self.memory.len() {
            self.memory[offset..offset + data.len()].copy_from_slice(data);
            Ok(())
        } else {
            Err(ExecutionError::MemoryAccessViolation { offset, size: data.len() })
        }
    }
}

/// Virtual machine for executing MIR IR with instruction-based execution
pub struct VirtualMachine {
    context: ExecutionContext,
    loaded_modules: HashMap<ContentHash, Module>,
    call_stack: CallStack,
    instruction_sequences: HashMap<u64, InstructionSequence>,
    execution_stack: Vec<Value>, // Operand stack for instruction execution
}

impl VirtualMachine {
    pub fn new(context: ExecutionContext) -> Self {
        VirtualMachine {
            context,
            loaded_modules: HashMap::new(),
            call_stack: CallStack::new(1000), // Max stack depth of 1000
            instruction_sequences: HashMap::new(),
            execution_stack: Vec::new(),
        }
    }
    
    pub fn load_module(&mut self, hash: ContentHash, module: Module) {
        self.loaded_modules.insert(hash, module);
    }
    
    pub fn register_function(&mut self, function_id: u64, instructions: InstructionSequence) {
        self.instruction_sequences.insert(function_id, instructions);
    }
    
    pub fn execute_module(&mut self, module_hash: ContentHash) -> Result<ExecutionResult, ExecutionError> {
        let _module = self.loaded_modules.get(&module_hash)
            .ok_or_else(|| ExecutionError::ModuleNotFound { hash: module_hash })?;
        
        // Find the main function or entry point
        let main_function_id = 0; // Assume function ID 0 is the entry point
        
        self.execute_function(main_function_id, &[])
    }
    
    pub fn execute_function(&mut self, function_id: u64, args: &[Value]) -> Result<ExecutionResult, ExecutionError> {
        let instructions = self.instruction_sequences.get(&function_id)
            .ok_or_else(|| ExecutionError::FunctionNotFound { id: function_id })?
            .clone();
        
        // Create new stack frame
        let frame = StackFrame::new(function_id, ContentHash::zero(), instructions.locals.clone());
        self.call_stack.push_frame(frame)
            .map_err(|e| ExecutionError::CallStack(e))?;
        
        // Set up arguments as local variables
        for (i, arg) in args.iter().enumerate() {
            if let Some(frame) = self.call_stack.current_frame_mut() {
                frame.set_local(i as u32, arg.clone())
                    .map_err(|e| ExecutionError::CallStack(e))?;
            }
        }
        
        let mut side_effects = Vec::new();
        
        // Execute instructions
        while let Some(frame) = self.call_stack.current_frame() {
            let ip = frame.instruction_pointer;
            if ip >= instructions.instructions.len() {
                break;
            }
            
            let instruction = &instructions.instructions[ip];
            let effect = self.execute_instruction(instruction)?;
            
            if let Some(effect) = effect {
                side_effects.push(effect);
            }
            
            // Advance instruction pointer if not modified by instruction
            if let Some(frame) = self.call_stack.current_frame_mut() {
                if frame.instruction_pointer == ip {
                    frame.advance_ip();
                }
            }
        }
        
        // Get return value from execution stack
        let return_value = self.execution_stack.pop();
        
        // Clean up stack frame
        self.call_stack.pop_frame();
        
        Ok(ExecutionResult {
            return_value,
            side_effects,
        })
    }
    
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<Option<SideEffect>, ExecutionError> {
        match instruction {
            Instruction::Nop => Ok(None),
            
            // Constants
            Instruction::I32Const { value } => {
                self.execution_stack.push(Value::I32(*value));
                Ok(None)
            },
            Instruction::I64Const { value } => {
                self.execution_stack.push(Value::I64(*value));
                Ok(None)
            },
            Instruction::F32Const { value } => {
                self.execution_stack.push(Value::F32(*value));
                Ok(None)
            },
            Instruction::F64Const { value } => {
                self.execution_stack.push(Value::F64(*value));
                Ok(None)
            },
            
            // Local variables
            Instruction::LocalGet { index } => {
                let value = self.call_stack.current_frame()
                    .ok_or(ExecutionError::NoCurrentFrame)?
                    .get_local(*index)
                    .ok_or(ExecutionError::InvalidLocalIndex(*index))?
                    .clone();
                self.execution_stack.push(value);
                Ok(None)
            },
            Instruction::LocalSet { index, value: _ } => {
                let value = self.execution_stack.pop()
                    .ok_or(ExecutionError::StackUnderflow)?;
                self.call_stack.current_frame_mut()
                    .ok_or(ExecutionError::NoCurrentFrame)?
                    .set_local(*index, value)
                    .map_err(|e| ExecutionError::CallStack(e))?;
                Ok(None)
            },
            
            // Global variables
            Instruction::GlobalGet { index } => {
                let value = self.context.get_global(*index)
                    .ok_or(ExecutionError::InvalidGlobalIndex(*index))?
                    .clone();
                self.execution_stack.push(value);
                Ok(None)
            },
            Instruction::GlobalSet { index, value: _ } => {
                let value = self.execution_stack.pop()
                    .ok_or(ExecutionError::StackUnderflow)?;
                self.context.set_global(*index, value);
                Ok(None)
            },
            
            // Arithmetic operations
            Instruction::I32Add => {
                let b = self.pop_i32()?;
                let a = self.pop_i32()?;
                self.execution_stack.push(Value::I32(a.wrapping_add(b)));
                Ok(None)
            },
            Instruction::I32Sub => {
                let b = self.pop_i32()?;
                let a = self.pop_i32()?;
                self.execution_stack.push(Value::I32(a.wrapping_sub(b)));
                Ok(None)
            },
            Instruction::I32Mul => {
                let b = self.pop_i32()?;
                let a = self.pop_i32()?;
                self.execution_stack.push(Value::I32(a.wrapping_mul(b)));
                Ok(None)
            },
            
            // Function calls
            Instruction::Call { function_id, args: _ } => {
                // Collect arguments from stack
                let arg_count = self.get_function_arg_count(*function_id)?;
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.execution_stack.pop().ok_or(ExecutionError::StackUnderflow)?);
                }
                args.reverse();
                
                let result = self.execute_function(*function_id, &args)?;
                if let Some(return_value) = result.return_value {
                    self.execution_stack.push(return_value);
                }
                Ok(None)
            },
            
            // Control flow
            Instruction::Return { value: _ } => {
                // Return value should already be on stack
                Ok(None)
            },
            
            // Observability
            Instruction::LogEvent { level, message, fields } => {
                Ok(Some(SideEffect::LogEvent {
                    level: level.clone(),
                    message: message.clone(),
                    fields: fields.clone(),
                }))
            },
            
            // Hot-reloading
            Instruction::StateSnapshot { module_hash } => {
                let snapshot = self.call_stack.create_snapshot();
                Ok(Some(SideEffect::StateSnapshot {
                    module_hash: *module_hash,
                    snapshot_data: format!("{:?}", snapshot),
                }))
            },
            
            _ => {
                // For now, return an error for unimplemented instructions
                Err(ExecutionError::UnimplementedInstruction(format!("{:?}", instruction)))
            }
        }
    }
    
    fn pop_i32(&mut self) -> Result<i32, ExecutionError> {
        match self.execution_stack.pop() {
            Some(Value::I32(val)) => Ok(val),
            Some(other) => Err(ExecutionError::TypeMismatch {
                expected: "i32".to_string(),
                found: format!("{:?}", other),
            }),
            None => Err(ExecutionError::StackUnderflow),
        }
    }
    
    fn get_function_arg_count(&self, _function_id: u64) -> Result<usize, ExecutionError> {
        // For now, assume all functions take 0 arguments
        // This should be looked up from function metadata
        Ok(0)
    }
    
    pub fn get_call_stack_trace(&self) -> Vec<crate::call_stack::StackTraceEntry> {
        self.call_stack.get_stack_trace()
    }
}

/// Execution result with return value and side effects
#[derive(Debug)]
pub struct ExecutionResult {
    pub return_value: Option<Value>,
    pub side_effects: Vec<SideEffect>,
}

/// Side effects produced during execution
#[derive(Debug)]
pub enum SideEffect {
    StateChange { key: String, old_value: Option<Value>, new_value: Value },
    NetworkCall { endpoint: String, data: Vec<u8> },
    FileWrite { path: String, content: Vec<u8> },
    LogEvent { level: LogLevel, message: String, fields: Vec<(String, Value)> },
    StateSnapshot { module_hash: ContentHash, snapshot_data: String },
}

/// Execution errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Module not found: {hash:?}")]
    ModuleNotFound { hash: ContentHash },
    
    #[error("Function not found: {id}")]
    FunctionNotFound { id: u64 },
    
    #[error("Call stack error: {0}")]
    CallStack(#[from] CallStackError),
    
    #[error("Function call error: {0}")]
    FunctionCall(#[from] CallError),
    
    #[error("Stack underflow")]
    StackUnderflow,
    
    #[error("No current frame")]
    NoCurrentFrame,
    
    #[error("Invalid local variable index: {0}")]
    InvalidLocalIndex(u32),
    
    #[error("Invalid global variable index: {0}")]
    InvalidGlobalIndex(u32),
    
    #[error("Memory access violation at offset {offset} with size {size}")]
    MemoryAccessViolation { offset: usize, size: usize },
    
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
    
    #[error("Unimplemented instruction: {0}")]
    UnimplementedInstruction(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::LocalVariable;
    use mir_types::TypeHash;
    
    fn create_test_context() -> ExecutionContext {
        let type_registry = Arc::new(TypeRegistry::new());
        let state_manager = StateManager::new();
        let distributed_runtime = DistributedRuntime::new(mir_runtime::distributed::NodeId::new(1));
        let telemetry = OpenTelemetryCollector::new();
        
        ExecutionContext::new(type_registry, state_manager, distributed_runtime, telemetry)
    }
    
    #[test]
    fn test_vm_creation() {
        let context = create_test_context();
        let vm = VirtualMachine::new(context);
        assert_eq!(vm.execution_stack.len(), 0);
        assert_eq!(vm.call_stack.depth(), 0);
    }
    
    #[test]
    fn test_simple_arithmetic() {
        let context = create_test_context();
        let mut vm = VirtualMachine::new(context);
        
        // Create a simple function that adds two constants
        let instructions = InstructionSequence {
            instructions: vec![
                Instruction::I32Const { value: 10 },
                Instruction::I32Const { value: 20 },
                Instruction::I32Add,
                Instruction::Return { value: None },
            ],
            locals: vec![],
            source_map: None,
        };
        
        vm.register_function(0, instructions);
        
        let result = vm.execute_function(0, &[]).unwrap();
        assert_eq!(result.return_value, Some(Value::I32(30)));
    }
    
    #[test]
    fn test_local_variables() {
        let context = create_test_context();
        let mut vm = VirtualMachine::new(context);
        
        // Create a function that uses local variables
        let instructions = InstructionSequence {
            instructions: vec![
                Instruction::I32Const { value: 42 },
                Instruction::LocalSet { index: 0, value: Box::new(Instruction::Nop) },
                Instruction::LocalGet { index: 0 },
                Instruction::Return { value: None },
            ],
            locals: vec![LocalVariable {
                index: 0,
                name: Some("test_var".to_string()),
                type_hash: TypeHash::new(mir_types::ContentHash::zero()),
            }],
            source_map: None,
        };
        
        vm.register_function(0, instructions);
        
        let result = vm.execute_function(0, &[]).unwrap();
        assert_eq!(result.return_value, Some(Value::I32(42)));
    }
    
    #[test]
    fn test_global_variables() {
        let context = create_test_context();
        let mut vm = VirtualMachine::new(context);
        
        // Set a global variable
        vm.context.set_global(0, Value::I32(100));
        
        // Create a function that reads the global variable
        let instructions = InstructionSequence {
            instructions: vec![
                Instruction::GlobalGet { index: 0 },
                Instruction::Return { value: None },
            ],
            locals: vec![],
            source_map: None,
        };
        
        vm.register_function(0, instructions);
        
        let result = vm.execute_function(0, &[]).unwrap();
        assert_eq!(result.return_value, Some(Value::I32(100)));
    }
}