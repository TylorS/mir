//! WASI component model support
//!
//! This module implements WASI component model compilation, interface type generation,
//! resource management, and capability-based security as specified in Requirement 7.

use mir_types::{ContentHash, Value, ResourceType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WASI component representation with full component model support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiComponent {
    /// Component metadata
    pub metadata: ComponentMetadata,
    /// Component interfaces (imports and exports)
    pub interfaces: Vec<ComponentInterface>,
    /// World definition for this component
    pub world: ComponentWorld,
    /// Resource types managed by this component
    pub resources: Vec<WasiResource>,
    /// Capability declarations
    pub capabilities: ComponentCapabilities,
    /// Component exports
    pub exports: Vec<ComponentExport>,
    /// Component imports
    pub imports: Vec<ComponentImport>,
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub content_hash: ContentHash,
}

/// Component interface definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInterface {
    pub name: String,
    pub namespace: Option<String>,
    pub functions: Vec<InterfaceFunction>,
    pub types: Vec<InterfaceType>,
    pub resources: Vec<InterfaceResource>,
}

/// Function in a component interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceFunction {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<WasiType>,
    pub is_async: bool,
    pub error_type: Option<WasiType>,
    pub documentation: Option<String>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub param_type: WasiType,
    pub is_optional: bool,
    pub default_value: Option<Value>,
}

/// Interface type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceType {
    pub name: String,
    pub definition: WasiTypeDefinition,
    pub documentation: Option<String>,
}

/// Interface resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceResource {
    pub name: String,
    pub methods: Vec<ResourceMethod>,
    pub static_methods: Vec<ResourceMethod>,
    pub constructor: Option<ResourceConstructor>,
    pub destructor: Option<ResourceDestructor>,
}

/// Resource method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMethod {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<WasiType>,
    pub is_static: bool,
    pub mutates_resource: bool,
}

/// Resource constructor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstructor {
    pub parameters: Vec<FunctionParameter>,
    pub can_fail: bool,
    pub error_type: Option<WasiType>,
}

/// Resource destructor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDestructor {
    pub can_fail: bool,
    pub error_type: Option<WasiType>,
}

/// WASI type system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasiType {
    // Primitive types
    Bool,
    U8, U16, U32, U64,
    S8, S16, S32, S64,
    F32, F64,
    Char,
    String,
    
    // Composite types
    List(Box<WasiType>),
    Record(Vec<RecordField>),
    Variant(Vec<VariantCase>),
    Enum(Vec<String>),
    Union(Vec<WasiType>),
    Option(Box<WasiType>),
    Result { ok: Option<Box<WasiType>>, err: Option<Box<WasiType>> },
    Tuple(Vec<WasiType>),
    
    // Resource types
    Resource(String),
    Borrow(String),
    Own(String),
    
    // Function types
    Function {
        parameters: Vec<WasiType>,
        return_type: Option<Box<WasiType>>,
    },
}

/// Record field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordField {
    pub name: String,
    pub field_type: WasiType,
    pub documentation: Option<String>,
}

/// Variant case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantCase {
    pub name: String,
    pub payload: Option<WasiType>,
    pub documentation: Option<String>,
}

/// WASI type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasiTypeDefinition {
    Alias(WasiType),
    Record(Vec<RecordField>),
    Variant(Vec<VariantCase>),
    Enum(Vec<String>),
    Union(Vec<WasiType>),
    Resource {
        methods: Vec<ResourceMethod>,
        static_methods: Vec<ResourceMethod>,
    },
}

/// Component world definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentWorld {
    pub name: String,
    pub imports: Vec<WorldImport>,
    pub exports: Vec<WorldExport>,
    pub includes: Vec<String>,
}

/// World import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldImport {
    pub name: String,
    pub interface: Option<String>,
    pub item_type: WorldItemType,
}

/// World export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldExport {
    pub name: String,
    pub interface: Option<String>,
    pub item_type: WorldItemType,
}

/// World item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldItemType {
    Interface(String),
    Function(InterfaceFunction),
    Type(InterfaceType),
    Resource(InterfaceResource),
}

/// WASI resource with lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiResource {
    pub name: String,
    pub resource_type: ResourceType,
    pub lifecycle: ResourceLifecycle,
    pub capabilities: ResourceCapabilities,
    pub methods: Vec<ResourceMethod>,
}

/// Resource lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLifecycle {
    pub creation_policy: CreationPolicy,
    pub ownership_model: OwnershipModel,
    pub cleanup_strategy: CleanupStrategy,
    pub sharing_policy: SharingPolicy,
}

