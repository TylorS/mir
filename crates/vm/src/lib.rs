//! MIR VM - Virtual machine execution engine
//! 
//! This crate provides the virtual machine for executing MIR IR, including:
//! - Execution context and engine
//! - Instruction set that maps to WASM operations
//! - Call stack management
//! - Hot-module-reloading coordination
//! - Function registry and optimization
//! - Recursion optimization

pub mod execution;
pub mod instruction;
pub mod call_stack;
pub mod hot_module_reloading;
pub mod function_registry;
pub mod recursion_optimizer;

pub use execution::{ExecutionContext, VirtualMachine, ExecutionResult, ExecutionError, SideEffect};
pub use instruction::{Instruction, InstructionSequence, InstructionId, LogLevel, SpanStatus, MetricType};
pub use call_stack::{CallStack, StackFrame, CallStackError, StackTraceEntry};
pub use hot_module_reloading::HMRCoordinator;
pub use function_registry::FunctionRegistry;
pub use recursion_optimizer::RecursionOptimizer;