//! WASI code generation
//!
//! This module implements WASI component model compilation from MIR IR,
//! including interface type generation and capability-based security.

use crate::component::{
    WasiComponent, ComponentInterface, InterfaceFunction, FunctionParameter,
    WasiType, ComponentCapabilities, ComponentExport,
    ComponentExportType, ExportVisibility, ComponentMetadata, ComponentWorld,
    WorldExport, WorldItemType, InterfaceType, WasiTypeDefinition,
    ResourceLifecycle, CreationPolicy,
    OwnershipModel, CleanupStrategy, SharingPolicy,
};
use mir_ast::{Module, Statement, Expression, ExportDeclaration, ASTNode};
use mir_types::{
    ContentHash, TypeHash, FunctionType, FunctionSignature,
    ResourceType,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// WASI code generator that compiles MIR IR to WASI components
pub struct WasiCodeGenerator {
    /// Type mapping from MIR types to WASI types
    type_mappings: HashMap<TypeHash, WasiType>,
    /// Component capabilities configuration
    default_capabilities: ComponentCapabilities,
    /// Resource management configuration
    #[allow(dead_code)]
    resource_config: ResourceConfig,
}

/// Resource management configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub default_lifecycle: ResourceLifecycle,
    pub security_policy: SecurityPolicy,
    pub resource_limits: ResourceLimits,
}

/// Security policy for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub enforce_capabilities: bool,
    pub allow_unsafe_operations: bool,
    pub require_explicit_permissions: bool,
    pub sandbox_resources: bool,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_file_handles: Option<usize>,
    pub max_network_connections: Option<usize>,
    pub max_memory_per_resource: Option<usize>,
    #[serde(with = "duration_option_serde")]
    pub max_cpu_time_per_operation: Option<std::time::Duration>,
}

mod duration_option_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => d.as_millis().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis: Option<u128> = Option::deserialize(deserializer)?;
        Ok(millis.map(|m| Duration::from_millis(m as u64)))
    }
}

/// WASI compilation context
pub struct CompilationContext {
    pub module: Module,
    pub component: WasiComponent,
    pub type_registry: HashMap<TypeHash, String>, // Simplified to avoid trait object issues
    pub function_registry: HashMap<String, FunctionType>,
    pub resource_registry: HashMap<String, ResourceType>,
    pub capability_requirements: HashSet<String>,
}

impl Default for WasiCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiCodeGenerator {
    /// Create a new WASI code generator
    pub fn new() -> Self {
        WasiCodeGenerator {
            type_mappings: Self::create_default_type_mappings(),
            default_capabilities: ComponentCapabilities::default(),
            resource_config: ResourceConfig::default(),
        }
    }
    
    /// Create a new WASI code generator with custom configuration
    pub fn with_config(
        capabilities: ComponentCapabilities,
        resource_config: ResourceConfig,
    ) -> Self {
        WasiCodeGenerator {
            type_mappings: Self::create_default_type_mappings(),
            default_capabilities: capabilities,
            resource_config,
        }
    }
    
    /// Generate WASI component from MIR module
    pub fn generate(&self, module: &Module) -> Result<WasiComponent, CodegenError> {
        let mut context = CompilationContext {
            module: module.clone(),
            component: WasiComponent::new(),
            type_registry: HashMap::new(),
            function_registry: HashMap::new(),
            resource_registry: HashMap::new(),
            capability_requirements: HashSet::new(),
        };
        
        // Set component metadata
        self.generate_component_metadata(&mut context)?;
        
        // Generate interfaces from module exports
        self.generate_interfaces(&mut context)?;
        
        // Generate world definition
        self.generate_world(&mut context)?;
        
        // Generate resources
        self.generate_resources(&mut context)?;
        
        // Analyze and set capabilities
        self.analyze_capabilities(&mut context)?;
        
        // Generate component exports
        self.generate_exports(&mut context)?;
        
        Ok(context.component)
    }
    