/// Resource creation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreationPolicy {
    Singleton,
    PerRequest,
    Pooled { max_instances: usize },
    Custom(String),
}

/// Resource ownership model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipModel {
    Owned,
    Borrowed { lifetime: String },
    Shared,
    Weak,
}

/// Resource cleanup strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupStrategy {
    Automatic,
    Manual,
    RAII,
    GarbageCollected,
}

/// Resource sharing policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingPolicy {
    Exclusive,
    ReadOnly,
    ReadWrite,
    Custom(Vec<String>),
}

/// Resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub can_read: bool,
    pub can_write: bool,
    pub can_execute: bool,
    pub can_share: bool,
    pub required_permissions: Vec<String>,
}

/// Component capabilities for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCapabilities {
    /// File system access capabilities
    pub filesystem: FilesystemCapabilities,
    /// Network access capabilities
    pub network: NetworkCapabilities,
    /// Environment access capabilities
    pub environment: EnvironmentCapabilities,
    /// Clock access capabilities
    pub clock: ClockCapabilities,
    /// Random number generation capabilities
    pub random: RandomCapabilities,
    /// Custom capabilities
    pub custom: HashMap<String, CapabilityDescriptor>,
}

/// Filesystem capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCapabilities {
    pub can_read: bool,
    pub can_write: bool,
    pub can_create: bool,
    pub can_delete: bool,
    pub allowed_paths: Vec<String>,
    pub denied_paths: Vec<String>,
}

/// Network capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCapabilities {
    pub can_connect: bool,
    pub can_listen: bool,
    pub allowed_hosts: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub protocols: Vec<NetworkProtocol>,
}

/// Network protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProtocol {
    TCP,
    UDP,
    HTTP,
    HTTPS,
    WebSocket,
    Custom(String),
}

/// Environment capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentCapabilities {
    pub can_read_env: bool,
    pub can_write_env: bool,
    pub allowed_variables: Vec<String>,
    pub denied_variables: Vec<String>,
}

/// Clock capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockCapabilities {
    pub can_read_monotonic: bool,
    pub can_read_wall_clock: bool,
    pub can_sleep: bool,
    pub precision: ClockPrecision,
}

/// Clock precision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClockPrecision {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
}

/// Random capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomCapabilities {
    pub can_generate: bool,
    pub entropy_sources: Vec<EntropySource>,
    pub max_bytes_per_request: Option<usize>,
}

/// Entropy source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropySource {
    SystemRandom,
    Hardware,
    Deterministic,
    Custom(String),
}

/// Custom capability descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub parameters: HashMap<String, Value>,
}

/// Component export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentExport {
    pub name: String,
    pub export_type: ComponentExportType,
    pub visibility: ExportVisibility,
}

/// Component export type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentExportType {
    Function(InterfaceFunction),
    Interface(ComponentInterface),
    Type(InterfaceType),
    Resource(InterfaceResource),
    Instance(String),
}

/// Export visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportVisibility {
    Public,
    Protected,
    Private,
}

/// Component import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentImport {
    pub name: String,
    pub import_type: ComponentImportType,
    pub required: bool,
    pub version_constraint: Option<String>,
}

/// Component import type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentImportType {
    Function(InterfaceFunction),
    Interface(String),
    Type(String),
    Resource(String),
    Instance(String),
}

impl Default for WasiComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiComponent {
    /// Create a new WASI component
    pub fn new() -> Self {
        WasiComponent {
            metadata: ComponentMetadata {
                name: "unnamed".to_string(),
                version: "0.1.0".to_string(),
                description: None,
                authors: Vec::new(),
                license: None,
                content_hash: ContentHash::new(b"empty"),
            },
            interfaces: Vec::new(),
            world: ComponentWorld {
                name: "default".to_string(),
                imports: Vec::new(),
                exports: Vec::new(),
                includes: Vec::new(),
            },
            resources: Vec::new(),
            capabilities: ComponentCapabilities::default(),
            exports: Vec::new(),
            imports: Vec::new(),
        }
    }
    
    /// Add an interface to the component
    pub fn add_interface(&mut self, interface: ComponentInterface) {
        self.interfaces.push(interface);
    }
    
    /// Add a resource to the component
    pub fn add_resource(&mut self, resource: WasiResource) {
        self.resources.push(resource);
    }
    
    /// Set component capabilities
    pub fn set_capabilities(&mut self, capabilities: ComponentCapabilities) {
        self.capabilities = capabilities;
    }
    
