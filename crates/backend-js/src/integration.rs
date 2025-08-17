//! Integration module for JavaScript/TypeScript backend
//!
//! This module provides a unified interface for generating JavaScript code,
//! TypeScript definitions, and source maps from MIR modules.

use crate::{
    JsCodeGenerator, JsTarget, JsGenerationOptions, JsOutput,
    TypeScriptGenerator, TypeScriptOptions, TypeScriptOutput,
    SourceMapGenerator, SourceMap
};
use mir_ast::Module;
use std::collections::HashMap;

/// Unified JavaScript/TypeScript backend
pub struct JsBackend {
    js_generator: JsCodeGenerator,
    ts_generator: TypeScriptGenerator,
    sourcemap_generator: SourceMapGenerator,
    options: BackendOptions,
}

/// Options for the JavaScript/TypeScript backend
#[derive(Debug, Clone)]
pub struct BackendOptions {
    pub js_options: JsGenerationOptions,
    pub ts_options: TypeScriptOptions,
    pub emit_typescript: bool,
    pub emit_source_maps: bool,
    pub output_directory: Option<String>,
}

/// Complete output from the JavaScript/TypeScript backend
#[derive(Debug, Clone)]
pub struct BackendOutput {
    pub javascript: JsOutput,
    pub typescript: Option<TypeScriptOutput>,
    pub source_map: Option<SourceMap>,
    pub metadata: BackendMetadata,
}

/// Metadata about the backend compilation
#[derive(Debug, Clone)]
pub struct BackendMetadata {
    pub module_name: String,
    pub target: JsTarget,
    pub has_typescript: bool,
    pub has_source_maps: bool,
    pub compilation_time_ms: u64,
    pub optimizations_applied: Vec<String>,
}

impl Default for JsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl JsBackend {
    /// Create a new JavaScript/TypeScript backend with default options
    pub fn new() -> Self {
        JsBackend {
            js_generator: JsCodeGenerator::new(),
            ts_generator: TypeScriptGenerator::new(),
            sourcemap_generator: SourceMapGenerator::new(),
            options: BackendOptions::default(),
        }
    }

    /// Create a new backend with custom options
    pub fn with_options(options: BackendOptions) -> Self {
        JsBackend {
            js_generator: JsCodeGenerator::with_options(options.js_options.clone()),
            ts_generator: TypeScriptGenerator::with_options(options.ts_options.clone()),
            sourcemap_generator: SourceMapGenerator::new(),
            options,
        }
    }

    /// Compile a MIR module to JavaScript/TypeScript
    pub fn compile(&self, module: &Module) -> Result<BackendOutput, BackendError> {
        let start_time = std::time::Instant::now();
        
        // Generate JavaScript
        let javascript = self.js_generator.generate(module)
            .map_err(|e| BackendError::JavaScriptGeneration(e.to_string()))?;

        // Generate TypeScript definitions if requested
        let typescript = if self.options.emit_typescript {
            Some(self.ts_generator.generate_types(module)
                .map_err(|e| BackendError::TypeScriptGeneration(e.to_string()))?)
        } else {
            None
        };

        // Generate source maps if requested
        let source_map = if self.options.emit_source_maps {
            let mut sources = HashMap::new();
            sources.insert(
                format!("{}.mir", module.name),
                format!("// Original MIR module: {}", module.name)
            );
            
            Some(self.sourcemap_generator.generate_sourcemap(module, &javascript.code, &sources)
                .map_err(|e| BackendError::SourceMapGeneration(e.to_string()))?)
        } else {
            None
        };

        let compilation_time = start_time.elapsed().as_millis() as u64;

        let metadata = BackendMetadata {
            module_name: module.name.clone(),
            target: javascript.metadata.target,
            has_typescript: typescript.is_some(),
            has_source_maps: source_map.is_some(),
            compilation_time_ms: compilation_time,
            optimizations_applied: if javascript.metadata.optimized {
                vec!["dead_code_elimination".to_string(), "constant_folding".to_string()]
            } else {
                Vec::new()
            },
        };

        Ok(BackendOutput {
            javascript,
            typescript,
            source_map,
            metadata,
        })
    }

    /// Compile multiple modules as a bundle
    pub fn compile_bundle(&self, modules: &[Module]) -> Result<Vec<BackendOutput>, BackendError> {
        modules.iter()
            .map(|module| self.compile(module))
            .collect()
    }

