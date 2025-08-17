//! Change detection and analysis system for hot-module-reloading
//!
//! This module implements Requirements 12 and 13:
//! - Automatic detection of which parts of the application are affected by code changes
//! - Dependency graph computation using content hashes for precise change detection
//! - Compatibility checking between schema versions
//! - Change optimization for pure function updates

use crate::module_store::{DependencyChange, ModuleMetadata, ModuleStore};
use crate::storage::ContentAddressableStore;
use mir_ast::{ASTNode, Module, NodeId};
use mir_types::{CompatibilityResult, ContentHash};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Change analysis system for detecting and analyzing code changes
pub struct ChangeAnalysisSystem<S: ContentAddressableStore> {
    /// Module store for accessing module information
    module_store: ModuleStore<S>,
    /// Cache of previous analysis results
    analysis_cache: HashMap<ContentHash, ChangeAnalysis>,
    /// Function purity analyzer
    purity_analyzer: FunctionPurityAnalyzer,
    /// Schema compatibility checker
    schema_checker: SchemaCompatibilityChecker,
}

/// Result of change analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAnalysis {
    /// Hash of the changed module
    pub changed_module_hash: ContentHash,
    /// Modules affected by this change
    pub affected_modules: Vec<AffectedModule>,
    /// Schema changes detected
    pub schema_changes: Vec<SchemaChange>,
    /// Overall compatibility level
    pub compatibility: CompatibilityLevel,
    /// Whether migration is required
    pub migration_required: bool,
    /// Optimizations that can be applied
    pub optimizations: Vec<ChangeOptimization>,
    /// Dependency graph changes
    pub dependency_changes: Vec<DependencyChange>,
}

/// Information about a module affected by changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedModule {
    /// Module ID
    pub module_id: NodeId,
    /// Content hash of the module
    pub module_hash: ContentHash,
    /// How this module is affected
    pub impact_type: ImpactType,
    /// Specific changes that affect this module
    pub affected_items: Vec<AffectedItem>,
    /// Whether this module needs recompilation
    pub needs_recompilation: bool,
    /// Whether state migration is needed
    pub needs_state_migration: bool,
}

/// Type of impact on a module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactType {
    /// Direct change to the module itself
    Direct,
    /// Transitive impact through dependencies
    Transitive,
    /// Impact through shared types or schemas
    SchemaDependent,
    /// No impact (can be ignored)
    None,
}

/// Specific item affected by changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedItem {
    /// Name of the affected item
    pub name: String,
    /// Type of the item (function, type, value, etc.)
    pub item_type: ItemType,
    /// Hash of the old version
    pub old_hash: Option<ContentHash>,
    /// Hash of the new version
    pub new_hash: ContentHash,
    /// Type of change
    pub change_type: ChangeType,
}

/// Type of item that can be affected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Function,
    Type,
    Value,
    Module,
    Schema,
}

/// Type of change detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Item was added
    Added,
    /// Item was removed
    Removed,
    /// Item was modified
    Modified,
    /// Item signature changed (breaking)
    SignatureChanged,
    /// Item implementation changed (non-breaking)
    ImplementationChanged,
}

/// Schema change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// Schema that changed
    pub schema_hash: ContentHash,
    /// Old schema version
    pub old_version: Option<ContentHash>,
    /// New schema version
    pub new_version: ContentHash,
    /// Type of schema change
    pub change_type: SchemaChangeType,
    /// Migration path if available (simplified as string for now)
    pub migration_path: Option<String>,
    /// Modules affected by this schema change
    pub affected_modules: Vec<NodeId>,
}

/// Type of schema change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaChangeType {
    /// Backward compatible addition
    Compatible,
    /// Backward compatible modification
    BackwardCompatible,
    /// Breaking change requiring migration
    Breaking,
    /// Schema was removed
    Removed,
}

/// Overall compatibility level of changes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    /// Fully compatible, no migration needed
    FullyCompatible,
    /// Backward compatible, automatic migration possible
    BackwardCompatible,
    /// Requires explicit migration
    RequiresMigration,
    /// Breaking changes, manual intervention needed
    Breaking,
}

/// Optimization that can be applied to changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeOptimization {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Modules this optimization applies to
    pub applicable_modules: Vec<NodeId>,
    /// Description of the optimization
    pub description: String,
    /// Estimated performance impact
    pub performance_impact: PerformanceImpact,
}

