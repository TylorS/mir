//! Module storage system with content-addressable hashing

use mir_types::{ContentHash, TypeHash};
use mir_ast::{Module, NodeId};
use crate::schema_storage::{SchemaAwareStore, StorageError};
use crate::storage::ContentAddressableStore;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Module storage system
pub struct ModuleStore<S: ContentAddressableStore> {
    /// Schema-aware storage for modules
    storage: SchemaAwareStore<S>,
    /// Module metadata index
    module_index: HashMap<NodeId, ModuleMetadata>,
    /// Content hash to module ID mapping
    hash_to_module: HashMap<ContentHash, NodeId>,
    /// Dependency graph
    dependency_graph: DependencyGraph,
    /// Import/export registry
    import_export_registry: ImportExportRegistry,
}

/// Metadata for stored modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub module_id: NodeId,
    pub content_hash: ContentHash,
    pub schema_hash: ContentHash,
    pub version: ModuleVersion,
    pub dependencies: Vec<NodeId>,
    pub exports: Vec<ExportInfo>,
    pub imports: Vec<ImportInfo>,
    pub stored_at: u64,
    pub last_modified: u64,
}

/// Module version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

/// Export information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    pub name: String,
    pub export_type: ExportType,
    pub type_hash: Option<TypeHash>,
    pub is_public: bool,
}

/// Import information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub name: String,
    pub module_path: String,
    pub import_type: ImportType,
    pub type_hash: Option<TypeHash>,
    pub alias: Option<String>,
}

/// Type of export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    Value,
    Type,
    Module,
    Function,
}

/// Type of import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportType {
    Value,
    Type,
    Module,
    Function,
}

/// Dependency graph for modules
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// Direct dependencies: module -> [dependencies]
    dependencies: HashMap<NodeId, HashSet<NodeId>>,
    /// Reverse dependencies: module -> [dependents]
    dependents: HashMap<NodeId, HashSet<NodeId>>,
}

/// Import/export registry for interface validation
#[derive(Debug, Clone, Default)]
pub struct ImportExportRegistry {
    /// Exports by module: module -> [exports]
    exports: HashMap<NodeId, Vec<ExportInfo>>,
    /// Imports by module: module -> [imports]
    imports: HashMap<NodeId, Vec<ImportInfo>>,
    /// Export index: (module, name) -> export_info
    export_index: HashMap<(NodeId, String), ExportInfo>,
}

/// Module storage result
#[derive(Debug, Clone)]
pub struct ModuleStoreResult {
    pub module_id: NodeId,
    pub content_hash: ContentHash,
    pub was_updated: bool,
    pub dependency_changes: Vec<DependencyChange>,
}

/// Dependency change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyChange {
    pub change_type: DependencyChangeType,
    pub module_id: NodeId,
    pub old_hash: Option<ContentHash>,
    pub new_hash: Option<ContentHash>,
}

/// Type of dependency change
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DependencyChangeType {
    Added,
    Removed,
    Modified,
}

/// Module query parameters
#[derive(Debug, Clone, Default)]
pub struct ModuleQueryParams {
    pub include_dependencies: bool,
    pub include_metadata: bool,
    pub target_version: Option<ModuleVersion>,
    pub allow_migration: bool,
}

impl<S: ContentAddressableStore> ModuleStore<S> {
    /// Create a new module store
    pub fn new(storage: S) -> Self {
        ModuleStore {
            storage: SchemaAwareStore::new(storage),
            module_index: HashMap::new(),
            hash_to_module: HashMap::new(),
            dependency_graph: DependencyGraph::default(),
            import_export_registry: ImportExportRegistry::default(),
        }
    }
    