    /// Generate component binary (WASM component)
    pub fn generate_binary(&self, component: &WasiComponent) -> Result<Vec<u8>, CodegenError> {
        // Generate WIT definition
        let _wit = component.generate_wit();
        
        // For now, return a placeholder WASM component binary
        // In a full implementation, this would use a WASM component compiler
        let mut binary = Vec::new();
        
        // WASM magic number
        binary.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d]);
        // WASM version
        binary.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        
        // Component-specific sections would be added here
        // This is a simplified placeholder
        
        Ok(binary)
    }
    
    /// Generate WIT (WebAssembly Interface Types) definition
    pub fn generate_wit(&self, component: &WasiComponent) -> String {
        component.generate_wit()
    }
    
    fn generate_component_metadata(&self, context: &mut CompilationContext) -> Result<(), CodegenError> {
        context.component.metadata = ComponentMetadata {
            name: context.module.name.clone(),
            version: "0.1.0".to_string(),
            description: Some(format!("WASI component generated from MIR module {}", context.module.name)),
            authors: vec!["MIR Compiler".to_string()],
            license: Some("MIT".to_string()),
            content_hash: context.module.content_hash(),
        };
        
        Ok(())
    }
    
    fn generate_interfaces(&self, context: &mut CompilationContext) -> Result<(), CodegenError> {
        // Generate main interface from module exports
        let mut main_interface = ComponentInterface {
            name: format!("{}-interface", context.module.name),
            namespace: Some("mir".to_string()),
            functions: Vec::new(),
            types: Vec::new(),
            resources: Vec::new(),
        };
        
        // Clone exports to avoid borrowing issues
        let exports = context.module.exports.clone();
        
        // Process module exports
        for export in &exports {
            if export.is_type {
                // Generate interface type
                let interface_type = self.generate_interface_type(export)?;
                main_interface.types.push(interface_type);
            } else {
                // Generate interface function
                let interface_function = self.generate_interface_function(export, &context.function_registry)?;
                main_interface.functions.push(interface_function);
            }
        }
        
        context.component.add_interface(main_interface);
        Ok(())
    }
    
    fn generate_interface_type(&self, export: &ExportDeclaration) -> Result<InterfaceType, CodegenError> {
        // Map MIR type to WASI type definition
        let type_definition = WasiTypeDefinition::Alias(WasiType::String); // Placeholder
        
        Ok(InterfaceType {
            name: export.name.clone(),
            definition: type_definition,
            documentation: Some(format!("Type exported from MIR module")),
        })
    }
    
    fn generate_interface_function(&self, export: &ExportDeclaration, function_registry: &HashMap<String, FunctionType>) -> Result<InterfaceFunction, CodegenError> {
        // Look up function type from registry or infer from export
        let function_signature = function_registry
            .get(&export.name)
            .map(|f| &f.signature)
            .cloned()
            .unwrap_or_else(|| {
                // Default function signature
                FunctionSignature {
                    parameter_types: Vec::new(),
                    return_type: TypeHash::new(ContentHash::new(b"unit")),
                    is_pure: true,
                    is_async: false,
                }
            });
        
        let parameters = function_signature.parameter_types
            .iter()
            .enumerate()
            .map(|(i, type_hash)| {
                FunctionParameter {
                    name: format!("param_{}", i),
                    param_type: self.map_type_hash_to_wasi_type(type_hash),
                    is_optional: false,
                    default_value: None,
                }
            })
            .collect();
        
        let return_type = if function_signature.return_type == TypeHash::new(ContentHash::new(b"unit")) {
            None
        } else {
            Some(self.map_type_hash_to_wasi_type(&function_signature.return_type))
        };
        
        Ok(InterfaceFunction {
            name: export.name.clone(),
            parameters,
            return_type,
            is_async: function_signature.is_async,
            error_type: None,
            documentation: Some(format!("Function exported from MIR module")),
        })
    }
    
    fn generate_world(&self, context: &mut CompilationContext) -> Result<(), CodegenError> {
        context.component.world = ComponentWorld {
            name: format!("{}-world", context.module.name),
            imports: Vec::new(), // Generated from module imports
            exports: context.component.interfaces.iter().map(|interface| {
                WorldExport {
                    name: interface.name.clone(),
                    interface: Some(interface.name.clone()),
                    item_type: WorldItemType::Interface(interface.name.clone()),
                }
            }).collect(),
            includes: Vec::new(),
        };
        
        Ok(())
    }
    
    fn generate_resources(&self, context: &mut CompilationContext) -> Result<(), CodegenError> {
        // Clone statements to avoid borrowing issues
        let statements = context.module.statements.clone();
        
        // Scan module for resource usage
        for statement in &statements {
            self.analyze_statement_for_resources(statement, context)?;
        }
        
        Ok(())
    }
    
    fn analyze_statement_for_resources(&self, statement: &Statement, context: &mut CompilationContext) -> Result<(), CodegenError> {
        match statement {
            Statement::Expression { expression, .. } => {
                self.analyze_expression_for_resources(expression, context)?;
            },
            Statement::VariableDeclaration { value: Some(value), .. } => {
                self.analyze_expression_for_resources(value, context)?;
            },
            Statement::FunctionDeclaration { body, .. } => {
                for stmt in body {
                    self.analyze_statement_for_resources(stmt, context)?;
                }
            },
            _ => {
                // Other statement types don't typically involve resources
            }
        }
        
        Ok(())
    }
    
    fn analyze_expression_for_resources(&self, expression: &Expression, context: &mut CompilationContext) -> Result<(), CodegenError> {
        match expression {
            Expression::FunctionCall { function: _, arguments, .. } => {
                // Analyze function calls for resource usage
                for arg in arguments {
                    self.analyze_expression_for_resources(arg, context)?;
                }
                
                // Check if this is a resource-related function call
                context.capability_requirements.insert("function-call".to_string());
            },
            // Note: MemberAccess and IndexAccess don't exist in the current AST
            // These would be handled by FunctionCall or other expression types
            _ => {
                // Other expression types
            }
        }
        
        Ok(())
    }
    
    fn analyze_capabilities(&self, context: &mut CompilationContext) -> Result<(), CodegenError> {
        let mut capabilities = self.default_capabilities.clone();
        
        // Analyze capability requirements based on detected patterns
        for requirement in &context.capability_requirements {
            match requirement.as_str() {
                "function-call" => {
                    // Basic function calls don't require special capabilities
                },
                "file-access" => {
                    capabilities.filesystem.can_read = true;
                },
                "network-access" => {
                    capabilities.network.can_connect = true;
                },
                _ => {
                    // Unknown capability requirement
                }
            }
        }
        
        context.component.set_capabilities(capabilities);
        Ok(())
    }
    
    fn generate_exports(&self, context: &mut CompilationContext) -> Result<(), CodegenError> {
        for export_decl in &context.module.exports {
            let export = ComponentExport {
                name: export_decl.name.clone(),
                export_type: if export_decl.is_type {
                    ComponentExportType::Type(InterfaceType {
                        name: export_decl.name.clone(),
                        definition: WasiTypeDefinition::Alias(WasiType::String),
                        documentation: None,
                    })
                } else {
                    ComponentExportType::Function(InterfaceFunction {
                        name: export_decl.name.clone(),
                        parameters: Vec::new(),
                        return_type: None,
                        is_async: false,
                        error_type: None,
                        documentation: None,
                    })
                },
                visibility: match export_decl.visibility {
                    mir_ast::ExportVisibility::Public => ExportVisibility::Public,
                    mir_ast::ExportVisibility::Protected => ExportVisibility::Protected,
                    mir_ast::ExportVisibility::Internal => ExportVisibility::Private,
                },
            };
            
            context.component.add_export(export);
        }
        
        Ok(())
    }
    
    fn map_type_hash_to_wasi_type(&self, type_hash: &TypeHash) -> WasiType {
        self.type_mappings.get(type_hash)
            .cloned()
            .unwrap_or(WasiType::String) // Default fallback
    }
    
    fn create_default_type_mappings() -> HashMap<TypeHash, WasiType> {
        let mut mappings = HashMap::new();
        
        // Map common MIR types to WASI types
        mappings.insert(TypeHash::new(ContentHash::new(b"bool")), WasiType::Bool);
        mappings.insert(TypeHash::new(ContentHash::new(b"i32")), WasiType::S32);
        mappings.insert(TypeHash::new(ContentHash::new(b"i64")), WasiType::S64);
        mappings.insert(TypeHash::new(ContentHash::new(b"u32")), WasiType::U32);
        mappings.insert(TypeHash::new(ContentHash::new(b"u64")), WasiType::U64);
        mappings.insert(TypeHash::new(ContentHash::new(b"f32")), WasiType::F32);
        mappings.insert(TypeHash::new(ContentHash::new(b"f64")), WasiType::F64);
        mappings.insert(TypeHash::new(ContentHash::new(b"string")), WasiType::String);
        mappings.insert(TypeHash::new(ContentHash::new(b"unit")), WasiType::Tuple(Vec::new()));
        
        mappings
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        ResourceConfig {
            default_lifecycle: ResourceLifecycle {
                creation_policy: CreationPolicy::PerRequest,
                ownership_model: OwnershipModel::Owned,
                cleanup_strategy: CleanupStrategy::RAII,
                sharing_policy: SharingPolicy::Exclusive,
            },
            security_policy: SecurityPolicy {
                enforce_capabilities: true,
                allow_unsafe_operations: false,
                require_explicit_permissions: true,
                sandbox_resources: true,
            },
            resource_limits: ResourceLimits {
                max_file_handles: Some(100),
                max_network_connections: Some(50),
                max_memory_per_resource: Some(1024 * 1024), // 1MB
                max_cpu_time_per_operation: Some(std::time::Duration::from_secs(30)),
            },
        }
    }
}

