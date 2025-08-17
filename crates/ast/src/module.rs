//! Module definitions with explicit import/export system
//!
//! This module implements Requirement 5: explicit import and export systems
//! for values and types, enabling precise control over module interfaces
//! and dependencies.

use crate::{ASTNode, NodeId, Statement, TypeBinding, ValueBinding};
use mir_types::{ContentHash, TypeHash, Value};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Module representation as first-class value with actor-like capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: NodeId,
    pub name: String,
    pub imports: Vec<ImportDeclaration>,
    pub exports: Vec<ExportDeclaration>,
    pub statements: Vec<Statement>,
    /// Type hash for this module as a first-class value
    pub type_hash: TypeHash,
    /// Actor-like capabilities for message passing
    pub capabilities: ModuleCapabilities,
    /// Message queue for actor-like behavior
    pub message_queue: VecDeque<ModuleMessage>,
}

/// Explicit import declaration for values and types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclaration {
    pub module_path: String,
    pub module_hash: Option<ContentHash>,
    pub imported_items: Vec<ImportItem>,
    pub import_hash: ContentHash,
}

/// Individual import item with compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub is_type: bool,
    pub expected_hash: Option<ContentHash>,
    pub compatibility_version: Option<String>,
}

/// Explicit export declaration making items publicly available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDeclaration {
    pub name: String,
    pub is_type: bool,
    pub exported_hash: ContentHash,
    pub visibility: ExportVisibility,
    pub compatibility_info: CompatibilityInfo,
}

/// Export visibility levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportVisibility {
    Public,    // Available to all modules
    Protected, // Available to modules in same package
    Internal,  // Available to modules in same crate
}

/// Compatibility information for exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub version: String,
    pub breaking_changes: Vec<String>,
    pub deprecated_features: Vec<String>,
    pub migration_hints: Vec<String>,
}

/// Dependency graph for modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<NodeId, ModuleDependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub cycles: Vec<Vec<NodeId>>,
}

/// Node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependencyNode {
    pub module_id: NodeId,
    pub module_name: String,
    pub module_hash: ContentHash,
    pub dependencies: HashSet<NodeId>,
    pub dependents: HashSet<NodeId>,
}

/// Edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub import_items: Vec<String>,
    pub dependency_type: DependencyType,
}

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    Direct,     // Direct import
    Transitive, // Indirect dependency
    Circular,   // Circular dependency (should be avoided)
}

/// Import/Export compatibility validation result
#[derive(Debug, Clone)]
pub enum CompatibilityResult {
    Compatible,
    CompatibleWithWarnings(Vec<String>),
    Incompatible(Vec<String>),
}

/// Error types for import/export operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportExportError {
    ModuleNotFound {
        module_path: String,
    },
    ItemNotFound {
        item_name: String,
        module_path: String,
    },
    ItemNotExported {
        item_name: String,
        module_path: String,
    },
    VisibilityViolation {
        item_name: String,
        required_visibility: ExportVisibility,
    },
    TypeMismatch {
        item_name: String,
        expected: TypeHash,
        found: TypeHash,
    },
    CircularDependency {
        cycle: Vec<NodeId>,
    },
    CompatibilityViolation {
        item_name: String,
        reason: String,
    },
    AliasConflict {
        alias: String,
        existing_item: String,
    },
}

impl std::fmt::Display for ImportExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportExportError::ModuleNotFound { module_path } => {
                write!(f, "Module not found: {module_path}")
            }
            ImportExportError::ItemNotFound {
                item_name,
                module_path,
            } => {
                write!(
                    f,
                    "Item '{item_name}' not found in module '{module_path}'"
                )
            }
            ImportExportError::ItemNotExported {
                item_name,
                module_path,
            } => {
                write!(
                    f,
                    "Item '{item_name}' is not exported from module '{module_path}'"
                )
            }
            ImportExportError::VisibilityViolation {
                item_name,
                required_visibility,
            } => {
                write!(
                    f,
                    "Cannot access '{item_name}', requires {required_visibility:?} visibility"
                )
            }
            ImportExportError::TypeMismatch {
                item_name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Type mismatch for '{item_name}': expected {expected}, found {found}"
                )
            }
            ImportExportError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {cycle:?}")
            }
            ImportExportError::CompatibilityViolation { item_name, reason } => {
                write!(f, "Compatibility violation for '{item_name}': {reason}")
            }
            ImportExportError::AliasConflict {
                alias,
                existing_item,
            } => {
                write!(
                    f,
                    "Alias '{alias}' conflicts with existing item '{existing_item}'"
                )
            }
        }
    }
}