    /// Store a module with content-addressable hashing
    pub fn store_module(&mut self, module: &Module) -> Result<ModuleStoreResult, ModuleStoreError> {
        // Calculate content hash for the module
        let module_content = self.serialize_module(module)?;
        let content_hash = ContentHash::new(&module_content);
        
        // Check if module already exists with same content
        let was_updated = if let Some(existing_metadata) = self.module_index.get(&module.id) {
            existing_metadata.content_hash != content_hash
        } else {
            true
        };
        
        // Extract module metadata
        let metadata = self.extract_module_metadata(module, content_hash)?;
        
        // Store module content directly in underlying storage
        let content_hash = self.storage.storage_mut().store(&module_content);
        
        // Update dependency graph
        let dependency_changes = self.update_dependency_graph(&module.id, &metadata.dependencies)?;
        
        // Update import/export registry
        self.update_import_export_registry(&module.id, &metadata.exports, &metadata.imports)?;
        
        // Update indices
        self.module_index.insert(module.id, metadata);
        self.hash_to_module.insert(content_hash, module.id);
        
        Ok(ModuleStoreResult {
            module_id: module.id,
            content_hash,
            was_updated,
            dependency_changes,
        })
    }
    
    /// Retrieve a module by ID
    pub fn retrieve_module(&self, module_id: &NodeId, _params: ModuleQueryParams) -> Result<Option<Module>, ModuleStoreError> {
        let metadata = match self.module_index.get(module_id) {
            Some(metadata) => metadata,
            None => return Ok(None),
        };
        
        // Retrieve from underlying storage
        let data = match self.storage.storage().retrieve(metadata.content_hash) {
            Some(data) => data,
            None => return Ok(None),
        };
        
        // Deserialize module
        let module = serde_json::from_slice(&data)
            .map_err(|e| ModuleStoreError::Deserialization(e.to_string()))?;
        
        Ok(Some(module))
    }
    
    /// Retrieve a module by content hash
    pub fn retrieve_module_by_hash(&self, content_hash: ContentHash, params: ModuleQueryParams) -> Result<Option<Module>, ModuleStoreError> {
        let module_id = match self.hash_to_module.get(&content_hash) {
            Some(id) => id,
            None => return Ok(None),
        };
        
        self.retrieve_module(module_id, params)
    }
    
    /// Get module metadata
    pub fn get_module_metadata(&self, module_id: &NodeId) -> Option<&ModuleMetadata> {
        self.module_index.get(module_id)
    }
    
