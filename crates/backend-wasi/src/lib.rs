//! MIR Backend WASI - WebAssembly System Interface support
//! 
//! This crate provides comprehensive WASI component model compilation, including:
//! - WASI component model compilation from MIR IR
//! - WASI interface type generation with full type system support
//! - WASI resource management with lifecycle control
//! - WASI capability-based security implementation
//! - NAPI bindings for JavaScript/TypeScript integration
//!
//! This implementation satisfies Requirement 7: corresponding AST representation
//! and instruction set that maps cleanly to WASM operations, specifically
//! targeting the WASI component model for capability-based security and
//! resource management.

pub mod codegen;
pub mod component;
pub mod napi;

// Re-export main types and functionality
pub use codegen::{
    WasiCodeGenerator, CodegenError, CompilationContext,
    ResourceConfig, SecurityPolicy, ResourceLimits,
};
pub use component::{
    WasiComponent, ComponentInterface, InterfaceFunction, FunctionParameter,
    WasiType, WasiResource, ComponentCapabilities, ComponentExport,
    ComponentExportType, ExportVisibility, ComponentMetadata, ComponentWorld,
    WorldExport, WorldImport, WorldItemType, InterfaceType, WasiTypeDefinition,
    RecordField, VariantCase, ResourceLifecycle, CreationPolicy,
    OwnershipModel, CleanupStrategy, SharingPolicy, ResourceCapabilities,
    FilesystemCapabilities, NetworkCapabilities, EnvironmentCapabilities,
    ClockCapabilities, RandomCapabilities, CapabilityDescriptor,
    ComponentImport, ComponentImportType, ResourceMethod, ResourceConstructor,
    ResourceDestructor, NetworkProtocol, ClockPrecision, EntropySource,
};
pub use napi::{
    WasiNapiBindings, WasiRuntimeConfig, JsWasiComponent,
    CompilationResult, ValidationResult,
};

/// WASI backend version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// WASI component model version supported
pub const WASI_COMPONENT_MODEL_VERSION: &str = "0.2.0";

/// Default WASI world name
pub const DEFAULT_WORLD_NAME: &str = "mir-component";

#[cfg(test)]
mod integration_tests {
    use super::*;
    use mir_ast::{Module, NodeId, ExportDeclaration};
    use mir_types::ContentHash;
    
    #[test]
    fn test_full_compilation_pipeline() {
        // Create a test module
        let mut module = Module::new(NodeId::new(1), "integration_test".to_string());
        
        // Add a simple export
        module.add_export(ExportDeclaration::value(
            "test_function".to_string(),
            ContentHash::new(b"test_function_hash"),
        ));
        
        // Create WASI code generator
        let generator = WasiCodeGenerator::new();
        
        // Generate WASI component
        let component = generator.generate(&module).expect("Component generation should succeed");
        
        // Verify component structure
        assert_eq!(component.metadata.name, "integration_test");
        assert!(!component.interfaces.is_empty());
        assert!(!component.exports.is_empty());
        
        // Generate WIT definition
        let wit = generator.generate_wit(&component);
        assert!(wit.contains("package integration_test:0.1.0"));
        assert!(wit.contains("world"));
        
        // Generate binary
        let binary = generator.generate_binary(&component).expect("Binary generation should succeed");
        assert!(binary.len() >= 8); // At least WASM magic + version
        assert_eq!(&binary[0..4], &[0x00, 0x61, 0x73, 0x6d]); // WASM magic
    }
    
    #[test]
    fn test_napi_integration() {
        // Create NAPI bindings
        let bindings = WasiNapiBindings::new();
        
        // Create a test module
        let module = Module::new(NodeId::new(1), "napi_test".to_string());
        let module_json = serde_json::to_string(&module).expect("Module serialization should succeed");
        
        // Compile through NAPI
        let result = bindings.compile_module(&module_json);
        assert!(result.success, "Compilation should succeed: {:?}", result.errors);
        assert!(result.component.is_some());
        
        let component = result.component.unwrap();
        assert_eq!(component.metadata.name, "napi_test");
        
        // Validate the component
        let component_json = serde_json::to_string(&component).expect("Component serialization should succeed");
        let validation = bindings.validate_component(&component_json);
        assert!(validation.is_valid, "Component should be valid: {:?}", validation.errors);
        
        // Generate TypeScript definitions
        let ts_def = bindings.generate_typescript_definitions(&component_json)
            .expect("TypeScript generation should succeed");
        assert!(ts_def.contains("export interface WasiComponent"));
    }
    
    #[test]
    fn test_capability_based_security() {
        let mut capabilities = ComponentCapabilities::default();
        
        // Enable filesystem read capability
        capabilities.filesystem.can_read = true;
        capabilities.filesystem.allowed_paths = vec!["/tmp".to_string()];
        
        // Enable network connect capability
        capabilities.network.can_connect = true;
        capabilities.network.allowed_hosts = vec!["localhost".to_string()];
        capabilities.network.allowed_ports = vec![8080];
        
        let resource_config = ResourceConfig {
            security_policy: SecurityPolicy {
                enforce_capabilities: true,
                allow_unsafe_operations: false,
                require_explicit_permissions: true,
                sandbox_resources: true,
            },
            ..ResourceConfig::default()
        };
        
        let generator = WasiCodeGenerator::with_config(capabilities.clone(), resource_config);
        let module = Module::new(NodeId::new(1), "security_test".to_string());
        
        let component = generator.generate(&module).expect("Component generation should succeed");
        
        // Verify capabilities are properly set
        assert_eq!(component.capabilities.filesystem.can_read, true);
        assert_eq!(component.capabilities.filesystem.allowed_paths, vec!["/tmp"]);
        assert_eq!(component.capabilities.network.can_connect, true);
        assert_eq!(component.capabilities.network.allowed_hosts, vec!["localhost"]);
    }
    
    #[test]
    fn test_resource_lifecycle_management() {
        let resource_config = ResourceConfig {
            default_lifecycle: ResourceLifecycle {
                creation_policy: CreationPolicy::Singleton,
                ownership_model: OwnershipModel::Shared,
                cleanup_strategy: CleanupStrategy::Automatic,
                sharing_policy: SharingPolicy::ReadOnly,
            },
            ..ResourceConfig::default()
        };
        
        let generator = WasiCodeGenerator::with_config(
            ComponentCapabilities::default(),
            resource_config,
        );
        
        let module = Module::new(NodeId::new(1), "resource_test".to_string());
        let component = generator.generate(&module).expect("Component generation should succeed");
        
        // Verify component was generated successfully
        assert_eq!(component.metadata.name, "resource_test");
    }
}