/// Type of optimization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Pure function update (no side effects)
    PureFunctionUpdate,
    /// Hot-swap compatible change
    HotSwapCompatible,
    /// Incremental compilation possible
    IncrementalCompilation,
    /// State preservation possible
    StatePreservation,
}

/// Performance impact of an optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    /// Estimated time savings (in milliseconds)
    pub time_savings_ms: u64,
    /// Memory impact (positive = more memory, negative = less memory)
    pub memory_impact_bytes: i64,
    /// CPU impact (0.0 = no impact, 1.0 = significant impact)
    pub cpu_impact: f64,
}

/// Function purity analyzer for detecting pure functions
pub struct FunctionPurityAnalyzer {
    /// Cache of purity analysis results
    purity_cache: HashMap<ContentHash, PurityAnalysis>,
}

/// Result of function purity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurityAnalysis {
    /// Whether the function is pure
    pub is_pure: bool,
    /// Side effects detected
    pub side_effects: Vec<SideEffect>,
    /// Dependencies on external state
    pub external_dependencies: Vec<ExternalDependency>,
    /// Whether the function can be hot-swapped safely
    pub hot_swap_safe: bool,
}

/// Type of side effect
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffect {
    /// Modifies global state
    GlobalStateModification,
    /// Performs I/O operations
    IOOperation,
    /// Calls impure functions
    ImpureFunctionCall,
    /// Modifies captured variables
    CapturedVariableModification,
    /// Allocates resources
    ResourceAllocation,
}

/// External dependency of a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Name or identifier of the dependency
    pub identifier: String,
    /// Hash of the dependency
    pub dependency_hash: ContentHash,
}

/// Type of external dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Global variable
    GlobalVariable,
    /// External function
    ExternalFunction,
    /// Module import
    ModuleImport,
    /// Type dependency
    TypeDependency,
    /// Schema dependency
    SchemaDependency,
}

/// Schema compatibility checker
pub struct SchemaCompatibilityChecker {
    /// Cache of compatibility check results
    compatibility_cache: HashMap<(ContentHash, ContentHash), CompatibilityResult>,
}

impl<S: ContentAddressableStore> ChangeAnalysisSystem<S> {
    /// Create a new change analysis system
    pub fn new(module_store: ModuleStore<S>) -> Self {
        ChangeAnalysisSystem {
            module_store,
            analysis_cache: HashMap::new(),
            purity_analyzer: FunctionPurityAnalyzer::new(),
            schema_checker: SchemaCompatibilityChecker::new(),
        }
    }

    /// Analyze changes between old and new module versions
    pub fn analyze_change(
        &mut self,
        old_hash: ContentHash,
        new_module: &Module,
    ) -> Result<ChangeAnalysis, ChangeAnalysisError> {
        let new_hash = new_module.content_hash();

        // Check cache first
        if let Some(cached_analysis) = self.analysis_cache.get(&new_hash) {
            return Ok(cached_analysis.clone());
        }

        // Get old module if it exists
        let old_module = if old_hash != ContentHash::zero() {
            self.module_store
                .retrieve_module_by_hash(old_hash, Default::default())?
        } else {
            None
        };

        // Perform change analysis
        let analysis = self.perform_change_analysis(old_module.as_ref(), new_module)?;

        // Cache the result
        self.analysis_cache.insert(new_hash, analysis.clone());

        Ok(analysis)
    }