    /// Write output to files
    pub fn write_output(&self, output: &BackendOutput, base_path: &str) -> Result<(), BackendError> {
        use std::fs;
        use std::path::Path;

        let base = Path::new(base_path);
        
        // Ensure output directory exists
        if let Some(parent) = base.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BackendError::FileSystem(e.to_string()))?;
        }

        // Write JavaScript file
        let js_path = base.with_extension("js");
        fs::write(&js_path, &output.javascript.code)
            .map_err(|e| BackendError::FileSystem(e.to_string()))?;

        // Write TypeScript definitions if present
        if let Some(ref ts_output) = output.typescript {
            let ts_path = base.with_extension("d.ts");
            fs::write(&ts_path, &ts_output.declarations)
                .map_err(|e| BackendError::FileSystem(e.to_string()))?;
        }

        // Write source map if present
        if let Some(ref source_map) = output.source_map {
            let map_path = base.with_extension("js.map");
            let map_json = serde_json::to_string_pretty(source_map)
                .map_err(|e| BackendError::Serialization(e.to_string()))?;
            fs::write(&map_path, map_json)
                .map_err(|e| BackendError::FileSystem(e.to_string()))?;
        }

        Ok(())
    }

    /// Set JavaScript target version
    pub fn set_js_target(&mut self, target: JsTarget) {
        self.js_generator.set_target(target);
        self.options.js_options.target = target;
    }

    /// Enable or disable minification
    pub fn set_minify(&mut self, minify: bool) {
        self.js_generator.set_minify(minify);
        self.options.js_options.minify = minify;
    }

    /// Enable or disable TypeScript generation
    pub fn set_emit_typescript(&mut self, emit: bool) {
        self.options.emit_typescript = emit;
    }

    /// Enable or disable source map generation
    pub fn set_emit_source_maps(&mut self, emit: bool) {
        self.js_generator.set_emit_source_maps(emit);
        self.options.emit_source_maps = emit;
    }

    /// Get current backend options
    pub fn options(&self) -> &BackendOptions {
        &self.options
    }

    /// Update backend options
    pub fn set_options(&mut self, options: BackendOptions) {
        self.js_generator = JsCodeGenerator::with_options(options.js_options.clone());
        self.ts_generator = TypeScriptGenerator::with_options(options.ts_options.clone());
        self.options = options;
    }
}

impl Default for BackendOptions {
    fn default() -> Self {
        BackendOptions {
            js_options: JsGenerationOptions::default(),
            ts_options: TypeScriptOptions::default(),
            emit_typescript: true,
            emit_source_maps: true,
            output_directory: None,
        }
    }
}

/// Errors that can occur in the JavaScript/TypeScript backend
#[derive(Debug, Clone)]
pub enum BackendError {
    JavaScriptGeneration(String),
    TypeScriptGeneration(String),
    SourceMapGeneration(String),
    FileSystem(String),
    Serialization(String),
    InvalidConfiguration(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::JavaScriptGeneration(msg) => write!(f, "JavaScript generation error: {}", msg),
            BackendError::TypeScriptGeneration(msg) => write!(f, "TypeScript generation error: {}", msg),
            BackendError::SourceMapGeneration(msg) => write!(f, "Source map generation error: {}", msg),
            BackendError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            BackendError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            BackendError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_ast::{Module, NodeId, Statement, Expression};
    use mir_types::Value;

    fn create_test_module() -> Module {
        let mut module = Module::new(NodeId::new(1), "test_module".to_string());
        
        // Add a simple function declaration
        module.statements.push(Statement::FunctionDeclaration {
            id: NodeId::new(2),
            name: "testFunction".to_string(),
            parameters: vec!["x".to_string(), "y".to_string()],
            body: vec![
                Statement::Expression {
                    id: NodeId::new(3),
                    expression: Expression::Literal {
                        id: NodeId::new(4),
                        value: Value::I32(42),
                    },
                },
            ],
        });
        
        module
    }

    #[test]
    fn test_backend_compilation() {
        let backend = JsBackend::new();
        let module = create_test_module();
        
        let result = backend.compile(&module);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(!output.javascript.code.is_empty());
        assert!(output.typescript.is_some());
        assert!(output.source_map.is_some());
        assert_eq!(output.metadata.module_name, "test_module");
    }

    #[test]
    fn test_backend_with_custom_options() {
        let mut options = BackendOptions::default();
        options.emit_typescript = false;
        options.emit_source_maps = false;
        options.js_options.minify = true;
        
        let backend = JsBackend::with_options(options);
        let module = create_test_module();
        
        let result = backend.compile(&module);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(!output.javascript.code.is_empty());
        assert!(output.typescript.is_none());
        assert!(output.source_map.is_none());
        assert!(output.javascript.metadata.minified);
    }

    #[test]
    fn test_bundle_compilation() {
        let backend = JsBackend::new();
        let modules = vec![
            create_test_module(),
            {
                let mut module = Module::new(NodeId::new(10), "second_module".to_string());
                module.statements.push(Statement::VariableDeclaration {
                    id: NodeId::new(11),
                    name: "testVar".to_string(),
                    value: Some(Expression::Literal {
                        id: NodeId::new(12),
                        value: Value::String("hello".to_string()),
                    }),
                });
                module
            },
        ];
        
        let result = backend.compile_bundle(&modules);
        assert!(result.is_ok());
        
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].metadata.module_name, "test_module");
        assert_eq!(outputs[1].metadata.module_name, "second_module");
    }

    #[test]
    fn test_target_configuration() {
        let mut backend = JsBackend::new();
        backend.set_js_target(JsTarget::ES5);
        backend.set_minify(true);
        backend.set_emit_typescript(false);
        
        let module = create_test_module();
        let result = backend.compile(&module);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.metadata.target, JsTarget::ES5);
        assert!(output.javascript.metadata.minified);
        assert!(output.typescript.is_none());
    }
}