/// Error types for WASI code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodegenError {
    /// Unsupported feature in MIR IR
    UnsupportedFeature(String),
    /// Invalid module structure
    InvalidModule(String),
    /// Type mapping error
    TypeMappingError {
        type_hash: String,
        reason: String,
    },
    /// Resource generation error
    ResourceError {
        resource_name: String,
        reason: String,
    },
    /// Capability analysis error
    CapabilityError {
        capability: String,
        reason: String,
    },
    /// Interface generation error
    InterfaceError {
        interface_name: String,
        reason: String,
    },
    /// Binary generation error
    BinaryGenerationError(String),
    /// WIT generation error
    WitGenerationError(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnsupportedFeature(feature) => {
                write!(f, "Unsupported feature: {}", feature)
            }
            CodegenError::InvalidModule(reason) => {
                write!(f, "Invalid module: {}", reason)
            }
            CodegenError::TypeMappingError { type_hash, reason } => {
                write!(f, "Type mapping error for {}: {}", type_hash, reason)
            }
            CodegenError::ResourceError { resource_name, reason } => {
                write!(f, "Resource error for {}: {}", resource_name, reason)
            }
            CodegenError::CapabilityError { capability, reason } => {
                write!(f, "Capability error for {}: {}", capability, reason)
            }
            CodegenError::InterfaceError { interface_name, reason } => {
                write!(f, "Interface error for {}: {}", interface_name, reason)
            }
            CodegenError::BinaryGenerationError(reason) => {
                write!(f, "Binary generation error: {}", reason)
            }
            CodegenError::WitGenerationError(reason) => {
                write!(f, "WIT generation error: {}", reason)
            }
        }
    }
}