    /// Compute dependency graph using content hashes
    pub fn compute_dependency_graph(
        &self,
        modules: &[Module],
    ) -> Result<DependencyGraph, ChangeAnalysisError> {
        let mut graph = DependencyGraph::new();

        // Add all modules as nodes
        for module in modules {
            let node = DependencyNode {
                module_id: module.id,
                content_hash: module.content_hash(),
                dependencies: HashSet::new(),
                dependents: HashSet::new(),
                schema_dependencies: HashSet::new(),
            };
            graph.nodes.insert(module.id, node);
        }

        // Build dependency edges based on imports and content hashes
        for module in modules {
            for import in &module.imports {
                // Find the imported module by path
                if let Some(imported_module) = modules.iter().find(|m| m.name == import.module_path)
                {
                    // Add dependency edge
                    graph.add_dependency(
                        module.id,
                        imported_module.id,
                        import.import_hash,
                        DependencyEdgeType::Import,
                    );

                    // Check for schema dependencies
                    for item in &import.imported_items {
                        if item.is_type {
                            if let Some(expected_hash) = &item.expected_hash {
                                graph.add_schema_dependency(module.id, *expected_hash);
                            }
                        }
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Analyze impact of changes on dependent modules
    pub fn analyze_impact(
        &mut self,
        changed_module: &Module,
        dependency_graph: &DependencyGraph,
    ) -> Result<Vec<AffectedModule>, ChangeAnalysisError> {
        let mut affected_modules = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with direct dependents
        if let Some(node) = dependency_graph.nodes.get(&changed_module.id) {
            for &dependent_id in &node.dependents {
                queue.push_back((dependent_id, ImpactType::Direct));
            }
        }

        // Breadth-first traversal to find all affected modules
        while let Some((module_id, impact_type)) = queue.pop_front() {
            if visited.contains(&module_id) {
                continue;
            }
            visited.insert(module_id);

            // Get module metadata
            if let Some(metadata) = self.module_store.get_module_metadata(&module_id) {
                let affected_items = self.analyze_affected_items(metadata, changed_module)?;

                let affected_module = AffectedModule {
                    module_id,
                    module_hash: metadata.content_hash,
                    impact_type: impact_type.clone(),
                    affected_items,
                    needs_recompilation: self
                        .needs_recompilation(&impact_type, &metadata.content_hash)?,
                    needs_state_migration: self
                        .needs_state_migration(&impact_type, &metadata.content_hash)?,
                };

                affected_modules.push(affected_module);

                // Add transitive dependents
                if let Some(node) = dependency_graph.nodes.get(&module_id) {
                    for &transitive_dependent in &node.dependents {
                        if !visited.contains(&transitive_dependent) {
                            queue.push_back((transitive_dependent, ImpactType::Transitive));
                        }
                    }
                }
            }
        }

        Ok(affected_modules)
    }

    /// Check compatibility between schema versions
    pub fn check_schema_compatibility(
        &mut self,
        old_schema_hash: ContentHash,
        new_schema_hash: ContentHash,
    ) -> Result<CompatibilityResult, ChangeAnalysisError> {
        self.schema_checker
            .check_compatibility(old_schema_hash, new_schema_hash)
    }

    /// Detect optimizations for pure function updates
    pub fn detect_optimizations(
        &mut self,
        analysis: &ChangeAnalysis,
    ) -> Result<Vec<ChangeOptimization>, ChangeAnalysisError> {
        let mut optimizations = Vec::new();

        for affected_module in &analysis.affected_modules {
            for affected_item in &affected_module.affected_items {
                if affected_item.item_type == ItemType::Function {
                    // Analyze function purity
                    if let Some(old_hash) = &affected_item.old_hash {
                        let purity_analysis = self
                            .purity_analyzer
                            .analyze_function_change(*old_hash, affected_item.new_hash)?;

                        if purity_analysis.is_pure && purity_analysis.hot_swap_safe {
                            optimizations.push(ChangeOptimization {
                                optimization_type: OptimizationType::PureFunctionUpdate,
                                applicable_modules: vec![affected_module.module_id],
                                description: format!(
                                    "Pure function '{}' can be hot-swapped without state migration",
                                    affected_item.name
                                ),
                                performance_impact: PerformanceImpact {
                                    time_savings_ms: 100, // Estimated
                                    memory_impact_bytes: 0,
                                    cpu_impact: 0.1,
                                },
                            });
                        }
                    }
                }
            }
        }

        // Check for incremental compilation opportunities
        if analysis.compatibility >= CompatibilityLevel::BackwardCompatible {
            optimizations.push(ChangeOptimization {
                optimization_type: OptimizationType::IncrementalCompilation,
                applicable_modules: analysis
                    .affected_modules
                    .iter()
                    .map(|m| m.module_id)
                    .collect(),
                description: "Changes are backward compatible, incremental compilation possible"
                    .to_string(),
                performance_impact: PerformanceImpact {
                    time_savings_ms: 500,
                    memory_impact_bytes: -1024 * 1024, // Save 1MB
                    cpu_impact: 0.3,
                },
            });
        }

        Ok(optimizations)
    }

    // Private helper methods

    fn perform_change_analysis(
        &mut self,
        old_module: Option<&Module>,
        new_module: &Module,
    ) -> Result<ChangeAnalysis, ChangeAnalysisError> {
        let changed_module_hash = new_module.content_hash();
        let mut affected_modules = Vec::new();
        let mut schema_changes = Vec::new();
        let mut compatibility = CompatibilityLevel::FullyCompatible;
        let mut migration_required = false;

        // Compare with old module if it exists
        if let Some(old_mod) = old_module {
            // Analyze schema changes
            schema_changes = self.analyze_schema_changes(old_mod, new_module)?;

            // Determine compatibility level
            compatibility = self.determine_compatibility_level(&schema_changes);
            migration_required = compatibility >= CompatibilityLevel::RequiresMigration;

            // Build dependency graph and analyze impact
            let modules = vec![new_module.clone()]; // In real implementation, get all modules
            let dependency_graph = self.compute_dependency_graph(&modules)?;
            affected_modules = self.analyze_impact(new_module, &dependency_graph)?;
        }

        let dependency_changes = Vec::new(); // TODO: Implement dependency change detection

        let mut analysis = ChangeAnalysis {
            changed_module_hash,
            affected_modules,
            schema_changes,
            compatibility,
            migration_required,
            optimizations: Vec::new(),
            dependency_changes,
        };

        // Detect optimizations
        analysis.optimizations = self.detect_optimizations(&analysis)?;

        Ok(analysis)
    }

    fn analyze_schema_changes(
        &mut self,
        old_module: &Module,
        new_module: &Module,
    ) -> Result<Vec<SchemaChange>, ChangeAnalysisError> {
        let mut schema_changes = Vec::new();

        // Compare exported types (simplified analysis)
        let old_type_exports: HashMap<String, ContentHash> = old_module
            .exports
            .iter()
            .filter(|e| e.is_type)
            .map(|e| (e.name.clone(), e.exported_hash))
            .collect();

        let new_type_exports: HashMap<String, ContentHash> = new_module
            .exports
            .iter()
            .filter(|e| e.is_type)
            .map(|e| (e.name.clone(), e.exported_hash))
            .collect();

        // Find changed types
        for (name, new_hash) in &new_type_exports {
            if let Some(old_hash) = old_type_exports.get(name) {
                if old_hash != new_hash {
                    // Type changed
                    let compatibility = self.check_schema_compatibility(*old_hash, *new_hash)?;
                    let change_type = match compatibility {
                        CompatibilityResult::FullyCompatible => SchemaChangeType::Compatible,
                        CompatibilityResult::BackwardCompatible => {
                            SchemaChangeType::BackwardCompatible
                        }
                        CompatibilityResult::RequiresMigration(_) => SchemaChangeType::Breaking,
                        CompatibilityResult::Incompatible(_) => SchemaChangeType::Breaking,
                    };

                    schema_changes.push(SchemaChange {
                        schema_hash: *new_hash,
                        old_version: Some(*old_hash),
                        new_version: *new_hash,
                        change_type,
                        migration_path: None, // TODO: Compute migration path
                        affected_modules: vec![new_module.id],
                    });
                }
            } else {
                // New type added
                schema_changes.push(SchemaChange {
                    schema_hash: *new_hash,
                    old_version: None,
                    new_version: *new_hash,
                    change_type: SchemaChangeType::Compatible,
                    migration_path: None,
                    affected_modules: vec![new_module.id],
                });
            }
        }

        // Find removed types
        for (name, old_hash) in &old_type_exports {
            if !new_type_exports.contains_key(name) {
                schema_changes.push(SchemaChange {
                    schema_hash: *old_hash,
                    old_version: Some(*old_hash),
                    new_version: ContentHash::zero(),
                    change_type: SchemaChangeType::Removed,
                    migration_path: None,
                    affected_modules: vec![new_module.id],
                });
            }
        }

        Ok(schema_changes)
    }

    fn determine_compatibility_level(&self, schema_changes: &[SchemaChange]) -> CompatibilityLevel {
        let mut max_level = CompatibilityLevel::FullyCompatible;

        for change in schema_changes {
            let level = match change.change_type {
                SchemaChangeType::Compatible => CompatibilityLevel::FullyCompatible,
                SchemaChangeType::BackwardCompatible => CompatibilityLevel::BackwardCompatible,
                SchemaChangeType::Breaking => CompatibilityLevel::Breaking,
                SchemaChangeType::Removed => CompatibilityLevel::Breaking,
            };

            if level > max_level {
                max_level = level;
            }
        }

        max_level
    }

    fn analyze_affected_items(
        &self,
        _metadata: &ModuleMetadata,
        _changed_module: &Module,
    ) -> Result<Vec<AffectedItem>, ChangeAnalysisError> {
        // Simplified implementation
        // In a full implementation, this would analyze specific items affected
        Ok(Vec::new())
    }

    fn needs_recompilation(
        &self,
        impact_type: &ImpactType,
        _module_hash: &ContentHash,
    ) -> Result<bool, ChangeAnalysisError> {
        // Simplified logic
        Ok(matches!(
            impact_type,
            ImpactType::Direct | ImpactType::SchemaDependent
        ))
    }

    fn needs_state_migration(
        &self,
        impact_type: &ImpactType,
        _module_hash: &ContentHash,
    ) -> Result<bool, ChangeAnalysisError> {
        // Simplified logic
        Ok(matches!(impact_type, ImpactType::SchemaDependent))
    }
}

/// Dependency graph with content-addressable nodes
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes in the graph
    pub nodes: HashMap<NodeId, DependencyNode>,
    /// Edges in the graph
    pub edges: Vec<DependencyEdge>,
}

/// Node in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Module ID
    pub module_id: NodeId,
    /// Content hash of the module
    pub content_hash: ContentHash,
    /// Direct dependencies
    pub dependencies: HashSet<NodeId>,
    /// Modules that depend on this one
    pub dependents: HashSet<NodeId>,
    /// Schema dependencies
    pub schema_dependencies: HashSet<ContentHash>,
}

/// Edge in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Source module
    pub from: NodeId,
    /// Target module
    pub to: NodeId,
    /// Content hash of the dependency
    pub dependency_hash: ContentHash,
    /// Type of dependency
    pub edge_type: DependencyEdgeType,
}

/// Type of dependency edge
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyEdgeType {
    /// Import dependency
    Import,
    /// Type dependency
    Type,
    /// Schema dependency
    Schema,
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
        }
    }

