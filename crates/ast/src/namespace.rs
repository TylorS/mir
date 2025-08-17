//! Module namespace system with separate value and type namespaces
//! 
//! This module implements Requirement 4: separate namespaces for values and types
//! within each module scope, enabling the same name to be used for related types
//! and values without conflicts.

use crate::NodeId;
use mir_types::{ContentHash, TypeHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Binding for a value in the value namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueBinding {
    pub name: String,
    pub value_type: TypeHash,
    pub content_hash: ContentHash,
    pub is_mutable: bool,
    pub visibility: Visibility,
    pub source_location: Option<SourceLocation>,
}

/// Binding for a type in the type namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeBinding {
    pub name: String,
    pub type_hash: TypeHash,
    pub content_hash: ContentHash,
    pub visibility: Visibility,
    pub source_location: Option<SourceLocation>,
}

/// Visibility of a binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Public,
    Internal, // Visible within the same crate/package
}

/// Source location information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Module namespace with separate value and type namespaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNamespace {
    pub module_id: NodeId,
    pub module_name: String,
    /// Value namespace: maps names to value bindings
    pub value_namespace: HashMap<String, ValueBinding>,
    /// Type namespace: maps names to type bindings
    pub type_namespace: HashMap<String, TypeBinding>,
    /// Parent scope for nested modules (optional)
    pub parent_scope: Option<Box<ModuleNamespace>>,
    /// Content hash of this namespace for versioning
    pub namespace_hash: ContentHash,
}

/// Context for name resolution
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    pub current_module: NodeId,
    pub lookup_chain: Vec<NodeId>,
    pub allow_private: bool,
    pub prefer_local: bool,
}

/// Result of name resolution
#[derive(Debug, Clone)]
pub enum ResolutionResult<T> {
    Found(T),
    Ambiguous(Vec<T>),
    NotFound,
}

/// Error types for namespace operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamespaceError {
    NameAlreadyExists { name: String, namespace: String },
    NameNotFound { name: String, namespace: String },
    AmbiguousName { name: String, candidates: Vec<String> },
    VisibilityViolation { name: String, required_visibility: Visibility },
    CircularDependency { modules: Vec<NodeId> },
    InvalidContext { reason: String },
}

impl std::fmt::Display for NamespaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceError::NameAlreadyExists { name, namespace } => {
                write!(f, "Name '{name}' already exists in {namespace} namespace")
            }
            NamespaceError::NameNotFound { name, namespace } => {
                write!(f, "Name '{name}' not found in {namespace} namespace")
            }
            NamespaceError::AmbiguousName { name, candidates } => {
                write!(f, "Ambiguous name '{name}', candidates: {candidates:?}")
            }
            NamespaceError::VisibilityViolation { name, required_visibility } => {
                write!(f, "Cannot access '{name}', requires {required_visibility:?} visibility")
            }
            NamespaceError::CircularDependency { modules } => {
                write!(f, "Circular dependency detected in modules: {modules:?}")
            }
            NamespaceError::InvalidContext { reason } => {
                write!(f, "Invalid resolution context: {reason}")
            }
        }
    }
}

impl std::error::Error for NamespaceError {}

impl ModuleNamespace {
    /// Create a new module namespace
    pub fn new(module_id: NodeId, module_name: String) -> Self {
        let namespace_data = format!("{module_name}:{module_id:?}");
        let namespace_hash = ContentHash::new(namespace_data.as_bytes());
        
        ModuleNamespace {
            module_id,
            module_name,
            value_namespace: HashMap::new(),
            type_namespace: HashMap::new(),
            parent_scope: None,
            namespace_hash,
        }
    }
    
    /// Create a nested namespace with a parent scope
    pub fn new_nested(
        module_id: NodeId,
        module_name: String,
        parent: Box<ModuleNamespace>,
    ) -> Self {
        let mut namespace = Self::new(module_id, module_name);
        namespace.parent_scope = Some(parent);
        namespace
    }
    
    /// Add a value binding to the value namespace
    pub fn add_value_binding(&mut self, binding: ValueBinding) -> Result<(), NamespaceError> {
        if self.value_namespace.contains_key(&binding.name) {
            return Err(NamespaceError::NameAlreadyExists {
                name: binding.name.clone(),
                namespace: "value".to_string(),
            });
        }
        
        self.value_namespace.insert(binding.name.clone(), binding);
        self.update_namespace_hash();
        Ok(())
    }
    
    /// Add a type binding to the type namespace
    pub fn add_type_binding(&mut self, binding: TypeBinding) -> Result<(), NamespaceError> {
        if self.type_namespace.contains_key(&binding.name) {
            return Err(NamespaceError::NameAlreadyExists {
                name: binding.name.clone(),
                namespace: "type".to_string(),
            });
        }
        
        self.type_namespace.insert(binding.name.clone(), binding);
        self.update_namespace_hash();
        Ok(())
    }
    
