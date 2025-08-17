use crate::source_mapping::{SourceMapRegistry, SourcePosition, GeneratedPosition};
use crate::debug_support::StackFrame;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Runtime error with source mapping information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMappedError {
    /// Error message
    pub message: String,
    /// Error kind/type
    pub kind: ErrorKind,
    /// Stack trace with source mapping
    pub stack_trace: Vec<StackFrame>,
    /// Original source location where error occurred
    pub source_location: Option<SourcePosition>,
    /// Additional context information
    pub context: ErrorContext,
    /// Error code for programmatic handling
    pub error_code: Option<String>,
}

/// Types of runtime errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Type system errors
    TypeError {
        expected: String,
        actual: String,
    },
    /// Runtime execution errors
    RuntimeError {
        operation: String,
    },
    /// Memory management errors
    MemoryError {
        allocation_size: Option<usize>,
    },
    /// Hot-reloading errors
    HotReloadError {
        module_hash: String,
        reason: String,
    },
    /// FFI errors
    FFIError {
        function_name: String,
        ffi_error: String,
    },
    /// Distributed coordination errors
    DistributedError {
        node_id: String,
        coordination_error: String,
    },
    /// User-defined errors
    UserError {
        user_message: String,
    },
}

/// Additional error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Module where error occurred
    pub module_hash: Option<String>,
    /// Function where error occurred
    pub function_name: Option<String>,
    /// Variable values at error time
    pub variable_values: std::collections::HashMap<String, String>,
    /// Timestamp when error occurred
    pub timestamp: String,
    /// Thread/execution context ID
    pub execution_context_id: Option<String>,
}

/// Error reporter that maps errors to original source locations
pub struct SourceMappedErrorReporter {
    source_map_registry: SourceMapRegistry,
    error_handlers: Vec<Box<dyn ErrorHandler>>,
    settings: ErrorReportingSettings,
}

/// Error reporting settings
#[derive(Debug, Clone)]
pub struct ErrorReportingSettings {
    /// Include variable values in error reports
    pub include_variable_values: bool,
    /// Maximum stack trace depth
    pub max_stack_trace_depth: usize,
    /// Include source code snippets in error reports
    pub include_source_snippets: bool,
    /// Number of lines to show around error location
    pub source_snippet_context_lines: usize,
}

/// Error handler trait for custom error processing
pub trait ErrorHandler: Send + Sync {
    fn handle_error(&mut self, error: &SourceMappedError);
}

impl SourceMappedErrorReporter {
    pub fn new(source_map_registry: SourceMapRegistry) -> Self {
        Self {
            source_map_registry,
            error_handlers: Vec::new(),
            settings: ErrorReportingSettings::default(),
        }
    }

    /// Report an error with source mapping
    pub fn report_error(
        &mut self,
        message: String,
        kind: ErrorKind,
        module_hash: crate::ContentHash,
        generated_position: GeneratedPosition,
        stack_trace: Vec<StackFrame>,
        context: ErrorContext,
    ) -> SourceMappedError {
        // Map error location to source
        let source_location = self.source_map_registry.map_to_source(&module_hash, generated_position);
        
        // Create mapped error
        let error = SourceMappedError {
            message,
            kind,
            stack_trace,
            source_location,
            context,
            error_code: None,
        };
        
        // Notify error handlers
        for handler in &mut self.error_handlers {
            handler.handle_error(&error);
        }
        
        error
    }

    /// Add an error handler
    pub fn add_error_handler(&mut self, handler: Box<dyn ErrorHandler>) {
        self.error_handlers.push(handler);
    }

    /// Generate human-readable error report
    pub fn format_error_report(&self, error: &SourceMappedError) -> String {
        let mut report = String::new();
        
        // Error header
        report.push_str(&format!("Error: {}\n", error.message));
        report.push_str(&format!("Kind: {:?}\n", error.kind));
        
        if let Some(code) = &error.error_code {
            report.push_str(&format!("Code: {code}\n"));
        }
        
        // Source location
        if let Some(ref source_pos) = error.source_location {
            report.push_str(&format!("\nAt: {}:{}:{}\n", 
                source_pos.file.display(),
                source_pos.line + 1,
                source_pos.column + 1
            ));
            
            // Include source snippet if enabled
            if self.settings.include_source_snippets {
                if let Some(snippet) = self.get_source_snippet(error) {
                    report.push_str(&format!("\nSource:\n{snippet}\n"));
                }
            }
        }
        
        // Stack trace
        if !error.stack_trace.is_empty() {
            report.push_str("\nStack trace:\n");
            for (i, frame) in error.stack_trace.iter()
                .take(self.settings.max_stack_trace_depth)
                .enumerate() {
                report.push_str(&format!("  {i}: {frame}\n"));
            }
        }
        
        // Context information
        if let Some(ref module_hash) = error.context.module_hash {
            report.push_str(&format!("\nModule: {module_hash}\n"));
        }
        
        if let Some(ref function_name) = error.context.function_name {
            report.push_str(&format!("Function: {function_name}\n"));
        }
        
        // Variable values
        if self.settings.include_variable_values && !error.context.variable_values.is_empty() {
            report.push_str("\nVariable values:\n");
            for (name, value) in &error.context.variable_values {
                report.push_str(&format!("  {name} = {value}\n"));
            }
        }
        
        report.push_str(&format!("\nTimestamp: {}\n", error.context.timestamp));
        
        report
    }