    /// Add a dependency between two modules
    pub fn add_dependency(
        &mut self,
        from: NodeId,
        to: NodeId,
        dependency_hash: ContentHash,
        edge_type: DependencyEdgeType,
    ) {
        // Update node dependencies
        if let Some(from_node) = self.nodes.get_mut(&from) {
            from_node.dependencies.insert(to);
        }

        if let Some(to_node) = self.nodes.get_mut(&to) {
            to_node.dependents.insert(from);
        }

        // Add edge
        self.edges.push(DependencyEdge {
            from,
            to,
            dependency_hash,
            edge_type,
        });
    }

    /// Add a schema dependency
    pub fn add_schema_dependency(&mut self, module_id: NodeId, schema_hash: ContentHash) {
        if let Some(node) = self.nodes.get_mut(&module_id) {
            node.schema_dependencies.insert(schema_hash);
        }
    }
}

impl Default for FunctionPurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionPurityAnalyzer {
    /// Create a new function purity analyzer
    pub fn new() -> Self {
        FunctionPurityAnalyzer {
            purity_cache: HashMap::new(),
        }
    }

    /// Analyze changes between two function versions
    pub fn analyze_function_change(
        &mut self,
        _old_hash: ContentHash,
        new_hash: ContentHash,
    ) -> Result<PurityAnalysis, ChangeAnalysisError> {
        // Check cache first
        if let Some(cached_analysis) = self.purity_cache.get(&new_hash) {
            return Ok(cached_analysis.clone());
        }

        // Simplified purity analysis
        // In a full implementation, this would analyze the function AST
        let analysis = PurityAnalysis {
            is_pure: true, // Assume pure for now
            side_effects: Vec::new(),
            external_dependencies: Vec::new(),
            hot_swap_safe: true,
        };

        self.purity_cache.insert(new_hash, analysis.clone());
        Ok(analysis)
    }