impl std::error::Error for ImportExportError {}

/// Trait for import/export operations
pub trait ImportExportManager {
    /// Validate that imported items exist and are compatible
    fn validate_imports(&self, module: &Module) -> Result<(), Vec<ImportExportError>>;

    /// Validate that exported items are properly declared
    fn validate_exports(&self, module: &Module) -> Result<(), Vec<ImportExportError>>;

    /// Check compatibility between import requirements and export declarations
    fn check_import_export_compatibility(
        &self,
        import: &ImportItem,
        export: &ExportDeclaration,
    ) -> CompatibilityResult;

    /// Build dependency graph from import/export information
    fn build_dependency_graph(
        &self,
        modules: &[Module],
    ) -> Result<DependencyGraph, ImportExportError>;

    /// Detect circular dependencies
    fn detect_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<NodeId>>;

    /// Resolve import to actual binding
    fn resolve_import(
        &self,
        import: &ImportItem,
        from_module: &Module,
    ) -> Result<ImportResolution, ImportExportError>;
}

/// Result of import resolution
#[derive(Debug, Clone)]
pub enum ImportResolution {
    ValueBinding(ValueBinding),
    TypeBinding(TypeBinding),
}

impl ASTNode for Module {
    fn node_id(&self) -> NodeId {
        self.id
    }

    fn content_hash(&self) -> ContentHash {
        let mut hasher_data = Vec::new();
        hasher_data.extend_from_slice(self.name.as_bytes());
        hasher_data.extend_from_slice(format!("{:?}", self.id).as_bytes());

        // Include imports in hash
        for import in &self.imports {
            hasher_data.extend_from_slice(import.module_path.as_bytes());
            hasher_data.extend_from_slice(import.import_hash.as_bytes());
        }

        // Include exports in hash
        for export in &self.exports {
            hasher_data.extend_from_slice(export.name.as_bytes());
            hasher_data.extend_from_slice(export.exported_hash.as_bytes());
        }

        // Include statements hash
        for statement in &self.statements {
            hasher_data.extend_from_slice(statement.content_hash().as_bytes());
        }

        ContentHash::new(&hasher_data)
    }
}

impl ImportDeclaration {
    /// Create a new import declaration
    pub fn new(module_path: String, imported_items: Vec<ImportItem>) -> Self {
        let mut hasher_data = Vec::new();
        hasher_data.extend_from_slice(module_path.as_bytes());

        for item in &imported_items {
            hasher_data.extend_from_slice(item.name.as_bytes());
            if let Some(alias) = &item.alias {
                hasher_data.extend_from_slice(alias.as_bytes());
            }
            hasher_data.push(if item.is_type { 1 } else { 0 });
        }

        let import_hash = ContentHash::new(&hasher_data);

        ImportDeclaration {
            module_path,
            module_hash: None,
            imported_items,
            import_hash,
        }
    }

    /// Set the module hash for this import
    pub fn with_module_hash(mut self, hash: ContentHash) -> Self {
        self.module_hash = Some(hash);
        self
    }
}

impl ImportItem {
    /// Create a new import item for a value
    pub fn value(name: String) -> Self {
        ImportItem {
            name,
            alias: None,
            is_type: false,
            expected_hash: None,
            compatibility_version: None,
        }
    }

    /// Create a new import item for a type
    pub fn type_item(name: String) -> Self {
        ImportItem {
            name,
            alias: None,
            is_type: true,
            expected_hash: None,
            compatibility_version: None,
        }
    }

    /// Set an alias for this import
    pub fn with_alias(mut self, alias: String) -> Self {
        self.alias = Some(alias);
        self
    }

    /// Set expected hash for compatibility checking
    pub fn with_expected_hash(mut self, hash: ContentHash) -> Self {
        self.expected_hash = Some(hash);
        self
    }

    /// Set compatibility version
    pub fn with_compatibility_version(mut self, version: String) -> Self {
        self.compatibility_version = Some(version);
        self
    }
}

impl ExportDeclaration {
    /// Create a new export declaration for a value
    pub fn value(name: String, hash: ContentHash) -> Self {
        ExportDeclaration {
            name,
            is_type: false,
            exported_hash: hash,
            visibility: ExportVisibility::Public,
            compatibility_info: CompatibilityInfo::default(),
        }
    }

