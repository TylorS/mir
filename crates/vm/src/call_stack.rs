//! Call stack management for VM execution

use mir_types::{Value, ContentHash};
use crate::instruction::LocalVariable;
use std::collections::HashMap;

/// Call stack frame
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_id: u64,
    pub module_hash: ContentHash,
    pub instruction_pointer: usize,
    pub locals: HashMap<u32, Value>,
    pub return_address: Option<usize>,
    pub span_id: Option<u64>, // For OpenTelemetry tracing
}

impl StackFrame {
    pub fn new(function_id: u64, module_hash: ContentHash, locals: Vec<LocalVariable>) -> Self {
        let mut local_map = HashMap::new();
        
        // Initialize locals with default values
        for local in locals {
            local_map.insert(local.index, Value::Null);
        }
        
        StackFrame {
            function_id,
            module_hash,
            instruction_pointer: 0,
            locals: local_map,
            return_address: None,
            span_id: None,
        }
    }
    
    pub fn get_local(&self, index: u32) -> Option<&Value> {
        self.locals.get(&index)
    }
    
    pub fn set_local(&mut self, index: u32, value: Value) -> Result<(), CallStackError> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.locals.entry(index) {
            e.insert(value);
            Ok(())
        } else {
            Err(CallStackError::InvalidLocalIndex(index))
        }
    }
    
    pub fn advance_ip(&mut self) {
        self.instruction_pointer += 1;
    }
    
    pub fn jump_to(&mut self, address: usize) {
        self.instruction_pointer = address;
    }
}

/// Call stack for managing function calls
#[derive(Debug)]
pub struct CallStack {
    frames: Vec<StackFrame>,
    max_depth: usize,
}

impl CallStack {
    pub fn new(max_depth: usize) -> Self {
        CallStack {
            frames: Vec::new(),
            max_depth,
        }
    }
    
    pub fn push_frame(&mut self, frame: StackFrame) -> Result<(), CallStackError> {
        if self.frames.len() >= self.max_depth {
            return Err(CallStackError::StackOverflow);
        }
        
        self.frames.push(frame);
        Ok(())
    }
    
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }
    
    pub fn current_frame(&self) -> Option<&StackFrame> {
        self.frames.last()
    }
    
    pub fn current_frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.frames.last_mut()
    }
    
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    
    pub fn get_stack_trace(&self) -> Vec<StackTraceEntry> {
        self.frames.iter().map(|frame| StackTraceEntry {
            function_id: frame.function_id,
            module_hash: frame.module_hash,
            instruction_pointer: frame.instruction_pointer,
        }).collect()
    }
    
    /// Preserve call stack state for hot-reloading
    pub fn create_snapshot(&self) -> CallStackSnapshot {
        CallStackSnapshot {
            frames: self.frames.clone(),
        }
    }
    
    /// Restore call stack from snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: CallStackSnapshot) -> Result<(), CallStackError> {
        if snapshot.frames.len() > self.max_depth {
            return Err(CallStackError::StackOverflow);
        }
        
        self.frames = snapshot.frames;
        Ok(())
    }
}

/// Stack trace entry for debugging
#[derive(Debug, Clone)]
pub struct StackTraceEntry {
    pub function_id: u64,
    pub module_hash: ContentHash,
    pub instruction_pointer: usize,
}

/// Call stack snapshot for hot-reloading
#[derive(Debug, Clone)]
pub struct CallStackSnapshot {
    pub frames: Vec<StackFrame>,
}

/// Call stack errors
#[derive(Debug, thiserror::Error)]
pub enum CallStackError {
    #[error("Stack overflow: maximum depth exceeded")]
    StackOverflow,
    
    #[error("Invalid local variable index: {0}")]
    InvalidLocalIndex(u32),
    
    #[error("No current frame")]
    NoCurrentFrame,
    
    #[error("Frame restoration failed: {0}")]
    RestorationFailed(String),
}