    /// Get all modules that depend on a given module
    pub fn get_dependents(&self, module_id: &NodeId) -> Vec<NodeId> {
        self.dependency_graph.dependents
            .get(module_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get all dependencies of a given module
    pub fn get_dependencies(&self, module_id: &NodeId) -> Vec<NodeId> {
        self.dependency_graph.dependencies
            .get(module_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Validate import/export compatibility
    pub fn validate_compatibility(&self, module_id: &NodeId) -> Result<CompatibilityReport, ModuleStoreError> {
        let mut report = CompatibilityReport {
            module_id: *module_id,
            issues: Vec::new(),
            is_compatible: true,
        };
        
        // Get module imports
        let empty_imports = Vec::new();
        let imports = self.import_export_registry.imports.get(module_id).unwrap_or(&empty_imports);
        
        // Check each import
        for import in imports {
            // For now, just check basic structure
            // In a full implementation, this would resolve module paths to IDs
            report.issues.push(CompatibilityIssue {
                issue_type: CompatibilityIssueType::MissingModule,
                description: format!("Module {} validation not fully implemented", import.module_path),
                severity: IssueSeverity::Warning,
            });
        }
        
        Ok(report)
    }
    
    /// Perform module-aware garbage collection
    pub fn module_gc(&mut self, reachable_modules: &HashSet<NodeId>) -> Result<ModuleGCResult, ModuleStoreError> {
        let mut reachable_hashes = HashSet::new();
        let mut removed_modules = Vec::new();
        
        // Collect content hashes for reachable modules
        for module_id in reachable_modules {
            if let Some(metadata) = self.module_index.get(module_id) {
                reachable_hashes.insert(metadata.content_hash);
            }
        }
        
        // Find modules to remove
        let all_modules: Vec<NodeId> = self.module_index.keys().cloned().collect();
        for module_id in all_modules {
            if !reachable_modules.contains(&module_id) {
                removed_modules.push(module_id);
                
                // Remove from indices
                if let Some(metadata) = self.module_index.remove(&module_id) {
                    self.hash_to_module.remove(&metadata.content_hash);
                }
                
                // Remove from dependency graph
                self.dependency_graph.dependencies.remove(&module_id);
                self.dependency_graph.dependents.remove(&module_id);
                
                // Remove from import/export registry
                self.import_export_registry.exports.remove(&module_id);
                self.import_export_registry.imports.remove(&module_id);
            }
        }
        
        // Perform storage GC on underlying storage
        self.storage.storage_mut().garbage_collect(&reachable_hashes);
        
        Ok(ModuleGCResult {
            removed_modules,
            storage_gc_result: crate::schema_storage::GCResult {
                migrated_count: 0,
                removed_count: 0,
                total_reachable: reachable_hashes.len(),
            },
        })
    }
    
    /// Get module store statistics
    pub fn stats(&self) -> ModuleStoreStats {
        let schema_stats = self.storage.schema_stats();
        
        ModuleStoreStats {
            total_modules: self.module_index.len(),
            total_dependencies: self.dependency_graph.dependencies.values().map(|deps| deps.len()).sum(),
            total_exports: self.import_export_registry.exports.values().map(|exports| exports.len()).sum(),
            total_imports: self.import_export_registry.imports.values().map(|imports| imports.len()).sum(),
            schema_stats,
        }
    }
    
    // Helper methods
    
    fn serialize_module(&self, module: &Module) -> Result<Vec<u8>, ModuleStoreError> {
        serde_json::to_vec(module)
            .map_err(|e| ModuleStoreError::Serialization(e.to_string()))
    }
    

    
    fn extract_module_metadata(&self, module: &Module, content_hash: ContentHash) -> Result<ModuleMetadata, ModuleStoreError> {
        // Extract dependencies from imports (simplified)
        let dependencies = Vec::new(); // TODO: Extract from actual import structure
        
        let exports = module.exports.iter()
            .map(|export| ExportInfo {
                name: export.name.clone(),
                export_type: if export.is_type { ExportType::Type } else { ExportType::Value },
                type_hash: None, // TODO: Extract from export structure
                is_public: matches!(export.visibility, mir_ast::ExportVisibility::Public),
            })
            .collect();
        
        let imports = module.imports.iter()
            .flat_map(|import_decl| {
                import_decl.imported_items.iter().map(|item| ImportInfo {
                    name: item.name.clone(),
                    module_path: import_decl.module_path.clone(),
                    import_type: if item.is_type { ImportType::Type } else { ImportType::Value },
                    type_hash: None, // TODO: Extract from import structure
                    alias: item.alias.clone(),
                })
            })
            .collect();
        
        let now = current_timestamp();
        
        Ok(ModuleMetadata {
            module_id: module.id,
            content_hash,
            schema_hash: self.get_module_schema_hash(),
            version: ModuleVersion {
                major: 1,
                minor: 0,
                patch: 0,
                pre_release: None,
            },
            dependencies,
            exports,
            imports,
            stored_at: now,
            last_modified: now,
        })
    }
    
    fn update_dependency_graph(&mut self, module_id: &NodeId, dependencies: &[NodeId]) -> Result<Vec<DependencyChange>, ModuleStoreError> {
        let mut changes = Vec::new();
        
        // Get old dependencies
        let old_deps = self.dependency_graph.dependencies.get(module_id).cloned().unwrap_or_default();
        let new_deps: HashSet<NodeId> = dependencies.iter().cloned().collect();
        
        // Find added dependencies
        for dep in &new_deps {
            if !old_deps.contains(dep) {
                changes.push(DependencyChange {
                    change_type: DependencyChangeType::Added,
                    module_id: *dep,
                    old_hash: None,
                    new_hash: self.module_index.get(dep).map(|m| m.content_hash),
                });
            }
        }
        
        // Find removed dependencies
        for dep in &old_deps {
            if !new_deps.contains(dep) {
                changes.push(DependencyChange {
                    change_type: DependencyChangeType::Removed,
                    module_id: *dep,
                    old_hash: self.module_index.get(dep).map(|m| m.content_hash),
                    new_hash: None,
                });
            }
        }
        
        // Update dependency graph
        self.dependency_graph.dependencies.insert(*module_id, new_deps.clone());
        
        // Update reverse dependencies
        for dep in &new_deps {
            self.dependency_graph.dependents.entry(*dep).or_default().insert(*module_id);
        }
        
        Ok(changes)
    }
    
    fn update_import_export_registry(&mut self, module_id: &NodeId, exports: &[ExportInfo], imports: &[ImportInfo]) -> Result<(), ModuleStoreError> {
        // Update exports
        self.import_export_registry.exports.insert(*module_id, exports.to_vec());
        
        // Update export index
        for export in exports {
            let key = (*module_id, export.name.clone());
            self.import_export_registry.export_index.insert(key, export.clone());
        }
        
        // Update imports
        self.import_export_registry.imports.insert(*module_id, imports.to_vec());
        
        Ok(())
    }
    
    fn get_module_schema_hash(&self) -> ContentHash {
        // In a real implementation, this would be the hash of the module schema
        ContentHash::new(b"module_schema_v1")
    }
}

/// Module store statistics
#[derive(Debug, Clone)]
pub struct ModuleStoreStats {
    pub total_modules: usize,
    pub total_dependencies: usize,
    pub total_exports: usize,
    pub total_imports: usize,
    pub schema_stats: crate::schema_storage::SchemaStats,
}

/// Module garbage collection result
#[derive(Debug, Clone)]
pub struct ModuleGCResult {
    pub removed_modules: Vec<NodeId>,
    pub storage_gc_result: crate::schema_storage::GCResult,
}

/// Compatibility report for a module
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    pub module_id: NodeId,
    pub issues: Vec<CompatibilityIssue>,
    pub is_compatible: bool,
}

/// Compatibility issue
#[derive(Debug, Clone)]
pub struct CompatibilityIssue {
    pub issue_type: CompatibilityIssueType,
    pub description: String,
    pub severity: IssueSeverity,
}

/// Type of compatibility issue
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityIssueType {
    MissingModule,
    MissingExport,
    TypeMismatch,
    VersionMismatch,
}

/// Severity of compatibility issue
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Module store errors
#[derive(Debug, Clone)]
pub enum ModuleStoreError {
    Storage(StorageError),
    Serialization(String),
    Deserialization(String),
    InvalidValue(String),
    ModuleNotFound(NodeId),
    DependencyError(String),
}

impl std::fmt::Display for ModuleStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleStoreError::Storage(err) => write!(f, "Storage error: {err}"),
            ModuleStoreError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            ModuleStoreError::Deserialization(msg) => write!(f, "Deserialization error: {msg}"),
            ModuleStoreError::InvalidValue(msg) => write!(f, "Invalid value: {msg}"),
            ModuleStoreError::ModuleNotFound(id) => write!(f, "Module not found: {id:?}"),
            ModuleStoreError::DependencyError(msg) => write!(f, "Dependency error: {msg}"),
        }
    }
}