    /// Create a new export declaration for a type
    pub fn type_item(name: String, hash: ContentHash) -> Self {
        ExportDeclaration {
            name,
            is_type: true,
            exported_hash: hash,
            visibility: ExportVisibility::Public,
            compatibility_info: CompatibilityInfo::default(),
        }
    }

    /// Set visibility for this export
    pub fn with_visibility(mut self, visibility: ExportVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set compatibility information
    pub fn with_compatibility_info(mut self, info: CompatibilityInfo) -> Self {
        self.compatibility_info = info;
        self
    }
}

impl Default for CompatibilityInfo {
    fn default() -> Self {
        CompatibilityInfo {
            version: "1.0.0".to_string(),
            breaking_changes: Vec::new(),
            deprecated_features: Vec::new(),
            migration_hints: Vec::new(),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        DependencyGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            cycles: Vec::new(),
        }
    }

    /// Add a module node to the graph
    pub fn add_node(&mut self, module: &Module) {
        let node = ModuleDependencyNode {
            module_id: module.id,
            module_name: module.name.clone(),
            module_hash: module.content_hash(),
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
        };

        self.nodes.insert(module.id, node);
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, from: NodeId, to: NodeId, import_items: Vec<String>) {
        // Update node dependencies
        if let Some(from_node) = self.nodes.get_mut(&from) {
            from_node.dependencies.insert(to);
        }

        if let Some(to_node) = self.nodes.get_mut(&to) {
            to_node.dependents.insert(from);
        }

        // Add edge
        let edge = DependencyEdge {
            from,
            to,
            import_items,
            dependency_type: DependencyType::Direct,
        };

        self.edges.push(edge);
    }

    /// Check if there's a path from one node to another
    pub fn has_path(&self, from: NodeId, to: NodeId) -> bool {
        if from == to {
            return true;
        }

        let mut visited = HashSet::new();
        let mut stack = vec![from];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }

            visited.insert(current);

            if let Some(node) = self.nodes.get(&current) {
                for &dep in &node.dependencies {
                    if dep == to {
                        return true;
                    }
                    if !visited.contains(&dep) {
                        stack.push(dep);
                    }
                }
            }
        }