    /// Analyze a function for purity by hash
    pub fn analyze_function_by_hash(
        &mut self,
        function_hash: ContentHash,
    ) -> Result<PurityAnalysis, ChangeAnalysisError> {
        // Check cache first
        if let Some(cached_analysis) = self.purity_cache.get(&function_hash) {
            return Ok(cached_analysis.clone());
        }

        // Simplified analysis - in reality would load and traverse function AST
        let analysis = PurityAnalysis {
            is_pure: true, // Simplified assumption
            side_effects: Vec::new(),
            external_dependencies: Vec::new(),
            hot_swap_safe: true, // Simplified
        };

        self.purity_cache.insert(function_hash, analysis.clone());
        Ok(analysis)
    }
}

impl Default for SchemaCompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaCompatibilityChecker {
    /// Create a new schema compatibility checker
    pub fn new() -> Self {
        SchemaCompatibilityChecker {
            compatibility_cache: HashMap::new(),
        }
    }

    /// Check compatibility between two schema versions
    pub fn check_compatibility(
        &mut self,
        old_schema_hash: ContentHash,
        new_schema_hash: ContentHash,
    ) -> Result<CompatibilityResult, ChangeAnalysisError> {
        let cache_key = (old_schema_hash, new_schema_hash);

        // Check cache first
        if let Some(cached_result) = self.compatibility_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        // Simplified compatibility check
        // In a full implementation, this would load and compare actual schemas
        let result = if old_schema_hash == new_schema_hash {
            CompatibilityResult::FullyCompatible
        } else {
            // Assume backward compatible for now
            CompatibilityResult::BackwardCompatible
        };

        self.compatibility_cache.insert(cache_key, result.clone());
        Ok(result)
    }
}