    /// Add an export to the component
    pub fn add_export(&mut self, export: ComponentExport) {
        self.exports.push(export);
    }
    
    /// Add an import to the component
    pub fn add_import(&mut self, import: ComponentImport) {
        self.imports.push(import);
    }
    
    /// Generate WIT (WebAssembly Interface Types) definition
    pub fn generate_wit(&self) -> String {
        let mut wit = String::new();
        
        // Package declaration
        wit.push_str(&format!("package {}:{}\n\n", 
            self.metadata.name, self.metadata.version));
        
        // World definition
        wit.push_str(&format!("world {} {{\n", self.world.name));
        
        // Imports
        for import in &self.world.imports {
            wit.push_str(&format!("  import {}: {}\n", 
                import.name, self.format_world_item(&import.item_type)));
        }
        
        // Exports
        for export in &self.world.exports {
            wit.push_str(&format!("  export {}: {}\n", 
                export.name, self.format_world_item(&export.item_type)));
        }
        
        wit.push_str("}\n\n");
        
        // Interface definitions
        for interface in &self.interfaces {
            wit.push_str(&self.generate_interface_wit(interface));
            wit.push('\n');
        }
        
        wit
    }
    
    fn format_world_item(&self, item: &WorldItemType) -> String {
        match item {
            WorldItemType::Interface(name) => name.clone(),
            WorldItemType::Function(func) => {
                format!("func({}){}", 
                    func.parameters.iter()
                        .map(|p| format!("{}: {}", p.name, self.format_wasi_type(&p.param_type)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    func.return_type.as_ref()
                        .map(|t| format!(" -> {}", self.format_wasi_type(t)))
                        .unwrap_or_default()
                )
            },
            WorldItemType::Type(type_def) => {
                format!("type {}", type_def.name)
            },
            WorldItemType::Resource(resource) => {
                format!("resource {}", resource.name)
            },
        }
    }
    
    fn generate_interface_wit(&self, interface: &ComponentInterface) -> String {
        let mut wit = String::new();
        
        let interface_name = if let Some(namespace) = &interface.namespace {
            format!("{}:{}", namespace, interface.name)
        } else {
            interface.name.clone()
        };
        
        wit.push_str(&format!("interface {} {{\n", interface_name));
        
        // Types
        for type_def in &interface.types {
            wit.push_str(&format!("  type {} = {}\n", 
                type_def.name, self.format_type_definition(&type_def.definition)));
        }
        
        // Resources
        for resource in &interface.resources {
            wit.push_str(&format!("  resource {} {{\n", resource.name));
            
            if let Some(constructor) = &resource.constructor {
                wit.push_str("    constructor(");
                wit.push_str(&constructor.parameters.iter()
                    .map(|p| format!("{}: {}", p.name, self.format_wasi_type(&p.param_type)))
                    .collect::<Vec<_>>()
                    .join(", "));
                wit.push_str(")\n");
            }
            
            for method in &resource.methods {
                if method.is_static {
                    wit.push_str("    static ");
                }
                wit.push_str(&format!("    {}: func(", method.name));
                wit.push_str(&method.parameters.iter()
                    .map(|p| format!("{}: {}", p.name, self.format_wasi_type(&p.param_type)))
                    .collect::<Vec<_>>()
                    .join(", "));
                wit.push_str(")");
                if let Some(return_type) = &method.return_type {
                    wit.push_str(&format!(" -> {}", self.format_wasi_type(return_type)));
                }
                wit.push_str("\n");
            }
            
            wit.push_str("  }\n");
        }
        
        // Functions
        for function in &interface.functions {
            wit.push_str(&format!("  {}: func(", function.name));
            wit.push_str(&function.parameters.iter()
                .map(|p| format!("{}: {}", p.name, self.format_wasi_type(&p.param_type)))
                .collect::<Vec<_>>()
                .join(", "));
            wit.push_str(")");
            if let Some(return_type) = &function.return_type {
                wit.push_str(&format!(" -> {}", self.format_wasi_type(return_type)));
            }
            wit.push_str("\n");
        }
        
        wit.push_str("}\n");
        wit
    }
    
    fn format_type_definition(&self, def: &WasiTypeDefinition) -> String {
        match def {
            WasiTypeDefinition::Alias(wasi_type) => self.format_wasi_type(wasi_type),
            WasiTypeDefinition::Record(fields) => {
                format!("record {{ {} }}", 
                    fields.iter()
                        .map(|f| format!("{}: {}", f.name, self.format_wasi_type(&f.field_type)))
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiTypeDefinition::Variant(cases) => {
                format!("variant {{ {} }}", 
                    cases.iter()
                        .map(|c| match &c.payload {
                            Some(payload) => format!("{}({})", c.name, self.format_wasi_type(payload)),
                            None => c.name.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiTypeDefinition::Enum(variants) => {
                format!("enum {{ {} }}", variants.join(", "))
            },
            WasiTypeDefinition::Union(types) => {
                format!("union {{ {} }}", 
                    types.iter()
                        .map(|t| self.format_wasi_type(t))
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiTypeDefinition::Resource { methods, static_methods: _ } => {
                format!("resource {{ {} }}", 
                    methods.iter()
                        .map(|m| format!("{}: func", m.name))
                        .collect::<Vec<_>>()
                        .join(", "))
            },
        }
    }
    
    fn format_wasi_type(&self, wasi_type: &WasiType) -> String {
        match wasi_type {
            WasiType::Bool => "bool".to_string(),
            WasiType::U8 => "u8".to_string(),
            WasiType::U16 => "u16".to_string(),
            WasiType::U32 => "u32".to_string(),
            WasiType::U64 => "u64".to_string(),
            WasiType::S8 => "s8".to_string(),
            WasiType::S16 => "s16".to_string(),
            WasiType::S32 => "s32".to_string(),
            WasiType::S64 => "s64".to_string(),
            WasiType::F32 => "f32".to_string(),
            WasiType::F64 => "f64".to_string(),
            WasiType::Char => "char".to_string(),
            WasiType::String => "string".to_string(),
            WasiType::List(inner) => format!("list<{}>", self.format_wasi_type(inner)),
            WasiType::Record(fields) => {
                format!("record {{ {} }}", 
                    fields.iter()
                        .map(|f| format!("{}: {}", f.name, self.format_wasi_type(&f.field_type)))
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiType::Variant(cases) => {
                format!("variant {{ {} }}", 
                    cases.iter()
                        .map(|c| match &c.payload {
                            Some(payload) => format!("{}({})", c.name, self.format_wasi_type(payload)),
                            None => c.name.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiType::Enum(variants) => {
                format!("enum {{ {} }}", variants.join(", "))
            },
            WasiType::Union(types) => {
                format!("union {{ {} }}", 
                    types.iter()
                        .map(|t| self.format_wasi_type(t))
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiType::Option(inner) => format!("option<{}>", self.format_wasi_type(inner)),
            WasiType::Result { ok, err } => {
                let ok_str = ok.as_ref().map(|t| self.format_wasi_type(t)).unwrap_or_default();
                let err_str = err.as_ref().map(|t| self.format_wasi_type(t)).unwrap_or_default();
                format!("result<{}, {}>", ok_str, err_str)
            },
            WasiType::Tuple(types) => {
                format!("tuple<{}>", 
                    types.iter()
                        .map(|t| self.format_wasi_type(t))
                        .collect::<Vec<_>>()
                        .join(", "))
            },
            WasiType::Resource(name) => format!("resource {}", name),
            WasiType::Borrow(name) => format!("borrow<{}>", name),
            WasiType::Own(name) => format!("own<{}>", name),
            WasiType::Function { parameters, return_type } => {
                let params = parameters.iter()
                    .map(|t| self.format_wasi_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = return_type.as_ref()
                    .map(|t| format!(" -> {}", self.format_wasi_type(t)))
                    .unwrap_or_default();
                format!("func({}){}", params, ret)
            },
        }
    }
}

impl Default for ComponentCapabilities {
    fn default() -> Self {
        ComponentCapabilities {
            filesystem: FilesystemCapabilities {
                can_read: false,
                can_write: false,
                can_create: false,
                can_delete: false,
                allowed_paths: Vec::new(),
                denied_paths: Vec::new(),
            },
            network: NetworkCapabilities {
                can_connect: false,
                can_listen: false,
                allowed_hosts: Vec::new(),
                allowed_ports: Vec::new(),
                protocols: Vec::new(),
            },
            environment: EnvironmentCapabilities {
                can_read_env: false,
                can_write_env: false,
                allowed_variables: Vec::new(),
                denied_variables: Vec::new(),
            },
            clock: ClockCapabilities {
                can_read_monotonic: false,
                can_read_wall_clock: false,
                can_sleep: false,
                precision: ClockPrecision::Millisecond,
            },
            random: RandomCapabilities {
                can_generate: false,
                entropy_sources: Vec::new(),
                max_bytes_per_request: None,
            },
            custom: HashMap::new(),
        }
    }
}