impl std::error::Error for CodegenError {}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_ast::{Module, NodeId};
    
    #[test]
    fn test_wasi_code_generator_creation() {
        let generator = WasiCodeGenerator::new();
        assert!(!generator.type_mappings.is_empty());
    }
    
    #[test]
    fn test_component_generation() {
        let generator = WasiCodeGenerator::new();
        let module = Module::new(NodeId::new(1), "test_module".to_string());
        
        let result = generator.generate(&module);
        assert!(result.is_ok());
        
        let component = result.unwrap();
        assert_eq!(component.metadata.name, "test_module");
        assert!(!component.interfaces.is_empty());
    }
    
    #[test]
    fn test_wit_generation() {
        let generator = WasiCodeGenerator::new();
        let module = Module::new(NodeId::new(1), "test_module".to_string());
        
        let component = generator.generate(&module).unwrap();
        let wit = generator.generate_wit(&component);
        
        assert!(wit.contains("package test_module:0.1.0"));
        assert!(wit.contains("world"));
    }
    
    #[test]
    fn test_type_mapping() {
        let generator = WasiCodeGenerator::new();
        
        let bool_hash = TypeHash::new(ContentHash::new(b"bool"));
        let mapped_type = generator.map_type_hash_to_wasi_type(&bool_hash);
        
        assert!(matches!(mapped_type, WasiType::Bool));
    }
}