    /// Update the namespace hash when bindings change
    fn update_namespace_hash(&mut self) {
        let mut hasher_data = Vec::new();
        hasher_data.extend_from_slice(self.module_name.as_bytes());
        hasher_data.extend_from_slice(format!("{:?}", self.module_id).as_bytes());
        
        // Include value bindings in hash
        let mut value_names: Vec<_> = self.value_namespace.keys().collect();
        value_names.sort();
        for name in value_names {
            hasher_data.extend_from_slice(name.as_bytes());
            if let Some(binding) = self.value_namespace.get(name) {
                hasher_data.extend_from_slice(binding.content_hash.as_bytes());
            }
        }
        
        // Include type bindings in hash
        let mut type_names: Vec<_> = self.type_namespace.keys().collect();
        type_names.sort();
        for name in type_names {
            hasher_data.extend_from_slice(name.as_bytes());
            if let Some(binding) = self.type_namespace.get(name) {
                hasher_data.extend_from_slice(binding.content_hash.as_bytes());
            }
        }
        
        self.namespace_hash = ContentHash::new(&hasher_data);
    }
    
    /// Get the content hash of this namespace
    pub fn content_hash(&self) -> ContentHash {
        self.namespace_hash
    }
}

/// Trait for namespace resolution operations
pub trait NamespaceResolver {
    /// Resolve a value name in the current context
    fn resolve_value(&self, name: &str, context: &ResolutionContext) -> ResolutionResult<ValueBinding>;
    
    /// Resolve a type name in the current context
    fn resolve_type(&self, name: &str, context: &ResolutionContext) -> ResolutionResult<TypeBinding>;
    
    /// Import a value binding from another module
    fn import_value(&mut self, name: String, binding: ValueBinding) -> Result<(), NamespaceError>;
    
    /// Import a type binding from another module
    fn import_type(&mut self, name: String, binding: TypeBinding) -> Result<(), NamespaceError>;
    
    /// Export a value binding (make it publicly available)
    fn export_value(&self, name: &str) -> Option<ValueBinding>;
    
    /// Export a type binding (make it publicly available)
    fn export_type(&self, name: &str) -> Option<TypeBinding>;
    
    /// Check if a name exists in the value namespace
    fn has_value(&self, name: &str) -> bool;
    
    /// Check if a name exists in the type namespace
    fn has_type(&self, name: &str) -> bool;
    
    /// Get all value names in this namespace
    fn get_value_names(&self) -> Vec<String>;
    
    /// Get all type names in this namespace
    fn get_type_names(&self) -> Vec<String>;
}

impl NamespaceResolver for ModuleNamespace {
    fn resolve_value(&self, name: &str, context: &ResolutionContext) -> ResolutionResult<ValueBinding> {
        // Check local value namespace first
        if let Some(binding) = self.value_namespace.get(name) {
            // Check visibility
            if self.can_access_binding(binding.visibility, context) {
                return ResolutionResult::Found(binding.clone());
            }
        }
        
        // Check parent scope if available and not preferring local only
        if !context.prefer_local {
            if let Some(parent) = &self.parent_scope {
                match parent.resolve_value(name, context) {
                    ResolutionResult::Found(binding) => return ResolutionResult::Found(binding),
                    ResolutionResult::Ambiguous(bindings) => return ResolutionResult::Ambiguous(bindings),
                    ResolutionResult::NotFound => {}
                }
            }
        }
        
        ResolutionResult::NotFound
    }
    
    fn resolve_type(&self, name: &str, context: &ResolutionContext) -> ResolutionResult<TypeBinding> {
        // Check local type namespace first
        if let Some(binding) = self.type_namespace.get(name) {
            // Check visibility
            if self.can_access_binding(binding.visibility, context) {
                return ResolutionResult::Found(binding.clone());
            }
        }
        
        // Check parent scope if available and not preferring local only
        if !context.prefer_local {
            if let Some(parent) = &self.parent_scope {
                match parent.resolve_type(name, context) {
                    ResolutionResult::Found(binding) => return ResolutionResult::Found(binding),
                    ResolutionResult::Ambiguous(bindings) => return ResolutionResult::Ambiguous(bindings),
                    ResolutionResult::NotFound => {}
                }
            }
        }
        
        ResolutionResult::NotFound
    }
    
    fn import_value(&mut self, name: String, binding: ValueBinding) -> Result<(), NamespaceError> {
        // Create a new binding with the imported name
        let imported_binding = ValueBinding {
            name: name.clone(),
            ..binding
        };
        
        self.add_value_binding(imported_binding)
    }
    