        false
    }

    /// Detect all cycles in the graph
    pub fn detect_all_cycles(&mut self) -> Vec<Vec<NodeId>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for &node_id in self.nodes.keys() {
            if !visited.contains(&node_id) {
                self.dfs_cycles(
                    node_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        self.cycles = cycles.clone();
        cycles
    }

    fn dfs_cycles(
        &self,
        node: NodeId,
        visited: &mut HashSet<NodeId>,
        rec_stack: &mut HashSet<NodeId>,
        path: &mut Vec<NodeId>,
        cycles: &mut Vec<Vec<NodeId>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(node_info) = self.nodes.get(&node) {
            for &dep in &node_info.dependencies {
                if !visited.contains(&dep) {
                    self.dfs_cycles(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&dep) {
                    // Found a cycle
                    if let Some(cycle_start) = path.iter().position(|&x| x == dep) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
    }
}
/// Default implementation of ImportExportManager
pub struct DefaultImportExportManager {
    modules: HashMap<String, Module>,
    module_registry: HashMap<NodeId, Module>,
}

impl DefaultImportExportManager {
    /// Create a new import/export manager
    pub fn new() -> Self {
        DefaultImportExportManager {
            modules: HashMap::new(),
            module_registry: HashMap::new(),
        }
    }

    /// Register a module with the manager
    pub fn register_module(&mut self, module: Module) {
        self.module_registry.insert(module.id, module.clone());
        self.modules.insert(module.name.clone(), module);
    }

    /// Get a module by path
    pub fn get_module(&self, path: &str) -> Option<&Module> {
        self.modules.get(path)
    }

    /// Get a module by ID
    pub fn get_module_by_id(&self, id: NodeId) -> Option<&Module> {
        self.module_registry.get(&id)
    }
}

impl ImportExportManager for DefaultImportExportManager {
    fn validate_imports(&self, module: &Module) -> Result<(), Vec<ImportExportError>> {
        let mut errors = Vec::new();

        for import in &module.imports {
            // Check if the imported module exists
            let imported_module = match self.get_module(&import.module_path) {
                Some(module) => module,
                None => {
                    errors.push(ImportExportError::ModuleNotFound {
                        module_path: import.module_path.clone(),
                    });
                    continue;
                }
            };

            // Validate each imported item
            for item in &import.imported_items {
                // Find the corresponding export
                let export = imported_module
                    .exports
                    .iter()
                    .find(|e| e.name == item.name && e.is_type == item.is_type);

                match export {
                    Some(export) => {
                        // Check compatibility
                        if let CompatibilityResult::Incompatible(reasons) = self.check_import_export_compatibility(item, export) {
                            for reason in reasons {
                                errors.push(ImportExportError::CompatibilityViolation {
                                    item_name: item.name.clone(),
                                    reason,
                                });
                            }
                        }
                    }
                    None => {
                        errors.push(ImportExportError::ItemNotExported {
                            item_name: item.name.clone(),
                            module_path: import.module_path.clone(),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_exports(&self, _module: &Module) -> Result<(), Vec<ImportExportError>> {
        let errors = Vec::new();

        // For now, we assume all exports are valid if they're declared
        // In a full implementation, this would check that the exported items
        // actually exist in the module's namespace

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_import_export_compatibility(
        &self,
        import: &ImportItem,
        export: &ExportDeclaration,
    ) -> CompatibilityResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check if expected hash matches (if provided)
        if let Some(expected_hash) = &import.expected_hash {
            if *expected_hash != export.exported_hash {
                errors.push(format!(
                    "Hash mismatch: expected {}, found {}",
                    expected_hash, export.exported_hash
                ));
            }
        }

        // Check for deprecated features
        if !export.compatibility_info.deprecated_features.is_empty() {
            warnings.push(format!(
                "Using deprecated features: {:?}",
                export.compatibility_info.deprecated_features
            ));
        }

        // Check for breaking changes
        if !export.compatibility_info.breaking_changes.is_empty() {
            errors.push(format!(
                "Breaking changes detected: {:?}",
                export.compatibility_info.breaking_changes
            ));
        }

        if !errors.is_empty() {
            CompatibilityResult::Incompatible(errors)
        } else if !warnings.is_empty() {
            CompatibilityResult::CompatibleWithWarnings(warnings)
        } else {
            CompatibilityResult::Compatible
        }
    }

    fn build_dependency_graph(
        &self,
        modules: &[Module],
    ) -> Result<DependencyGraph, ImportExportError> {
        let mut graph = DependencyGraph::new();

        // Add all modules as nodes
        for module in modules {
            graph.add_node(module);
        }

        // Add dependencies based on imports
        for module in modules {
            for import in &module.imports {
                // Find the imported module
                if let Some(imported_module) = self.get_module(&import.module_path) {
                    let import_items: Vec<String> = import
                        .imported_items
                        .iter()
                        .map(|item| item.name.clone())
                        .collect();

                    graph.add_dependency(module.id, imported_module.id, import_items);
                }
            }
        }

        // Detect cycles
        let cycles = graph.detect_all_cycles();
        if !cycles.is_empty() {
            return Err(ImportExportError::CircularDependency {
                cycle: cycles[0].clone(),
            });
        }

        Ok(graph)
    }

    fn detect_cycles(&self, graph: &DependencyGraph) -> Vec<Vec<NodeId>> {
        graph.cycles.clone()
    }

    fn resolve_import(
        &self,
        import: &ImportItem,
        from_module: &Module,
    ) -> Result<ImportResolution, ImportExportError> {
        // This is a simplified implementation
        // In a full system, this would resolve the import to actual bindings
        // from the module's namespace

        // For now, return a placeholder
        Err(ImportExportError::ItemNotFound {
            item_name: import.name.clone(),
            module_path: from_module.name.clone(),
        })
    }
}

impl Default for DefaultImportExportManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_module(id: u64, name: &str) -> Module {
        Module::new(NodeId::new(id), name.to_string())
    }

    #[test]
    fn test_import_export_creation() {
        // Test import creation
        let import_item =
            ImportItem::value("test_function".to_string()).with_alias("my_function".to_string());

        assert_eq!(import_item.name, "test_function");
        assert_eq!(import_item.alias, Some("my_function".to_string()));
        assert!(!import_item.is_type);

        let import_decl = ImportDeclaration::new("test_module".to_string(), vec![import_item]);

        assert_eq!(import_decl.module_path, "test_module");
        assert_eq!(import_decl.imported_items.len(), 1);

        // Test export creation
        let export_decl = ExportDeclaration::value(
            "exported_function".to_string(),
            ContentHash::new(b"test_hash"),
        )
        .with_visibility(ExportVisibility::Protected);

        assert_eq!(export_decl.name, "exported_function");
        assert!(!export_decl.is_type);
        assert_eq!(export_decl.visibility, ExportVisibility::Protected);
    }

    #[test]
    fn test_dependency_graph_creation() {
        let mut manager = DefaultImportExportManager::new();

        // Create modules
        let mut module_a = create_test_module(1, "module_a");
        let mut module_b = create_test_module(2, "module_b");
        let module_c = create_test_module(3, "module_c");

        // Module A imports from Module B
        module_a.add_import(ImportDeclaration::new(
            "module_b".to_string(),
            vec![ImportItem::value("function_b".to_string())],
        ));

        // Module B imports from Module C
        module_b.add_import(ImportDeclaration::new(
            "module_c".to_string(),
            vec![ImportItem::type_item("TypeC".to_string())],
        ));

        // Register modules
        manager.register_module(module_a.clone());
        manager.register_module(module_b.clone());
        manager.register_module(module_c.clone());

        // Build dependency graph
        let modules = vec![module_a, module_b, module_c];
        let graph = manager.build_dependency_graph(&modules).unwrap();

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        // Check that module A depends on module B
        assert!(graph.has_path(NodeId::new(1), NodeId::new(2)));
        // Check that module A transitively depends on module C
        assert!(graph.has_path(NodeId::new(1), NodeId::new(3)));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut manager = DefaultImportExportManager::new();

        // Create modules with circular dependency
        let mut module_a = create_test_module(1, "module_a");
        let mut module_b = create_test_module(2, "module_b");

        // Module A imports from Module B
        module_a.add_import(ImportDeclaration::new(
            "module_b".to_string(),
            vec![ImportItem::value("function_b".to_string())],
        ));

        // Module B imports from Module A (circular)
        module_b.add_import(ImportDeclaration::new(
            "module_a".to_string(),
            vec![ImportItem::value("function_a".to_string())],
        ));

        // Register modules
        manager.register_module(module_a.clone());
        manager.register_module(module_b.clone());

        // Building dependency graph should detect the cycle
        let modules = vec![module_a, module_b];
        match manager.build_dependency_graph(&modules) {
            Err(ImportExportError::CircularDependency { cycle }) => {
                assert_eq!(cycle.len(), 2);
            }
            _ => panic!("Expected circular dependency error"),
        }
    }

    #[test]
    fn test_import_validation() {
        let mut manager = DefaultImportExportManager::new();

        // Create exporting module
        let mut exporting_module = create_test_module(1, "exporter");
        exporting_module.add_export(ExportDeclaration::value(
            "exported_function".to_string(),
            ContentHash::new(b"function_hash"),
        ));

        // Create importing module
        let mut importing_module = create_test_module(2, "importer");
        importing_module.add_import(ImportDeclaration::new(
            "exporter".to_string(),
            vec![ImportItem::value("exported_function".to_string())],
        ));

        // Register modules
        manager.register_module(exporting_module);
        manager.register_module(importing_module.clone());

        // Validation should succeed
        assert!(manager.validate_imports(&importing_module).is_ok());

        // Test with non-existent import
        let mut bad_importing_module = create_test_module(3, "bad_importer");
        bad_importing_module.add_import(ImportDeclaration::new(
            "exporter".to_string(),
            vec![ImportItem::value("non_existent_function".to_string())],
        ));

        match manager.validate_imports(&bad_importing_module) {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    ImportExportError::ItemNotExported { item_name, .. } => {
                        assert_eq!(item_name, "non_existent_function");
                    }
                    _ => panic!("Expected ItemNotExported error"),
                }
            }
            _ => panic!("Expected validation errors"),
        }
    }

    #[test]
    fn test_compatibility_checking() {
        let manager = DefaultImportExportManager::new();

        let import = ImportItem::value("test_item".to_string())
            .with_expected_hash(ContentHash::new(b"expected"));

        let export = ExportDeclaration::value("test_item".to_string(), ContentHash::new(b"actual"));

        // Should be incompatible due to hash mismatch
        match manager.check_import_export_compatibility(&import, &export) {
            CompatibilityResult::Incompatible(errors) => {
                assert!(!errors.is_empty());
                assert!(errors[0].contains("Hash mismatch"));
            }
            _ => panic!("Expected incompatible result"),
        }

        // Test with matching hash
        let matching_export =
            ExportDeclaration::value("test_item".to_string(), ContentHash::new(b"expected"));

        match manager.check_import_export_compatibility(&import, &matching_export) {
            CompatibilityResult::Compatible => {}
            _ => panic!("Expected compatible result"),
        }
    }
}

/// Module capabilities for actor-like behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCapabilities {
    /// Can this module send messages to other modules?
    pub can_send_messages: bool,
    /// Can this module receive messages from other modules?
    pub can_receive_messages: bool,
    /// Allowed message types this module can handle
    pub allowed_message_types: Vec<MessageType>,
}

/// Types of messages that can be sent between modules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// General data message
    Data,
    /// Control message for coordination
    Control,
    /// Error notification
    Error,
    /// Custom message type
    Custom(String),
}

/// Message passed between modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMessage {
    /// Unique message ID
    pub id: MessageId,
    /// Source module
    pub from: NodeId,
    /// Destination module
    pub to: NodeId,
    /// Message type
    pub message_type: MessageType,
    /// Message payload
    pub payload: MessagePayload,
    /// Timestamp when message was created
    pub timestamp: u64,
    /// Priority (higher numbers = higher priority)
    pub priority: u32,
}

/// Unique identifier for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub u64);

/// Message payload containing the actual data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Generic data payload
    Data(Value),
    /// Control message for coordination
    Control {
        command: String,
        parameters: HashMap<String, Value>,
    },
    /// Error information
    Error {
        error_code: String,
        error_message: String,
        context: HashMap<String, Value>,
    },
    /// Custom payload
    Custom {
        payload_type: String,
        data: HashMap<String, Value>,
    },
}

/// Error types for module operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleError {
    /// Message sending failed
    MessageSendFailed { to: NodeId, reason: String },
    /// Message processing failed
    MessageProcessingFailed {
        message_id: MessageId,
        reason: String,
    },
    /// Capability violation
    CapabilityViolation { required_capability: String },
    /// General module error
    General { message: String },
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::MessageSendFailed { to, reason } => {
                write!(f, "Failed to send message to module {to:?}: {reason}")
            }
            ModuleError::MessageProcessingFailed { message_id, reason } => {
                write!(f, "Failed to process message {message_id:?}: {reason}")
            }
            ModuleError::CapabilityViolation {
                required_capability,
            } => {
                write!(
                    f,
                    "Module lacks required capability: {required_capability}"
                )
            }
            ModuleError::General { message } => {
                write!(f, "Module error: {message}")
            }
        }
    }
}

impl std::error::Error for ModuleError {}

/// Trait for first-class module operations
pub trait FirstClassModuleOperations {
    /// Send a message to another module
    fn send_message(&mut self, to: NodeId, message: ModuleMessage) -> Result<(), ModuleError>;

    /// Receive the next message from the queue
    fn receive_message(&mut self) -> Option<ModuleMessage>;

    /// Process a received message
    fn process_message(
        &mut self,
        message: ModuleMessage,
    ) -> Result<Option<ModuleMessage>, ModuleError>;

    /// Check if the module can handle a specific message type
    fn can_handle_message(&self, message_type: &MessageType) -> bool;

    /// Get module capabilities
    fn get_capabilities(&self) -> &ModuleCapabilities;

    /// Update module capabilities
    fn update_capabilities(&mut self, capabilities: ModuleCapabilities) -> Result<(), ModuleError>;

    /// Get the module's type hash
    fn type_hash(&self) -> TypeHash;

    /// Get the number of queued messages
    fn message_queue_size(&self) -> usize;

    /// Clear the message queue
    fn clear_message_queue(&mut self);
}

impl Module {
    /// Create a new module with default capabilities
    pub fn new(id: NodeId, name: String) -> Self {
        let content_hash = ContentHash::new(name.as_bytes());
        let type_hash = TypeHash::new(content_hash);

        Module {
            id,
            name,
            imports: Vec::new(),
            exports: Vec::new(),
            statements: Vec::new(),
            type_hash,
            capabilities: ModuleCapabilities::default(),
            message_queue: VecDeque::new(),
        }
    }

    /// Create a module with specific capabilities
    pub fn with_capabilities(id: NodeId, name: String, capabilities: ModuleCapabilities) -> Self {
        let mut module = Self::new(id, name);
        module.capabilities = capabilities;
        module
    }

    /// Add an import declaration
    pub fn add_import(&mut self, import: ImportDeclaration) {
        self.imports.push(import);
        self.update_type_hash();
    }

    /// Add an export declaration
    pub fn add_export(&mut self, export: ExportDeclaration) {
        self.exports.push(export);
        self.update_type_hash();
    }

    /// Add a statement
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
        self.update_type_hash();
    }

    /// Update the type hash when module content changes
    fn update_type_hash(&mut self) {
        let content_hash = self.content_hash();
        self.type_hash = TypeHash::new(content_hash);
    }

    /// Queue a message for this module
    pub fn queue_message(&mut self, message: ModuleMessage) {
        // Insert message in priority order (higher priority first)
        let insert_pos = self
            .message_queue
            .iter()
            .position(|m| m.priority < message.priority)
            .unwrap_or(self.message_queue.len());

        self.message_queue.insert(insert_pos, message);
    }
}

impl FirstClassModuleOperations for Module {
    fn send_message(&mut self, _to: NodeId, message: ModuleMessage) -> Result<(), ModuleError> {
        // Check if module has capability to send messages
        if !self.capabilities.can_send_messages {
            return Err(ModuleError::CapabilityViolation {
                required_capability: "can_send_messages".to_string(),
            });
        }

        // Check if module can send this type of message
        if !self
            .capabilities
            .allowed_message_types
            .contains(&message.message_type)
        {
            return Err(ModuleError::CapabilityViolation {
                required_capability: format!("message_type_{:?}", message.message_type),
            });
        }

        // In a real implementation, this would send the message through
        // the distributed runtime. For now, we just validate the operation.
        Ok(())
    }

    fn receive_message(&mut self) -> Option<ModuleMessage> {
        // Check if module has capability to receive messages
        if !self.capabilities.can_receive_messages {
            return None;
        }

        self.message_queue.pop_front()
    }

    fn process_message(
        &mut self,
        message: ModuleMessage,
    ) -> Result<Option<ModuleMessage>, ModuleError> {
        // Check if module can handle this message type
        if !self.can_handle_message(&message.message_type) {
            return Err(ModuleError::MessageProcessingFailed {
                message_id: message.id,
                reason: format!("Cannot handle message type {:?}", message.message_type),
            });
        }

        // Process based on message type
        match &message.payload {
            MessagePayload::Data(_) => {
                // Handle data message
                Ok(None)
            }

            MessagePayload::Control {
                command,
                parameters: _,
            } => {
                // Handle control message
                match command.as_str() {
                    "ping" => {
                        // Respond with pong
                        Ok(Some(ModuleMessage {
                            id: MessageId::new(message.id.0 + 1),
                            from: self.id,
                            to: message.from,
                            message_type: MessageType::Control,
                            payload: MessagePayload::Control {
                                command: "pong".to_string(),
                                parameters: HashMap::new(),
                            },
                            timestamp: message.timestamp + 1,
                            priority: message.priority,
                        }))
                    }
                    _ => Ok(None),
                }
            }

            MessagePayload::Error { .. } => {
                // Handle error message
                Ok(None)
            }

            MessagePayload::Custom { .. } => {
                // Handle custom message
                Ok(None)
            }
        }
    }

    fn can_handle_message(&self, message_type: &MessageType) -> bool {
        self.capabilities
            .allowed_message_types
            .contains(message_type)
    }

    fn get_capabilities(&self) -> &ModuleCapabilities {
        &self.capabilities
    }

    fn update_capabilities(&mut self, capabilities: ModuleCapabilities) -> Result<(), ModuleError> {
        self.capabilities = capabilities;
        Ok(())
    }

    fn type_hash(&self) -> TypeHash {
        self.type_hash
    }

    fn message_queue_size(&self) -> usize {
        self.message_queue.len()
    }

    fn clear_message_queue(&mut self) {
        self.message_queue.clear();
    }
}

impl Default for ModuleCapabilities {
    fn default() -> Self {
        ModuleCapabilities {
            can_send_messages: true,
            can_receive_messages: true,
            allowed_message_types: vec![
                MessageType::Data,
                MessageType::Control,
                MessageType::Error,
            ],
        }
    }
}

impl MessageId {
    pub fn new(id: u64) -> Self {
        MessageId(id)
    }

    pub fn random() -> Self {
        MessageId(rand::random())
    }
}

#[cfg(test)]
mod first_class_module_tests {
    use super::*;

    #[test]
    fn test_module_as_first_class_value() {
        let module = Module::new(NodeId::new(1), "test_module".to_string());

        assert_eq!(module.name, "test_module");
        assert_eq!(module.message_queue_size(), 0);
        assert!(module.get_capabilities().can_send_messages);
        assert!(module.get_capabilities().can_receive_messages);
    }

    #[test]
    fn test_module_capabilities() {
        let mut capabilities = ModuleCapabilities::default();
        capabilities
            .allowed_message_types
            .push(MessageType::Custom("special".to_string()));

        let module =
            Module::with_capabilities(NodeId::new(1), "test_module".to_string(), capabilities);

        assert!(module.can_handle_message(&MessageType::Data));
        assert!(module.can_handle_message(&MessageType::Custom("special".to_string())));
        assert!(!module.can_handle_message(&MessageType::Custom("other".to_string())));
    }

    #[test]
    fn test_message_handling() {
        let mut module = Module::new(NodeId::new(1), "test_module".to_string());

        // Create a ping message
        let ping_message = ModuleMessage {
            id: MessageId::new(1),
            from: NodeId::new(2),
            to: NodeId::new(1),
            message_type: MessageType::Control,
            payload: MessagePayload::Control {
                command: "ping".to_string(),
                parameters: HashMap::new(),
            },
            timestamp: 1000,
            priority: 10,
        };

        // Process the message
        let response = module.process_message(ping_message).unwrap();

        // Should receive pong response
        assert!(response.is_some());
        if let Some(resp) = response {
            match resp.payload {
                MessagePayload::Control { command, .. } => {
                    assert_eq!(command, "pong");
                }
                _ => panic!("Expected Control payload with pong command"),
            }
        }
    }

    #[test]
    fn test_message_queue_priority() {
        let mut module = Module::new(NodeId::new(1), "test_module".to_string());

        // Add messages with different priorities
        let low_priority = ModuleMessage {
            id: MessageId::new(1),
            from: NodeId::new(2),
            to: NodeId::new(1),
            message_type: MessageType::Data,
            payload: MessagePayload::Data(Value::I32(1)),
            timestamp: 1000,
            priority: 1,
        };

        let high_priority = ModuleMessage {
            id: MessageId::new(2),
            from: NodeId::new(2),
            to: NodeId::new(1),
            message_type: MessageType::Data,
            payload: MessagePayload::Data(Value::I32(2)),
            timestamp: 1001,
            priority: 10,
        };

        // Queue messages (low priority first)
        module.queue_message(low_priority);
        module.queue_message(high_priority);

        // High priority message should be received first
        let first_message = module.receive_message().unwrap();
        assert_eq!(first_message.priority, 10);

        let second_message = module.receive_message().unwrap();
        assert_eq!(second_message.priority, 1);
    }

    #[test]
    fn test_capability_violations() {
        let mut module = Module::new(NodeId::new(1), "test_module".to_string());

        // Remove message sending capability
        let mut capabilities = module.capabilities.clone();
        capabilities.can_send_messages = false;
        module.update_capabilities(capabilities).unwrap();

        // Try to send a message
        let message = ModuleMessage {
            id: MessageId::new(1),
            from: NodeId::new(1),
            to: NodeId::new(2),
            message_type: MessageType::Data,
            payload: MessagePayload::Data(Value::I32(42)),
            timestamp: 1000,
            priority: 10,
        };

        // Should fail due to capability violation
        match module.send_message(NodeId::new(2), message) {
            Err(ModuleError::CapabilityViolation {
                required_capability,
            }) => {
                assert_eq!(required_capability, "can_send_messages");
            }
            _ => panic!("Expected capability violation error"),
        }
    }

    #[test]
    fn test_module_type_hash_updates() {
        let mut module = Module::new(NodeId::new(1), "test_module".to_string());
        let initial_hash = module.type_hash();

        // Add an import - should update type hash
        module.add_import(ImportDeclaration::new(
            "other_module".to_string(),
            vec![ImportItem::value("function".to_string())],
        ));

        let new_hash = module.type_hash();
        assert_ne!(initial_hash, new_hash);
    }
}