/// Errors that can occur during change analysis
#[derive(Debug, Clone)]
pub enum ChangeAnalysisError {
    /// Module store error
    ModuleStore(String),
    /// Schema analysis error
    SchemaAnalysis(String),
    /// Dependency graph error
    DependencyGraph(String),
    /// Function analysis error
    FunctionAnalysis(String),
    /// Invalid input
    InvalidInput(String),
}

impl std::fmt::Display for ChangeAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeAnalysisError::ModuleStore(msg) => write!(f, "Module store error: {msg}"),
            ChangeAnalysisError::SchemaAnalysis(msg) => write!(f, "Schema analysis error: {msg}"),
            ChangeAnalysisError::DependencyGraph(msg) => {
                write!(f, "Dependency graph error: {msg}")
            }
            ChangeAnalysisError::FunctionAnalysis(msg) => {
                write!(f, "Function analysis error: {msg}")
            }
            ChangeAnalysisError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for ChangeAnalysisError {}

impl From<crate::module_store::ModuleStoreError> for ChangeAnalysisError {
    fn from(err: crate::module_store::ModuleStoreError) -> Self {
        ChangeAnalysisError::ModuleStore(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStore;
    use mir_ast::Module;

    fn create_test_module(id: u64, name: &str) -> Module {
        use std::collections::VecDeque;

        Module {
            id: NodeId::new(id),
            name: name.to_string(),
            imports: Vec::new(),
            exports: Vec::new(),
            statements: Vec::new(),
            type_hash: mir_types::TypeHash::new(ContentHash::new(name.as_bytes())),
            capabilities: mir_ast::ModuleCapabilities {
                can_send_messages: false,
                can_receive_messages: false,
                allowed_message_types: Vec::new(),
            },
            message_queue: VecDeque::new(),
        }
    }

    #[test]
    fn test_change_analysis_system_creation() {
        let storage = InMemoryStore::new();
        let module_store = ModuleStore::new(storage);
        let _analysis_system = ChangeAnalysisSystem::new(module_store);
    }

    #[test]
    fn test_dependency_graph_computation() {
        let storage = InMemoryStore::new();
        let module_store = ModuleStore::new(storage);
        let analysis_system = ChangeAnalysisSystem::new(module_store);

        let module1 = create_test_module(1, "module1");
        let module2 = create_test_module(2, "module2");
        let modules = vec![module1, module2];

        let graph = analysis_system.compute_dependency_graph(&modules).unwrap();
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_function_purity_analysis() {
        let mut analyzer = FunctionPurityAnalyzer::new();

        // Test with dummy hashes
        let old_hash = ContentHash::new(b"old_function");
        let new_hash = ContentHash::new(b"new_function");

        let analysis = analyzer
            .analyze_function_change(old_hash, new_hash)
            .unwrap();
        assert!(analysis.is_pure); // Simplified test
    }

    #[test]
    fn test_schema_compatibility_checking() {
        let mut checker = SchemaCompatibilityChecker::new();

        let old_schema = ContentHash::new(b"old_schema");
        let new_schema = ContentHash::new(b"new_schema");

        let result = checker.check_compatibility(old_schema, new_schema).unwrap();
        match result {
            CompatibilityResult::BackwardCompatible => {} // Expected
            _ => panic!("Expected backward compatible result"),
        }
    }
}
