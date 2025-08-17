//! Source map generation for JavaScript output
//!
//! This module generates source maps that link generated JavaScript back to the original
//! MIR source code, enabling debugging and profiling of the original source.

use mir_ast::{Module, Statement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source map generator for JavaScript output
pub struct SourceMapGenerator {
    version: u8,
    include_sources_content: bool,
    include_names: bool,
}

/// Source map data structure (Version 3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub version: u8,
    pub file: Option<String>,
    pub source_root: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Option<Vec<Option<String>>>,
    pub names: Vec<String>,
    pub mappings: String,
}

/// A single mapping entry
#[derive(Debug, Clone)]
pub struct Mapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub source_index: Option<u32>,
    pub original_line: Option<u32>,
    pub original_column: Option<u32>,
    pub name_index: Option<u32>,
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Context for source map generation
pub struct SourceMapContext {
    sources: Vec<String>,
    sources_content: Vec<Option<String>>,
    names: Vec<String>,
    mappings: Vec<Mapping>,
    source_index_map: HashMap<String, u32>,
    name_index_map: HashMap<String, u32>,
}

impl Default for SourceMapGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMapGenerator {
    /// Create a new source map generator
    pub fn new() -> Self {
        SourceMapGenerator {
            version: 3,
            include_sources_content: true,
            include_names: true,
        }
    }

    /// Generate a source map from MIR module to JavaScript
    pub fn generate_sourcemap(
        &self,
        module: &Module,
        generated_code: &str,
        original_sources: &HashMap<String, String>,
    ) -> Result<SourceMap, SourceMapError> {
        let mut context = SourceMapContext::new();
        
        // Add original sources
        for (file_path, content) in original_sources {
            context.add_source(file_path.clone(), Some(content.clone()));
        }
        
        // Generate mappings by analyzing the module
        self.generate_mappings(module, generated_code, &mut context)?;
        
        // Build the final source map
        let mappings_string = self.encode_mappings(&context.mappings)?;
        
        Ok(SourceMap {
            version: self.version,
            file: Some(format!("{}.js", module.name)),
            source_root: None,
            sources: context.sources,
            sources_content: if self.include_sources_content {
                Some(context.sources_content)
            } else {
                None
            },
            names: if self.include_names {
                context.names
            } else {
                Vec::new()
            },
            mappings: mappings_string,
        })
    }

    /// Generate mappings between original and generated code
    fn generate_mappings(
        &self,
        module: &Module,
        generated_code: &str,
        context: &mut SourceMapContext,
    ) -> Result<(), SourceMapError> {
        let _generated_lines: Vec<&str> = generated_code.lines().collect();
        
        // For now, create a simple mapping strategy
        // In a full implementation, this would track exact positions during code generation
        
        let mut current_line = 0u32;
        
        // Map module-level constructs
        for statement in &module.statements {
            if let Some(mapping) = self.create_statement_mapping(statement, current_line, context)? {
                context.mappings.push(mapping);
            }
            current_line += 1;
        }
        
        Ok(())
    }

    /// Create a mapping for a statement
    fn create_statement_mapping(
        &self,
        statement: &Statement,
        generated_line: u32,
        context: &mut SourceMapContext,
    ) -> Result<Option<Mapping>, SourceMapError> {
        // For now, create basic mappings
        // In a full implementation, this would use actual source location information
        
        match statement {
            Statement::FunctionDeclaration { name, .. } => {
                let name_index = context.add_name(name.clone());
                
                Ok(Some(Mapping {
                    generated_line,
                    generated_column: 0,
                    source_index: Some(0), // Assume first source file
                    original_line: Some(generated_line), // Simplified mapping
                    original_column: Some(0),
                    name_index: Some(name_index),
                }))
            }
            
            Statement::VariableDeclaration { name, .. } => {
                let name_index = context.add_name(name.clone());
                
                Ok(Some(Mapping {
                    generated_line,
                    generated_column: 0,
                    source_index: Some(0),
                    original_line: Some(generated_line),
                    original_column: Some(0),
                    name_index: Some(name_index),
                }))
            }
            
            _ => Ok(Some(Mapping {
                generated_line,
                generated_column: 0,
                source_index: Some(0),
                original_line: Some(generated_line),
                original_column: Some(0),
                name_index: None,
            }))
        }
    }

    /// Encode mappings using VLQ (Variable Length Quantity) encoding
    fn encode_mappings(&self, mappings: &[Mapping]) -> Result<String, SourceMapError> {
        let mut encoded = String::new();
        let mut previous_generated_line = 0u32;
        let mut previous_generated_column = 0u32;
        let mut previous_source_index = 0u32;
        let mut previous_original_line = 0u32;
        let mut previous_original_column = 0u32;
        let mut previous_name_index = 0u32;
        
        for mapping in mappings {
            // Handle line breaks
            while previous_generated_line < mapping.generated_line {
                encoded.push(';');
                previous_generated_line += 1;
                previous_generated_column = 0;
            }
            
            if !encoded.is_empty() && !encoded.ends_with(';') {
                encoded.push(',');
            }
            
            // Encode generated column (always present)
            encoded.push_str(&self.encode_vlq(
                mapping.generated_column as i32 - previous_generated_column as i32
            ));
            previous_generated_column = mapping.generated_column;
            
            // Encode source information if present
            if let (Some(source_index), Some(original_line), Some(original_column)) = 
                (mapping.source_index, mapping.original_line, mapping.original_column) {
                
                // Source index
                encoded.push_str(&self.encode_vlq(
                    source_index as i32 - previous_source_index as i32
                ));
                previous_source_index = source_index;
                
                // Original line
                encoded.push_str(&self.encode_vlq(
                    original_line as i32 - previous_original_line as i32
                ));
                previous_original_line = original_line;
                
                // Original column
                encoded.push_str(&self.encode_vlq(
                    original_column as i32 - previous_original_column as i32
                ));
                previous_original_column = original_column;
                
                // Name index if present
                if let Some(name_index) = mapping.name_index {
                    encoded.push_str(&self.encode_vlq(
                        name_index as i32 - previous_name_index as i32
                    ));
                    previous_name_index = name_index;
                }
            }
        }
        
        Ok(encoded)
    }

