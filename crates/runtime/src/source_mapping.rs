use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Source map information linking execution to original source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    /// Version of the source map format
    pub version: u32,
    /// Original source file paths
    pub sources: Vec<PathBuf>,
    /// Source content (optional, for embedded sources)
    pub sources_content: Option<Vec<Option<String>>>,
    /// Mapping data from generated to original positions
    pub mappings: Vec<SourceMapping>,
    /// Names referenced in the mappings
    pub names: Vec<String>,
}

/// Individual source mapping entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapping {
    /// Generated line number (0-based)
    pub generated_line: u32,
    /// Generated column number (0-based)
    pub generated_column: u32,
    /// Original source file index
    pub source_index: Option<u32>,
    /// Original line number (0-based)
    pub original_line: Option<u32>,
    /// Original column number (0-based)
    pub original_column: Option<u32>,
    /// Name index for identifiers
    pub name_index: Option<u32>,
}

/// Source position in original code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourcePosition {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub name: Option<String>,
}

/// Generated position in compiled code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeneratedPosition {
    pub line: u32,
    pub column: u32,
}

/// Source map registry for managing multiple source maps
pub struct SourceMapRegistry {
    /// Maps from module hash to source map
    source_maps: HashMap<crate::ContentHash, SourceMap>,
    /// Reverse mapping cache for performance
    reverse_mappings: HashMap<crate::ContentHash, Vec<(GeneratedPosition, SourcePosition)>>,
}

impl SourceMapRegistry {
    pub fn new() -> Self {
        Self {
            source_maps: HashMap::new(),
            reverse_mappings: HashMap::new(),
        }
    }

    /// Register a source map for a module
    pub fn register_source_map(&mut self, module_hash: crate::ContentHash, source_map: SourceMap) {
        // Build reverse mapping for efficient lookups
        let mut reverse_mapping = Vec::new();
        
        for mapping in &source_map.mappings {
            if let (Some(source_idx), Some(orig_line), Some(orig_col)) = 
                (mapping.source_index, mapping.original_line, mapping.original_column) {
                
                if let Some(source_file) = source_map.sources.get(source_idx as usize) {
                    let name = mapping.name_index
                        .and_then(|idx| source_map.names.get(idx as usize))
                        .cloned();
                    
                    let generated_pos = GeneratedPosition {
                        line: mapping.generated_line,
                        column: mapping.generated_column,
                    };
                    
                    let source_pos = SourcePosition {
                        file: source_file.clone(),
                        line: orig_line,
                        column: orig_col,
                        name,
                    };
                    
                    reverse_mapping.push((generated_pos, source_pos));
                }
            }
        }
        
        // Sort by generated position for binary search
        reverse_mapping.sort_by_key(|(gen_pos, _)| (gen_pos.line, gen_pos.column));
        
        self.reverse_mappings.insert(module_hash, reverse_mapping);
        self.source_maps.insert(module_hash, source_map);
    }

    /// Map generated position to original source position
    pub fn map_to_source(&self, module_hash: &crate::ContentHash, generated_pos: GeneratedPosition) -> Option<SourcePosition> {
        let reverse_mapping = self.reverse_mappings.get(module_hash)?;
        
        // Binary search for closest mapping
        let idx = reverse_mapping.binary_search_by_key(&(generated_pos.line, generated_pos.column), 
            |(gen_pos, _)| (gen_pos.line, gen_pos.column));
        
        match idx {
            Ok(exact_idx) => Some(reverse_mapping[exact_idx].1.clone()),
            Err(insert_idx) => {
                if insert_idx > 0 {
                    // Use the previous mapping as the closest
                    Some(reverse_mapping[insert_idx - 1].1.clone())
                } else {
                    None
                }
            }
        }
    }

    /// Get all source files for a module
    pub fn get_source_files(&self, module_hash: &crate::ContentHash) -> Option<&Vec<PathBuf>> {
        self.source_maps.get(module_hash).map(|sm| &sm.sources)
    }

    /// Get source content for a specific file
    pub fn get_source_content(&self, module_hash: &crate::ContentHash, source_index: usize) -> Option<&str> {
        let source_map = self.source_maps.get(module_hash)?;
        source_map.sources_content.as_ref()?
            .get(source_index)?
            .as_ref()
            .map(|s| s.as_str())
    }
}

impl Default for SourceMapRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map_registration() {
        let mut registry = SourceMapRegistry::new();
        let module_hash = crate::ContentHash::from_bytes([1; 8]);
        
        let source_map = SourceMap {
            version: 3,
            sources: vec![PathBuf::from("src/main.rs")],
            sources_content: Some(vec![Some("fn main() {}".to_string())]),
            mappings: vec![
                SourceMapping {
                    generated_line: 0,
                    generated_column: 0,
                    source_index: Some(0),
                    original_line: Some(0),
                    original_column: Some(0),
                    name_index: Some(0),
                }
            ],
            names: vec!["main".to_string()],
        };
        
        registry.register_source_map(module_hash, source_map);
        
        let mapped = registry.map_to_source(&module_hash, GeneratedPosition { line: 0, column: 0 });
        assert!(mapped.is_some());
        
        let pos = mapped.unwrap();
        assert_eq!(pos.file, PathBuf::from("src/main.rs"));
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
        assert_eq!(pos.name, Some("main".to_string()));
    }
}