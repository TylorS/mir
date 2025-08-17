//! NAPI bindings for WASI backend
//!
//! This module provides Node.js NAPI bindings for the WASI backend,
//! enabling JavaScript/TypeScript integration with WASI components.

use crate::WasiCodeGenerator;
use crate::codegen::{ResourceConfig, SecurityPolicy};
use crate::component::{ComponentCapabilities, ComponentMetadata};
use mir_ast::Module;
use serde::{Deserialize, Serialize};

/// NAPI bindings for WASI backend
pub struct WasiNapiBindings {
    generator: WasiCodeGenerator,
    runtime_config: WasiRuntimeConfig,
}

/// WASI runtime configuration for NAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiRuntimeConfig {
    pub enable_debugging: bool,
    pub enable_profiling: bool,
    pub memory_limit: Option<usize>,
    pub execution_timeout: Option<u64>, // milliseconds
    pub allowed_imports: Vec<String>,
    pub security_policy: SecurityPolicy,
}

/// JavaScript-compatible WASI component representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsWasiComponent {
    pub metadata: ComponentMetadata,
    pub wit_definition: String,
    pub binary_data: Vec<u8>,
    pub capabilities: ComponentCapabilities,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

/// NAPI compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub success: bool,
    pub component: Option<JsWasiComponent>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub compilation_time_ms: u64,
}

/// NAPI validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub security_issues: Vec<String>,
}

impl Default for WasiNapiBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiNapiBindings {
    /// Create new NAPI bindings with default configuration
    pub fn new() -> Self {
        WasiNapiBindings {
            generator: WasiCodeGenerator::new(),
            runtime_config: WasiRuntimeConfig::default(),
        }
    }
    
    /// Create new NAPI bindings with custom configuration
    pub fn with_config(config: WasiRuntimeConfig) -> Self {
        let resource_config = ResourceConfig {
            security_policy: config.security_policy.clone(),
            ..ResourceConfig::default()
        };
        
        WasiNapiBindings {
            generator: WasiCodeGenerator::with_config(
                ComponentCapabilities::default(),
                resource_config,
            ),
            runtime_config: config,
        }
    }
    