    fn import_type(&mut self, name: String, binding: TypeBinding) -> Result<(), NamespaceError> {
        // Create a new binding with the imported name
        let imported_binding = TypeBinding {
            name: name.clone(),
            ..binding
        };
        
        self.add_type_binding(imported_binding)
    }
    
    fn export_value(&self, name: &str) -> Option<ValueBinding> {
        self.value_namespace.get(name)
            .filter(|binding| binding.visibility == Visibility::Public)
            .cloned()
    }
    
    fn export_type(&self, name: &str) -> Option<TypeBinding> {
        self.type_namespace.get(name)
            .filter(|binding| binding.visibility == Visibility::Public)
            .cloned()
    }
    
    fn has_value(&self, name: &str) -> bool {
        self.value_namespace.contains_key(name)
    }
    
    fn has_type(&self, name: &str) -> bool {
        self.type_namespace.contains_key(name)
    }
    
    fn get_value_names(&self) -> Vec<String> {
        self.value_namespace.keys().cloned().collect()
    }
    
    fn get_type_names(&self) -> Vec<String> {
        self.type_namespace.keys().cloned().collect()
    }
}

impl ModuleNamespace {
    /// Check if a binding can be accessed in the given context
    fn can_access_binding(&self, visibility: Visibility, context: &ResolutionContext) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Private => context.allow_private && context.current_module == self.module_id,
            Visibility::Internal => {
                // For now, treat internal as private
                // In a full implementation, this would check crate/package boundaries
                context.allow_private && context.current_module == self.module_id
            }
        }
    }
    
    /// Get a value binding directly (for internal use)
    pub fn get_value_binding(&self, name: &str) -> Option<&ValueBinding> {
        self.value_namespace.get(name)
    }
    
    /// Get a type binding directly (for internal use)
    pub fn get_type_binding(&self, name: &str) -> Option<&TypeBinding> {
        self.type_namespace.get(name)
    }
    
    /// Remove a value binding
    pub fn remove_value_binding(&mut self, name: &str) -> Option<ValueBinding> {
        let result = self.value_namespace.remove(name);
        if result.is_some() {
            self.update_namespace_hash();
        }
        result
    }
    
    /// Remove a type binding
    pub fn remove_type_binding(&mut self, name: &str) -> Option<TypeBinding> {
        let result = self.type_namespace.remove(name);
        if result.is_some() {
            self.update_namespace_hash();
        }
        result
    }
    
    /// Update an existing value binding
    pub fn update_value_binding(&mut self, binding: ValueBinding) -> Result<(), NamespaceError> {
        if !self.value_namespace.contains_key(&binding.name) {
            return Err(NamespaceError::NameNotFound {
                name: binding.name.clone(),
                namespace: "value".to_string(),
            });
        }
        
        self.value_namespace.insert(binding.name.clone(), binding);
        self.update_namespace_hash();
        Ok(())
    }
    
    /// Update an existing type binding
    pub fn update_type_binding(&mut self, binding: TypeBinding) -> Result<(), NamespaceError> {
        if !self.type_namespace.contains_key(&binding.name) {
            return Err(NamespaceError::NameNotFound {
                name: binding.name.clone(),
                namespace: "type".to_string(),
            });
        }
        
        self.type_namespace.insert(binding.name.clone(), binding);
        self.update_namespace_hash();
        Ok(())
    }
}

impl ResolutionContext {
    /// Create a new resolution context
    pub fn new(current_module: NodeId) -> Self {
        ResolutionContext {
            current_module,
            lookup_chain: vec![current_module],
            allow_private: true,
            prefer_local: false,
        }
    }
    
    /// Create a context for external module access
    pub fn external(current_module: NodeId) -> Self {
        ResolutionContext {
            current_module,
            lookup_chain: vec![current_module],
            allow_private: false,
            prefer_local: false,
        }
    }
    
    /// Add a module to the lookup chain (for circular dependency detection)
    pub fn with_module(mut self, module_id: NodeId) -> Result<Self, NamespaceError> {
        if self.lookup_chain.contains(&module_id) {
            return Err(NamespaceError::CircularDependency {
                modules: self.lookup_chain,
            });
        }
        
        self.lookup_chain.push(module_id);
        Ok(self)
    }
    
