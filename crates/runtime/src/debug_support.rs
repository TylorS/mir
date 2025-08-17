use crate::source_mapping::{SourceMapRegistry, SourcePosition, GeneratedPosition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Debug information for runtime execution
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Current execution stack with source mapping
    pub call_stack: Vec<StackFrame>,
    /// Variable bindings with source locations
    pub variable_bindings: HashMap<String, VariableInfo>,
    /// Breakpoints set by debugger
    pub breakpoints: Vec<Breakpoint>,
}

/// Stack frame with source mapping information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Module hash for this frame
    pub module_hash: crate::ContentHash,
    /// Function name
    pub function_name: String,
    /// Generated position in compiled code
    pub generated_position: GeneratedPosition,
    /// Mapped source position (if available)
    pub source_position: Option<SourcePosition>,
    /// Local variables in this frame
    pub locals: HashMap<String, String>, // Variable name -> serialized value
}

/// Variable information with source location
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub value: String, // Serialized representation
    pub type_name: String,
    pub source_location: Option<SourcePosition>,
}

/// Breakpoint information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breakpoint {
    pub id: u32,
    pub source_file: std::path::PathBuf,
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,
    pub enabled: bool,
}

/// Debug session manager
pub struct DebugSession {
    source_map_registry: SourceMapRegistry,
    active_breakpoints: HashMap<u32, Breakpoint>,
    next_breakpoint_id: u32,
    execution_paused: bool,
    current_debug_info: Option<DebugInfo>,
}

impl DebugSession {
    pub fn new(source_map_registry: SourceMapRegistry) -> Self {
        Self {
            source_map_registry,
            active_breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
            execution_paused: false,
            current_debug_info: None,
        }
    }

    /// Set a breakpoint at a source location
    pub fn set_breakpoint(&mut self, file: std::path::PathBuf, line: u32, column: Option<u32>, condition: Option<String>) -> u32 {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        let breakpoint = Breakpoint {
            id,
            source_file: file,
            line,
            column,
            condition,
            enabled: true,
        };
        
        self.active_breakpoints.insert(id, breakpoint);
        id
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: u32) -> bool {
        self.active_breakpoints.remove(&id).is_some()
    }

    /// Enable/disable a breakpoint
    pub fn toggle_breakpoint(&mut self, id: u32, enabled: bool) -> bool {
        if let Some(bp) = self.active_breakpoints.get_mut(&id) {
            bp.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Check if execution should pause at current position
    pub fn should_pause_at(&self, module_hash: &crate::ContentHash, generated_pos: GeneratedPosition) -> bool {
        if let Some(source_pos) = self.source_map_registry.map_to_source(module_hash, generated_pos) {
            for breakpoint in self.active_breakpoints.values() {
                if breakpoint.enabled && 
                   breakpoint.source_file == source_pos.file && 
                   breakpoint.line == source_pos.line {
                    
                    // Check column if specified
                    if let Some(bp_col) = breakpoint.column {
                        if bp_col != source_pos.column {
                            continue;
                        }
                    }
                    
                    // TODO: Evaluate condition if specified
                    return true;
                }
            }
        }
        false
    }

    /// Update current execution state for debugging
    pub fn update_execution_state(&mut self, call_stack: Vec<StackFrame>) {
        // Map generated positions to source positions
        let mut mapped_stack = Vec::new();
        
        for mut frame in call_stack {
            frame.source_position = self.source_map_registry.map_to_source(
                &frame.module_hash, 
                frame.generated_position.clone()
            );
            mapped_stack.push(frame);
        }
        
        self.current_debug_info = Some(DebugInfo {
            call_stack: mapped_stack,
            variable_bindings: HashMap::new(), // TODO: Populate from VM state
            breakpoints: self.active_breakpoints.values().cloned().collect(),
        });
    }

    /// Get current debug information
    pub fn get_debug_info(&self) -> Option<&DebugInfo> {
        self.current_debug_info.as_ref()
    }

    /// Pause execution
    pub fn pause(&mut self) {
        self.execution_paused = true;
    }

    /// Resume execution
    pub fn resume(&mut self) {
        self.execution_paused = false;
    }

    /// Check if execution is paused
    pub fn is_paused(&self) -> bool {
        self.execution_paused
    }

    /// Get source content for debugging
    pub fn get_source_content(&self, module_hash: &crate::ContentHash, source_index: usize) -> Option<&str> {
        self.source_map_registry.get_source_content(module_hash, source_index)
    }
}

/// Debug event for external debugger integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEvent {
    BreakpointHit {
        breakpoint_id: u32,
        stack_frame: StackFrame,
    },
    StepComplete {
        stack_frame: StackFrame,
    },
    ExceptionThrown {
        exception: String,
        stack_trace: Vec<StackFrame>,
    },
    ExecutionPaused,
    ExecutionResumed,
}

/// Debug event handler trait for external integration
pub trait DebugEventHandler {
    fn handle_debug_event(&mut self, event: DebugEvent);
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref source_pos) = self.source_position {
            write!(f, "{}:{} in {} ({}:{}:{})", 
                source_pos.file.display(), 
                source_pos.line + 1, // Convert to 1-based for display
                self.function_name,
                source_pos.file.file_name().unwrap_or_default().to_string_lossy(),
                source_pos.line + 1,
                source_pos.column + 1
            )
        } else {
            write!(f, "{}:{}:{} in {}", 
                self.module_hash, 
                self.generated_position.line + 1,
                self.generated_position.column + 1,
                self.function_name
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::path::PathBuf;

    #[test]
    fn test_breakpoint_management() {
        let registry = SourceMapRegistry::new();
        let mut session = DebugSession::new(registry);
        
        let bp_id = session.set_breakpoint(
            PathBuf::from("src/main.rs"), 
            10, 
            Some(5), 
            None
        );
        
        assert_eq!(bp_id, 1);
        assert!(session.active_breakpoints.contains_key(&bp_id));
        
        assert!(session.toggle_breakpoint(bp_id, false));
        assert!(!session.active_breakpoints[&bp_id].enabled);
        
        assert!(session.remove_breakpoint(bp_id));
        assert!(!session.active_breakpoints.contains_key(&bp_id));
    }

    #[test]
    fn test_stack_frame_display() {
        let frame = StackFrame {
            module_hash: crate::ContentHash::from_bytes([1; 8]),
            function_name: "main".to_string(),
            generated_position: GeneratedPosition { line: 0, column: 0 },
            source_position: Some(SourcePosition {
                file: PathBuf::from("src/main.rs"),
                line: 5,
                column: 10,
                name: Some("main".to_string()),
            }),
            locals: HashMap::new(),
        };
        
        let display = format!("{frame}");
        assert!(display.contains("src/main.rs:6"));
        assert!(display.contains("main"));
    }
}