    /// Get source code snippet around error location
    fn get_source_snippet(&self, error: &SourceMappedError) -> Option<String> {
        let source_pos = error.source_location.as_ref()?;
        let module_hash = error.context.module_hash.as_ref()?;
        
        // Parse module hash
        let hash_bytes = hex::decode(module_hash).ok()?;
        if hash_bytes.len() != 8 {
            return None;
        }
        let mut hash_array = [0u8; 8];
        hash_array.copy_from_slice(&hash_bytes);
        let module_hash = crate::ContentHash::new(&hash_array);
        
        // Find source file index
        let source_files = self.source_map_registry.get_source_files(&module_hash)?;
        let source_index = source_files.iter().position(|f| f == &source_pos.file)?;
        
        // Get source content
        let source_content = self.source_map_registry.get_source_content(&module_hash, source_index)?;
        let lines: Vec<&str> = source_content.lines().collect();
        
        let error_line = source_pos.line as usize;
        let context_lines = self.settings.source_snippet_context_lines;
        
        let start_line = error_line.saturating_sub(context_lines);
        let end_line = (error_line + context_lines + 1).min(lines.len());
        
        let mut snippet = String::new();
        
        for (i, line) in lines[start_line..end_line].iter().enumerate() {
            let line_num = start_line + i + 1; // 1-based line numbers
            let marker = if start_line + i == error_line { ">>>" } else { "   " };
            snippet.push_str(&format!("{marker} {line_num:4} | {line}\n"));
            
            // Add column indicator for error line
            if start_line + i == error_line {
                let spaces = " ".repeat(8 + source_pos.column as usize);
                snippet.push_str(&format!("{spaces}^\n"));
            }
        }
        
        Some(snippet)
    }

    /// Update error reporting settings
    pub fn update_settings(&mut self, settings: ErrorReportingSettings) {
        self.settings = settings;
    }
}

/// Console error handler that prints errors to stderr
pub struct ConsoleErrorHandler;

impl ErrorHandler for ConsoleErrorHandler {
    fn handle_error(&mut self, error: &SourceMappedError) {
        eprintln!("Runtime Error: {}", error.message);
        if let Some(ref source_pos) = error.source_location {
            eprintln!("  at {}:{}:{}", 
                source_pos.file.display(),
                source_pos.line + 1,
                source_pos.column + 1
            );
        }
    }
}

/// File error handler that logs errors to a file
pub struct FileErrorHandler {
    file_path: std::path::PathBuf,
}

impl FileErrorHandler {
    pub fn new(file_path: std::path::PathBuf) -> Self {
        Self { file_path }
    }
}

impl ErrorHandler for FileErrorHandler {
    fn handle_error(&mut self, error: &SourceMappedError) {
        if let Ok(serialized) = serde_json::to_string_pretty(error) {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.file_path) {
                use std::io::Write;
                let _ = writeln!(file, "{serialized}");
            }
        }
    }
}

impl Default for ErrorReportingSettings {
    fn default() -> Self {
        Self {
            include_variable_values: true,
            max_stack_trace_depth: 20,
            include_source_snippets: true,
            source_snippet_context_lines: 3,
        }
    }
}

impl fmt::Display for SourceMappedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        
        if let Some(ref source_pos) = self.source_location {
            write!(f, " at {}:{}:{}", 
                source_pos.file.display(),
                source_pos.line + 1,
                source_pos.column + 1
            )?;
        }
        
        Ok(())
    }
}

impl std::error::Error for SourceMappedError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_mapping::{SourceMap, SourceMapping};
    use std::path::PathBuf;

    #[test]
    fn test_error_reporting() {
        let mut registry = crate::source_mapping::SourceMapRegistry::new();
        let module_hash = crate::ContentHash::from_bytes([1; 8]);
        
        // Register source map with content
        let source_map = SourceMap {
            version: 3,
            sources: vec![PathBuf::from("src/main.rs")],
            sources_content: Some(vec![Some("fn main() {\n    panic!(\"test error\");\n}".to_string())]),
            mappings: vec![
                SourceMapping {
                    generated_line: 1,
                    generated_column: 4,
                    source_index: Some(0),
                    original_line: Some(1),
                    original_column: Some(4),
                    name_index: None,
                }
            ],
            names: vec![],
        };
        
        registry.register_source_map(module_hash, source_map);
        
        let mut reporter = SourceMappedErrorReporter::new(registry);
        
        let error = reporter.report_error(
            "Test panic".to_string(),
            ErrorKind::RuntimeError { operation: "panic".to_string() },
            module_hash,
            GeneratedPosition { line: 1, column: 4 },
            vec![],
            ErrorContext {
                module_hash: Some(hex::encode(module_hash.as_bytes())),
                function_name: Some("main".to_string()),
                variable_values: std::collections::HashMap::new(),
                timestamp: "2023-01-01T00:00:00Z".to_string(),
                execution_context_id: None,
            },
        );
        
        assert_eq!(error.message, "Test panic");
        assert!(error.source_location.is_some());
        
        let report = reporter.format_error_report(&error);
        assert!(report.contains("Test panic"));
        assert!(report.contains("src/main.rs:2:5"));
    }
}