impl std::error::Error for ModuleStoreError {}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStore;

    fn create_test_module(id: u64, name: &str) -> Module {
        use std::collections::VecDeque;
        
        Module {
            id: NodeId::new(id),
            name: name.to_string(),
            imports: Vec::new(),
            exports: Vec::new(),
            statements: Vec::new(),
            type_hash: TypeHash::new(ContentHash::new(name.as_bytes())),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: Vec::new(),
            },
            message_queue: VecDeque::new(),
        }
    }

    #[test]
    fn test_module_store_basic_operations() {
        let storage = InMemoryStore::new();
        let mut module_store = ModuleStore::new(storage);
        
        let module = create_test_module(1, "test_module");
        let result = module_store.store_module(&module).unwrap();
        
        assert_eq!(result.module_id, module.id);
        assert!(result.was_updated);
        
        let params = ModuleQueryParams {
            include_dependencies: false,
            include_metadata: true,
            target_version: None,
            allow_migration: false,
        };
        
        let retrieved = module_store.retrieve_module(&module.id, params).unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_module = retrieved.unwrap();
        assert_eq!(retrieved_module.id, module.id);
    }
    
    #[test]
    fn test_module_deduplication() {
        let storage = InMemoryStore::new();
        let mut module_store = ModuleStore::new(storage);
        
        let module = create_test_module(1, "test_module");
        
        // Store the same module twice
        let result1 = module_store.store_module(&module).unwrap();
        let result2 = module_store.store_module(&module).unwrap();
        
        // Content hashes should be the same
        assert_eq!(result1.content_hash, result2.content_hash);
        
        // Second store should not be considered an update
        assert!(!result2.was_updated);
    }
}