    /// Encode a number using Variable Length Quantity (VLQ) encoding
    fn encode_vlq(&self, value: i32) -> String {
        let mut vlq = if value < 0 { ((-value) << 1) | 1 } else { value << 1 };
        let mut encoded = String::new();
        
        loop {
            let mut digit = vlq & 0x1F;
            vlq >>= 5;
            
            if vlq > 0 {
                digit |= 0x20; // Continuation bit
            }
            
            encoded.push(self.base64_encode_digit(digit as u8));
            
            if vlq == 0 {
                break;
            }
        }
        
        encoded
    }

    /// Encode a 6-bit digit as Base64
    fn base64_encode_digit(&self, digit: u8) -> char {
        match digit {
            0..=25 => (b'A' + digit) as char,
            26..=51 => (b'a' + digit - 26) as char,
            52..=61 => (b'0' + digit - 52) as char,
            62 => '+',
            63 => '/',
            _ => panic!("Invalid Base64 digit: {}", digit),
        }
    }

    /// Set whether to include source content in the source map
    pub fn set_include_sources_content(&mut self, include: bool) {
        self.include_sources_content = include;
    }

    /// Set whether to include names in the source map
    pub fn set_include_names(&mut self, include: bool) {
        self.include_names = include;
    }

    /// Generate a simple source map for basic debugging (legacy method)
    pub fn generate_simple_sourcemap(&self, original_source: &str, _generated_code: &str) -> String {
        // Create a basic source map for backward compatibility
        let source_map = SourceMap {
            version: 3,
            file: Some("generated.js".to_string()),
            source_root: None,
            sources: vec!["original.mir".to_string()],
            sources_content: Some(vec![Some(original_source.to_string())]),
            names: Vec::new(),
            mappings: "AAAA".to_string(), // Basic mapping
        };
        
        serde_json::to_string(&source_map).unwrap_or_else(|_| "{}".to_string())
    }
}

impl SourceMapContext {
    fn new() -> Self {
        SourceMapContext {
            sources: Vec::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            mappings: Vec::new(),
            source_index_map: HashMap::new(),
            name_index_map: HashMap::new(),
        }
    }

    fn add_source(&mut self, source: String, content: Option<String>) -> u32 {
        if let Some(&index) = self.source_index_map.get(&source) {
            return index;
        }
        
        let index = self.sources.len() as u32;
        self.sources.push(source.clone());
        self.sources_content.push(content);
        self.source_index_map.insert(source, index);
        index
    }

    fn add_name(&mut self, name: String) -> u32 {
        if let Some(&index) = self.name_index_map.get(&name) {
            return index;
        }
        
        let index = self.names.len() as u32;
        self.names.push(name.clone());
        self.name_index_map.insert(name, index);
        index
    }
}

/// Errors that can occur during source map generation
#[derive(Debug, Clone)]
pub enum SourceMapError {
    EncodingError(String),
    InvalidMapping(String),
    SerializationError(String),
}

impl std::fmt::Display for SourceMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceMapError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            SourceMapError::InvalidMapping(msg) => write!(f, "Invalid mapping: {}", msg),
            SourceMapError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for SourceMapError {}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_ast::{Module, NodeId};

    #[test]
    fn test_source_map_generation() {
        let generator = SourceMapGenerator::new();
        let module = Module::new(NodeId::new(1), "test_module".to_string());
        
        let mut sources = HashMap::new();
        sources.insert("test.mir".to_string(), "// Original MIR source".to_string());
        
        let result = generator.generate_sourcemap(&module, "// Generated JS", &sources);
        assert!(result.is_ok());
        
        let source_map = result.unwrap();
        assert_eq!(source_map.version, 3);
        assert_eq!(source_map.sources.len(), 1);
        assert_eq!(source_map.sources[0], "test.mir");
    }

    #[test]
    fn test_vlq_encoding() {
        let generator = SourceMapGenerator::new();
        
        // Test basic VLQ encoding
        assert_eq!(generator.encode_vlq(0), "A");
        assert_eq!(generator.encode_vlq(1), "C");
        assert_eq!(generator.encode_vlq(-1), "D");
        assert_eq!(generator.encode_vlq(15), "e");
        assert_eq!(generator.encode_vlq(-15), "f");
    }

    #[test]
    fn test_base64_encoding() {
        let generator = SourceMapGenerator::new();
        
        assert_eq!(generator.base64_encode_digit(0), 'A');
        assert_eq!(generator.base64_encode_digit(25), 'Z');
        assert_eq!(generator.base64_encode_digit(26), 'a');
        assert_eq!(generator.base64_encode_digit(51), 'z');
        assert_eq!(generator.base64_encode_digit(52), '0');
        assert_eq!(generator.base64_encode_digit(61), '9');
        assert_eq!(generator.base64_encode_digit(62), '+');
        assert_eq!(generator.base64_encode_digit(63), '/');
    }

    #[test]
    fn test_simple_sourcemap_generation() {
        let generator = SourceMapGenerator::new();
        let source_map = generator.generate_simple_sourcemap(
            "// Original source",
            "// Generated code"
        );
        
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&source_map).unwrap();
        assert_eq!(parsed["version"], 3);
    }
}