    /// Set whether to prefer local bindings over parent scope
    pub fn prefer_local(mut self, prefer: bool) -> Self {
        self.prefer_local = prefer;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;
    use mir_types::TypeHash;
    
    fn create_test_value_binding(name: &str) -> ValueBinding {
        ValueBinding {
            name: name.to_string(),
            value_type: TypeHash::new(ContentHash::new(b"test_type")),
            content_hash: ContentHash::new(name.as_bytes()),
            is_mutable: false,
            visibility: Visibility::Public,
            source_location: None,
        }
    }
    
    fn create_test_type_binding(name: &str) -> TypeBinding {
        TypeBinding {
            name: name.to_string(),
            type_hash: TypeHash::new(ContentHash::new(name.as_bytes())),
            content_hash: ContentHash::new(name.as_bytes()),
            visibility: Visibility::Public,
            source_location: None,
        }
    }
    
    #[test]
    fn test_separate_namespaces() {
        let mut namespace = ModuleNamespace::new(NodeId::new(1), "test_module".to_string());
        
        // Add a value and type with the same name
        let value_binding = create_test_value_binding("Item");
        let type_binding = create_test_type_binding("Item");
        
        assert!(namespace.add_value_binding(value_binding).is_ok());
        assert!(namespace.add_type_binding(type_binding).is_ok());
        
        // Both should exist in their respective namespaces
        assert!(namespace.has_value("Item"));
        assert!(namespace.has_type("Item"));
        
        // Should be able to resolve both
        let context = ResolutionContext::new(NodeId::new(1));
        
        match namespace.resolve_value("Item", &context) {
            ResolutionResult::Found(binding) => {
                assert_eq!(binding.name, "Item");
            }
            _ => panic!("Expected to find value binding"),
        }
        
        match namespace.resolve_type("Item", &context) {
            ResolutionResult::Found(binding) => {
                assert_eq!(binding.name, "Item");
            }
            _ => panic!("Expected to find type binding"),
        }
    }
    
    #[test]
    fn test_name_conflicts_within_namespace() {
        let mut namespace = ModuleNamespace::new(NodeId::new(1), "test_module".to_string());
        
        let binding1 = create_test_value_binding("duplicate");
        let binding2 = create_test_value_binding("duplicate");
        
        // First binding should succeed
        assert!(namespace.add_value_binding(binding1).is_ok());
        
        // Second binding with same name should fail
        match namespace.add_value_binding(binding2) {
            Err(NamespaceError::NameAlreadyExists { name, namespace: ns }) => {
                assert_eq!(name, "duplicate");
                assert_eq!(ns, "value");
            }
            _ => panic!("Expected NameAlreadyExists error"),
        }
    }
    
    #[test]
    fn test_visibility_access_control() {
        let mut namespace = ModuleNamespace::new(NodeId::new(1), "test_module".to_string());
        
        let mut private_binding = create_test_value_binding("private_item");
        private_binding.visibility = Visibility::Private;
        
        namespace.add_value_binding(private_binding).unwrap();
        
        // Should be accessible from same module
        let internal_context = ResolutionContext::new(NodeId::new(1));
        match namespace.resolve_value("private_item", &internal_context) {
            ResolutionResult::Found(_) => {}
            _ => panic!("Expected to find private binding from same module"),
        }
        
        // Should not be accessible from external module
        let external_context = ResolutionContext::external(NodeId::new(2));
        match namespace.resolve_value("private_item", &external_context) {
            ResolutionResult::NotFound => {}
            _ => panic!("Expected not to find private binding from external module"),
        }
    }
    
    #[test]
    fn test_parent_scope_resolution() {
        let mut parent = ModuleNamespace::new(NodeId::new(1), "parent".to_string());
        let parent_binding = create_test_value_binding("parent_item");
        parent.add_value_binding(parent_binding).unwrap();
        
        let child = ModuleNamespace::new_nested(NodeId::new(2), "child".to_string(), Box::new(parent));
        
        // Should be able to resolve parent binding from child
        let context = ResolutionContext::new(NodeId::new(2));
        match child.resolve_value("parent_item", &context) {
            ResolutionResult::Found(binding) => {
                assert_eq!(binding.name, "parent_item");
            }
            _ => panic!("Expected to find parent binding from child scope"),
        }
    }
    
    #[test]
    fn test_import_export_operations() {
        let mut namespace = ModuleNamespace::new(NodeId::new(1), "test_module".to_string());
        
        let binding = create_test_value_binding("exported_item");
        namespace.add_value_binding(binding).unwrap();
        
        // Should be able to export public binding
        let exported = namespace.export_value("exported_item");
        assert!(exported.is_some());
        
        // Should be able to import with different name
        let mut other_namespace = ModuleNamespace::new(NodeId::new(2), "other_module".to_string());
        let imported_binding = exported.unwrap();
        
        assert!(other_namespace.import_value("imported_item".to_string(), imported_binding).is_ok());
        assert!(other_namespace.has_value("imported_item"));
    }
    
    #[test]
    fn test_namespace_hash_updates() {
        let mut namespace = ModuleNamespace::new(NodeId::new(1), "test_module".to_string());
        let initial_hash = namespace.content_hash();
        
        // Adding a binding should change the hash
        let binding = create_test_value_binding("new_item");
        namespace.add_value_binding(binding).unwrap();
        let new_hash = namespace.content_hash();
        
        assert_ne!(initial_hash, new_hash);
    }
}