    /// Compile MIR module to WASI component (JavaScript API)
    pub fn compile_module(&self, module_json: &str) -> CompilationResult {
        let start_time = std::time::Instant::now();
        
        // Parse module from JSON
        let module: Module = match serde_json::from_str(module_json) {
            Ok(m) => m,
            Err(e) => {
                return CompilationResult {
                    success: false,
                    component: None,
                    errors: vec![format!("Failed to parse module JSON: {}", e)],
                    warnings: Vec::new(),
                    compilation_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };
        
        // Generate WASI component
        match self.generator.generate(&module) {
            Ok(component) => {
                // Generate WIT definition
                let wit_definition = self.generator.generate_wit(&component);
                
                // Generate binary
                let binary_data = match self.generator.generate_binary(&component) {
                    Ok(binary) => binary,
                    Err(e) => {
                        return CompilationResult {
                            success: false,
                            component: None,
                            errors: vec![format!("Binary generation failed: {}", e)],
                            warnings: Vec::new(),
                            compilation_time_ms: start_time.elapsed().as_millis() as u64,
                        };
                    }
                };
                
                let js_component = JsWasiComponent {
                    metadata: component.metadata.clone(),
                    wit_definition,
                    binary_data,
                    capabilities: component.capabilities.clone(),
                    exports: component.exports.iter().map(|e| e.name.clone()).collect(),
                    imports: component.imports.iter().map(|i| i.name.clone()).collect(),
                };
                
                CompilationResult {
                    success: true,
                    component: Some(js_component),
                    errors: Vec::new(),
                    warnings: Vec::new(),
                    compilation_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            Err(e) => {
                CompilationResult {
                    success: false,
                    component: None,
                    errors: vec![format!("Compilation failed: {}", e)],
                    warnings: Vec::new(),
                    compilation_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
        }
    }
    
    /// Validate WASI component (JavaScript API)
    pub fn validate_component(&self, component_json: &str) -> ValidationResult {
        // Parse component from JSON
        let component: JsWasiComponent = match serde_json::from_str(component_json) {
            Ok(c) => c,
            Err(e) => {
                return ValidationResult {
                    is_valid: false,
                    errors: vec![format!("Failed to parse component JSON: {}", e)],
                    warnings: Vec::new(),
                    security_issues: Vec::new(),
                };
            }
        };
        
        let mut errors = Vec::new();
        let warnings = Vec::new();
        let mut security_issues = Vec::new();
        
        // Validate WIT definition
        if component.wit_definition.is_empty() {
            errors.push("WIT definition is empty".to_string());
        }
        
        // Validate binary data
        if component.binary_data.len() < 8 {
            errors.push("Binary data is too small to be a valid WASM component".to_string());
        } else {
            // Check WASM magic number
            let magic = &component.binary_data[0..4];
            if magic != [0x00, 0x61, 0x73, 0x6d] {
                errors.push("Invalid WASM magic number".to_string());
            }
        }
        
        // Validate capabilities
        self.validate_capabilities(&component.capabilities, &mut security_issues);
        
        // Check for potential security issues
        if component.capabilities.filesystem.can_write && component.capabilities.network.can_connect {
            security_issues.push("Component has both filesystem write and network access".to_string());
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            security_issues,
        }
    }
    
    /// Generate TypeScript definitions for WASI component
    pub fn generate_typescript_definitions(&self, component_json: &str) -> Result<String, String> {
        let component: JsWasiComponent = serde_json::from_str(component_json)
            .map_err(|e| format!("Failed to parse component JSON: {}", e))?;
        
        let mut ts_def = String::new();
        
        // Generate interface for component
        ts_def.push_str(&format!("// TypeScript definitions for WASI component: {}\n\n", component.metadata.name));
        
        ts_def.push_str("export interface WasiComponent {\n");
        ts_def.push_str("  metadata: ComponentMetadata;\n");
        ts_def.push_str("  instantiate(): Promise<ComponentInstance>;\n");
        ts_def.push_str("  validate(): ValidationResult;\n");
        ts_def.push_str("}\n\n");
        
        ts_def.push_str("export interface ComponentMetadata {\n");
        ts_def.push_str("  name: string;\n");
        ts_def.push_str("  version: string;\n");
        ts_def.push_str("  description?: string;\n");
        ts_def.push_str("  authors: string[];\n");
        ts_def.push_str("  license?: string;\n");
        ts_def.push_str("}\n\n");
        
        ts_def.push_str("export interface ComponentInstance {\n");
        for export in &component.exports {
            ts_def.push_str(&format!("  {}: (...args: any[]) => any;\n", export));
        }
        ts_def.push_str("}\n\n");
        
        ts_def.push_str("export interface ValidationResult {\n");
        ts_def.push_str("  isValid: boolean;\n");
        ts_def.push_str("  errors: string[];\n");
        ts_def.push_str("  warnings: string[];\n");
        ts_def.push_str("  securityIssues: string[];\n");
        ts_def.push_str("}\n");
        
        Ok(ts_def)
    }
    
    /// Create JavaScript bindings code
    pub fn create_bindings(&self) -> String {
        let mut bindings = String::new();
        
        bindings.push_str("// NAPI bindings for WASI backend\n");
        bindings.push_str("const { WasiNapiBindings } = require('./native');\n\n");
        
        bindings.push_str("class WasiCompiler {\n");
        bindings.push_str("  constructor(config = {}) {\n");
        bindings.push_str("    this.bindings = new WasiNapiBindings(config);\n");
        bindings.push_str("  }\n\n");
        
        bindings.push_str("  compileModule(moduleJson) {\n");
        bindings.push_str("    return this.bindings.compileModule(moduleJson);\n");
        bindings.push_str("  }\n\n");
        
        bindings.push_str("  validateComponent(componentJson) {\n");
        bindings.push_str("    return this.bindings.validateComponent(componentJson);\n");
        bindings.push_str("  }\n\n");
        
        bindings.push_str("  generateTypeScriptDefinitions(componentJson) {\n");
        bindings.push_str("    return this.bindings.generateTypeScriptDefinitions(componentJson);\n");
        bindings.push_str("  }\n");
        bindings.push_str("}\n\n");
        
        bindings.push_str("module.exports = { WasiCompiler };\n");
        
        bindings
    }
    
    /// Get runtime configuration
    pub fn get_config(&self) -> &WasiRuntimeConfig {
        &self.runtime_config
    }
    
    /// Update runtime configuration
    pub fn set_config(&mut self, config: WasiRuntimeConfig) {
        self.runtime_config = config;
    }
    
    fn validate_capabilities(&self, capabilities: &ComponentCapabilities, security_issues: &mut Vec<String>) {
        // Check filesystem capabilities
        if capabilities.filesystem.can_delete && capabilities.filesystem.allowed_paths.is_empty() {
            security_issues.push("Component can delete files but has no path restrictions".to_string());
        }
        
        // Check network capabilities
        if capabilities.network.can_listen && capabilities.network.allowed_ports.is_empty() {
            security_issues.push("Component can listen on network but has no port restrictions".to_string());
        }
        
        // Check environment capabilities
        if capabilities.environment.can_write_env && capabilities.environment.allowed_variables.is_empty() {
            security_issues.push("Component can write environment variables but has no restrictions".to_string());
        }
    }
}

impl Default for WasiRuntimeConfig {
    fn default() -> Self {
        WasiRuntimeConfig {
            enable_debugging: false,
            enable_profiling: false,
            memory_limit: Some(64 * 1024 * 1024), // 64MB
            execution_timeout: Some(30000), // 30 seconds
            allowed_imports: vec![
                "wasi:filesystem".to_string(),
                "wasi:sockets".to_string(),
                "wasi:clocks".to_string(),
                "wasi:random".to_string(),
            ],
            security_policy: SecurityPolicy {
                enforce_capabilities: true,
                allow_unsafe_operations: false,
                require_explicit_permissions: true,
                sandbox_resources: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_ast::{Module, NodeId};
    
    #[test]
    fn test_napi_bindings_creation() {
        let bindings = WasiNapiBindings::new();
        assert!(bindings.runtime_config.enable_debugging == false);
    }
    
    #[test]
    fn test_module_compilation() {
        let bindings = WasiNapiBindings::new();
        let module = Module::new(NodeId::new(1), "test_module".to_string());
        let module_json = serde_json::to_string(&module).unwrap();
        
        let result = bindings.compile_module(&module_json);
        assert!(result.success);
        assert!(result.component.is_some());
    }
    
    #[test]
    fn test_typescript_generation() {
        let bindings = WasiNapiBindings::new();
        let component = JsWasiComponent {
            metadata: ComponentMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: Vec::new(),
                license: None,
                content_hash: mir_types::ContentHash::new(b"test"),
            },
            wit_definition: "".to_string(),
            binary_data: Vec::new(),
            capabilities: ComponentCapabilities::default(),
            exports: vec!["testFunction".to_string()],
            imports: Vec::new(),
        };
        
        let component_json = serde_json::to_string(&component).unwrap();
        let ts_def = bindings.generate_typescript_definitions(&component_json).unwrap();
        
        assert!(ts_def.contains("export interface WasiComponent"));
        assert!(ts_def.contains("testFunction"));
    }
    
    #[test]
    fn test_bindings_generation() {
        let bindings = WasiNapiBindings::new();
        let js_bindings = bindings.create_bindings();
        
        assert!(js_bindings.contains("class WasiCompiler"));
        assert!(js_bindings.contains("compileModule"));
        assert!(js_bindings.contains("validateComponent"));
    }
}