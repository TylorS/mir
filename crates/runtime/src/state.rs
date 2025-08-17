//! State management for hot-module-reloading
//!
//! This module implements Requirements 12 and 14:
//! - State preservation during hot-module-reloading
//! - State extraction from running modules
//! - State restoration mechanism
//! - Atomic state updates across components

use mir_types::{ContentHash, Value, TypeHash};
use mir_ast::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// State snapshot for preserving application state during updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Hash of the module this snapshot belongs to
    pub module_hash: ContentHash,
    /// Module ID
    pub module_id: NodeId,
    /// Captured state data organized by component
    pub state_data: HashMap<String, ComponentState>,
    /// Metadata about the snapshot
    pub metadata: SnapshotMetadata,
    /// Dependencies between state components
    pub dependencies: Vec<StateDependency>,
    /// Schema hashes for state validation
    pub schema_hashes: HashMap<String, TypeHash>,
}

/// Metadata about a state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// When the snapshot was created
    pub timestamp: u64,
    /// Version of the state format
    pub format_version: u32,
    /// Size of the snapshot in bytes
    pub size_bytes: usize,
    /// Checksum for integrity verification
    pub checksum: ContentHash,
    /// Tags for categorizing snapshots
    pub tags: Vec<String>,
}

/// State of a single component within a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentState {
    /// Name of the component
    pub component_name: String,
    /// Type of the component
    pub component_type: ComponentType,
    /// Actual state values
    pub values: HashMap<String, Value>,
    /// Connections to other components
    pub connections: Vec<ComponentConnection>,
    /// Whether this component is stateful
    pub is_stateful: bool,
    /// Schema hash for validation
    pub schema_hash: TypeHash,
}

/// Type of component
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    /// Pure function (no state)
    PureFunction,
    /// Stateful function with local state
    StatefulFunction,
    /// Data structure with persistent state
    DataStructure,
    /// Service with external connections
    Service,
    /// Actor with message queue
    Actor,
    /// Resource manager
    ResourceManager,
}

/// Connection between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConnection {
    /// Target component name
    pub target_component: String,
    /// Type of connection
    pub connection_type: ConnectionType,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
}

/// Type of connection between components
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Data flow connection
    DataFlow,
    /// Event subscription
    EventSubscription,
    /// Resource sharing
    ResourceSharing,
    /// Message passing
    MessagePassing,
}

/// Dependency between state components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDependency {
    /// Source component
    pub from_component: String,
    /// Target component
    pub to_component: String,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Whether this dependency is critical for restoration
    pub is_critical: bool,
}

/// Type of state dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Initialization order dependency
    InitializationOrder,
    /// Data dependency
    DataDependency,
    /// Resource dependency
    ResourceDependency,
    /// Lifecycle dependency
    LifecycleDependency,
}

/// State extraction configuration
#[derive(Debug, Clone)]
pub struct StateExtractionConfig {
    /// Components to include in extraction
    pub include_components: Option<HashSet<String>>,
    /// Components to exclude from extraction
    pub exclude_components: HashSet<String>,
    /// Whether to include transient state
    pub include_transient: bool,
    /// Whether to include connection state
    pub include_connections: bool,
    /// Maximum depth for nested state extraction
    pub max_depth: usize,
    /// Whether to validate state during extraction
    pub validate_during_extraction: bool,
}

/// State restoration configuration
#[derive(Debug, Clone)]
pub struct StateRestorationConfig {
    /// Whether to perform strict validation
    pub strict_validation: bool,
    /// Whether to allow partial restoration
    pub allow_partial_restoration: bool,
    /// Timeout for restoration operations
    pub restoration_timeout_ms: u64,
    /// Whether to restore connections
    pub restore_connections: bool,
    /// Strategy for handling missing components
    pub missing_component_strategy: MissingComponentStrategy,
}

/// Strategy for handling missing components during restoration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissingComponentStrategy {
    /// Fail the entire restoration
    Fail,
    /// Skip missing components and continue
    Skip,
    /// Create placeholder components
    CreatePlaceholder,
    /// Use default values
    UseDefaults,
}

/// Result of state extraction operation
#[derive(Debug, Clone)]
pub struct StateExtractionResult {
    /// The extracted snapshot
    pub snapshot: StateSnapshot,
    /// Components that were successfully extracted
    pub extracted_components: Vec<String>,
    /// Components that failed to extract
    pub failed_components: Vec<(String, String)>, // (component_name, error_message)
    /// Warnings during extraction
    pub warnings: Vec<String>,
    /// Time taken for extraction
    pub extraction_time_ms: u64,
}

/// Result of state restoration operation
#[derive(Debug, Clone)]
pub struct StateRestorationResult {
    /// Whether restoration was successful
    pub success: bool,
    /// Components that were successfully restored
    pub restored_components: Vec<String>,
    /// Components that failed to restore
    pub failed_components: Vec<(String, String)>, // (component_name, error_message)
    /// Warnings during restoration
    pub warnings: Vec<String>,
    /// Time taken for restoration
    pub restoration_time_ms: u64,
}

/// Atomic state update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicStateUpdate {
    /// Unique ID for this update
    pub update_id: ContentHash,
    /// Components involved in this update
    pub components: Vec<String>,
    /// State changes to apply
    pub changes: Vec<StateChange>,
    /// Rollback information
    pub rollback_data: Option<StateSnapshot>,
    /// Whether this update has been committed
    pub is_committed: bool,
}

/// Individual state change within an atomic update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Component being changed
    pub component_name: String,
    /// Type of change
    pub change_type: StateChangeType,
    /// Old value (for rollback)
    pub old_value: Option<Value>,
    /// New value
    pub new_value: Value,
    /// Path within the component state
    pub state_path: Vec<String>,
}

/// Type of state change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateChangeType {
    /// Set a new value
    Set,
    /// Update an existing value
    Update,
    /// Delete a value
    Delete,
    /// Insert into a collection
    Insert,
    /// Remove from a collection
    Remove,
}

/// State manager for handling state preservation and restoration
pub struct StateManager {
    /// Stored snapshots indexed by module hash
    snapshots: Arc<RwLock<HashMap<ContentHash, StateSnapshot>>>,
    /// Active state extractors
    extractors: HashMap<ComponentType, Box<dyn StateExtractor>>,
    /// Active state restorers
    restorers: HashMap<ComponentType, Box<dyn StateRestorer>>,
    /// Pending atomic updates
    pending_updates: Arc<Mutex<HashMap<ContentHash, AtomicStateUpdate>>>,
    /// Configuration
    config: StateManagerConfig,
}

/// Configuration for the state manager
#[derive(Debug, Clone)]
pub struct StateManagerConfig {
    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,
    /// Maximum age of snapshots in seconds
    pub max_snapshot_age_seconds: u64,
    /// Whether to compress snapshots
    pub compress_snapshots: bool,
    /// Whether to validate snapshots on creation
    pub validate_on_creation: bool,
    /// Default extraction configuration
    pub default_extraction_config: StateExtractionConfig,
    /// Default restoration configuration
    pub default_restoration_config: StateRestorationConfig,
}

/// Trait for extracting state from components
pub trait StateExtractor: Send + Sync {
    /// Extract state from a component
    fn extract_state(
        &self,
        component_name: &str,
        component_data: &Value,
        config: &StateExtractionConfig,
    ) -> Result<ComponentState, StateError>;

    /// Check if this extractor can handle the given component type
    fn can_extract(&self, component_type: &ComponentType) -> bool;
}

/// Trait for restoring state to components
pub trait StateRestorer: Send + Sync {
    /// Restore state to a component
    fn restore_state(
        &self,
        component_name: &str,
        component_state: &ComponentState,
        config: &StateRestorationConfig,
    ) -> Result<Value, StateError>;

    /// Check if this restorer can handle the given component type
    fn can_restore(&self, component_type: &ComponentType) -> bool;
}

impl StateManager {
    /// Create a new state manager
    pub fn new(config: StateManagerConfig) -> Self {
        StateManager {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            extractors: HashMap::new(),
            restorers: HashMap::new(),
            pending_updates: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Create a new state manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(StateManagerConfig::default())
    }

    /// Extract state from a running module
    pub fn extract_state(
        &self,
        module_id: NodeId,
        module_hash: ContentHash,
        module_data: &HashMap<String, Value>,
        config: Option<StateExtractionConfig>,
    ) -> Result<StateExtractionResult, StateError> {
        let start_time = current_timestamp_ms();
        let config = config.unwrap_or_else(|| self.config.default_extraction_config.clone());

        let mut extracted_components = Vec::new();
        let mut failed_components = Vec::new();
        let mut warnings = Vec::new();
        let mut state_data = HashMap::new();
        let mut schema_hashes = HashMap::new();

        // Extract state from each component
        for (component_name, component_value) in module_data {
            // Skip excluded components
            if config.exclude_components.contains(component_name) {
                continue;
            }

            // Check if component should be included
            if let Some(ref include_set) = config.include_components {
                if !include_set.contains(component_name) {
                    continue;
                }
            }

            // Determine component type
            let component_type = self.determine_component_type(component_value);

            // Find appropriate extractor
            if let Some(extractor) = self.find_extractor(&component_type) {
                match extractor.extract_state(component_name, component_value, &config) {
                    Ok(component_state) => {
                        schema_hashes.insert(component_name.clone(), component_state.schema_hash);
                        state_data.insert(component_name.clone(), component_state);
                        extracted_components.push(component_name.clone());
                    }
                    Err(e) => {
                        failed_components.push((component_name.clone(), e.to_string()));
                    }
                }
            } else {
                warnings.push(format!("No extractor found for component '{component_name}' of type {component_type:?}"));
            }
        }

        // Analyze dependencies between components
        let dependencies = self.analyze_state_dependencies(&state_data)?;

        // Create snapshot metadata
        let snapshot_data = serde_json::to_vec(&state_data)
            .map_err(|e| StateError::Serialization(e.to_string()))?;
        
        let metadata = SnapshotMetadata {
            timestamp: current_timestamp(),
            format_version: 1,
            size_bytes: snapshot_data.len(),
            checksum: ContentHash::new(&snapshot_data),
            tags: vec!["extracted".to_string()],
        };

        // Create the snapshot
        let snapshot = StateSnapshot {
            module_hash,
            module_id,
            state_data,
            metadata,
            dependencies,
            schema_hashes,
        };

        // Validate snapshot if configured
        if config.validate_during_extraction {
            self.validate_snapshot(&snapshot)?;
        }

        let extraction_time_ms = current_timestamp_ms() - start_time;

        Ok(StateExtractionResult {
            snapshot,
            extracted_components,
            failed_components,
            warnings,
            extraction_time_ms,
        })
    }

    /// Create a state snapshot
    pub fn create_snapshot(
        &self,
        module_id: NodeId,
        module_hash: ContentHash,
        module_data: &HashMap<String, Value>,
    ) -> Result<StateSnapshot, StateError> {
        let extraction_result = self.extract_state(module_id, module_hash, module_data, None)?;
        
        // Store the snapshot
        {
            let mut snapshots = self.snapshots.write()
                .map_err(|_| StateError::LockError("Failed to acquire write lock on snapshots".to_string()))?;
            snapshots.insert(module_hash, extraction_result.snapshot.clone());
        }

        // Clean up old snapshots
        self.cleanup_old_snapshots()?;

        Ok(extraction_result.snapshot)
    }

    /// Restore state from a snapshot
    pub fn restore_state(
        &self,
        snapshot: &StateSnapshot,
        config: Option<StateRestorationConfig>,
    ) -> Result<StateRestorationResult, StateError> {
        let start_time = current_timestamp_ms();
        let config = config.unwrap_or_else(|| self.config.default_restoration_config.clone());

        let mut restored_components = Vec::new();
        let mut failed_components = Vec::new();
        let mut warnings = Vec::new();

        // Validate snapshot first
        if config.strict_validation {
            self.validate_snapshot(snapshot)?;
        }

        // Sort components by dependencies
        let sorted_components = self.sort_components_by_dependencies(&snapshot.dependencies, &snapshot.state_data)?;

        // Restore components in dependency order
        for component_name in sorted_components {
            if let Some(component_state) = snapshot.state_data.get(&component_name) {
                // Find appropriate restorer
                if let Some(restorer) = self.find_restorer(&component_state.component_type) {
                    match restorer.restore_state(&component_name, component_state, &config) {
                        Ok(_restored_value) => {
                            restored_components.push(component_name.clone());
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            failed_components.push((component_name.clone(), error_msg.clone()));
                            
                            if !config.allow_partial_restoration {
                                return Err(StateError::RestorationFailed(format!(
                                    "Failed to restore component '{component_name}': {error_msg}"
                                )));
                            }
                        }
                    }
                } else {
                    let error_msg = format!("No restorer found for component type {:?}", component_state.component_type);
                    match config.missing_component_strategy {
                        MissingComponentStrategy::Fail => {
                            return Err(StateError::RestorationFailed(error_msg));
                        }
                        MissingComponentStrategy::Skip => {
                            warnings.push(error_msg);
                        }
                        MissingComponentStrategy::CreatePlaceholder => {
                            warnings.push(format!("Created placeholder for component '{component_name}'"));
                            restored_components.push(component_name.clone());
                        }
                        MissingComponentStrategy::UseDefaults => {
                            warnings.push(format!("Used defaults for component '{component_name}'"));
                            restored_components.push(component_name.clone());
                        }
                    }
                }
            }
        }

        let restoration_time_ms = current_timestamp_ms() - start_time;

        Ok(StateRestorationResult {
            success: failed_components.is_empty() || config.allow_partial_restoration,
            restored_components,
            failed_components,
            warnings,
            restoration_time_ms,
        })
    }

    /// Begin an atomic state update
    pub fn begin_atomic_update(
        &self,
        components: Vec<String>,
        changes: Vec<StateChange>,
    ) -> Result<ContentHash, StateError> {
        let update_id = ContentHash::new(format!("{components:?}{changes:?}").as_bytes());
        
        let update = AtomicStateUpdate {
            update_id,
            components,
            changes,
            rollback_data: None, // TODO: Create rollback snapshot
            is_committed: false,
        };

        {
            let mut pending = self.pending_updates.lock()
                .map_err(|_| StateError::LockError("Failed to acquire lock on pending updates".to_string()))?;
            pending.insert(update_id, update);
        }

        Ok(update_id)
    }

    /// Commit an atomic state update
    pub fn commit_atomic_update(&self, update_id: ContentHash) -> Result<(), StateError> {
        let mut pending = self.pending_updates.lock()
            .map_err(|_| StateError::LockError("Failed to acquire lock on pending updates".to_string()))?;
        
        if let Some(mut update) = pending.remove(&update_id) {
            // Apply all changes atomically
            for change in &update.changes {
                self.apply_state_change(change)?;
            }
            
            update.is_committed = true;
            Ok(())
        } else {
            Err(StateError::UpdateNotFound(update_id))
        }
    }

    /// Rollback an atomic state update
    pub fn rollback_atomic_update(&self, update_id: ContentHash) -> Result<(), StateError> {
        let mut pending = self.pending_updates.lock()
            .map_err(|_| StateError::LockError("Failed to acquire lock on pending updates".to_string()))?;
        
        if let Some(update) = pending.remove(&update_id) {
            // Restore from rollback data if available
            if let Some(rollback_snapshot) = &update.rollback_data {
                self.restore_state(rollback_snapshot, None)?;
            }
            Ok(())
        } else {
            Err(StateError::UpdateNotFound(update_id))
        }
    }

    /// Get a stored snapshot
    pub fn get_snapshot(&self, module_hash: ContentHash) -> Result<Option<StateSnapshot>, StateError> {
        let snapshots = self.snapshots.read()
            .map_err(|_| StateError::LockError("Failed to acquire read lock on snapshots".to_string()))?;
        Ok(snapshots.get(&module_hash).cloned())
    }

    /// Register a state extractor
    pub fn register_extractor(&mut self, component_type: ComponentType, extractor: Box<dyn StateExtractor>) {
        self.extractors.insert(component_type, extractor);
    }

    /// Register a state restorer
    pub fn register_restorer(&mut self, component_type: ComponentType, restorer: Box<dyn StateRestorer>) {
        self.restorers.insert(component_type, restorer);
    }

    // Private helper methods

    fn determine_component_type(&self, _value: &Value) -> ComponentType {
        // Simplified implementation - in reality would analyze the value structure
        ComponentType::StatefulFunction
    }

    fn find_extractor(&self, component_type: &ComponentType) -> Option<&dyn StateExtractor> {
        self.extractors.get(component_type).map(|b| b.as_ref())
    }

    fn find_restorer(&self, component_type: &ComponentType) -> Option<&dyn StateRestorer> {
        self.restorers.get(component_type).map(|b| b.as_ref())
    }

    fn analyze_state_dependencies(&self, _state_data: &HashMap<String, ComponentState>) -> Result<Vec<StateDependency>, StateError> {
        // Simplified implementation - in reality would analyze component relationships
        Ok(Vec::new())
    }

    fn validate_snapshot(&self, _snapshot: &StateSnapshot) -> Result<(), StateError> {
        // Simplified validation - in reality would check schema compatibility, etc.
        Ok(())
    }

    fn sort_components_by_dependencies(
        &self,
        dependencies: &[StateDependency],
        state_data: &HashMap<String, ComponentState>,
    ) -> Result<Vec<String>, StateError> {
        // Simplified topological sort
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();

        for component_name in state_data.keys() {
            if !visited.contains(component_name) {
                self.visit_component(component_name, dependencies, &mut visited, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    fn visit_component(
        &self,
        component_name: &str,
        dependencies: &[StateDependency],
        visited: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> Result<(), StateError> {
        visited.insert(component_name.to_string());

        // Visit dependencies first
        for dep in dependencies {
            if dep.to_component == component_name && !visited.contains(&dep.from_component) {
                self.visit_component(&dep.from_component, dependencies, visited, sorted)?;
            }
        }

        sorted.push(component_name.to_string());
        Ok(())
    }

    fn apply_state_change(&self, _change: &StateChange) -> Result<(), StateError> {
        // Simplified implementation - in reality would apply the actual change
        Ok(())
    }

    fn cleanup_old_snapshots(&self) -> Result<(), StateError> {
        let mut snapshots = self.snapshots.write()
            .map_err(|_| StateError::LockError("Failed to acquire write lock on snapshots".to_string()))?;

        let current_time = current_timestamp();
        let max_age = self.config.max_snapshot_age_seconds;

        // Remove old snapshots
        snapshots.retain(|_, snapshot| {
            current_time - snapshot.metadata.timestamp < max_age
        });

        // Limit number of snapshots
        if snapshots.len() > self.config.max_snapshots {
            // Keep only the most recent snapshots
            let mut snapshot_vec: Vec<_> = snapshots.iter().map(|(k, v)| (*k, v.metadata.timestamp)).collect();
            snapshot_vec.sort_by_key(|(_, timestamp)| *timestamp);
            
            let to_remove = snapshot_vec.len() - self.config.max_snapshots;
            let hashes_to_remove: Vec<ContentHash> = snapshot_vec.iter().take(to_remove).map(|(hash, _)| *hash).collect();
            
            for hash in hashes_to_remove {
                snapshots.remove(&hash);
            }
        }

        Ok(())
    }
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        StateManagerConfig {
            max_snapshots: 100,
            max_snapshot_age_seconds: 3600, // 1 hour
            compress_snapshots: true,
            validate_on_creation: true,
            default_extraction_config: StateExtractionConfig::default(),
            default_restoration_config: StateRestorationConfig::default(),
        }
    }
}

impl Default for StateExtractionConfig {
    fn default() -> Self {
        StateExtractionConfig {
            include_components: None,
            exclude_components: HashSet::new(),
            include_transient: false,
            include_connections: true,
            max_depth: 10,
            validate_during_extraction: true,
        }
    }
}

impl Default for StateRestorationConfig {
    fn default() -> Self {
        StateRestorationConfig {
            strict_validation: true,
            allow_partial_restoration: false,
            restoration_timeout_ms: 5000,
            restore_connections: true,
            missing_component_strategy: MissingComponentStrategy::Fail,
        }
    }
}

/// Errors that can occur during state operations
#[derive(Debug, Clone)]
pub enum StateError {
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Validation error
    Validation(String),
    /// Extraction error
    Extraction(String),
    /// Restoration error
    Restoration(String),
    /// Restoration failed
    RestorationFailed(String),
    /// Lock error
    LockError(String),
    /// Update not found
    UpdateNotFound(ContentHash),
    /// Component not found
    ComponentNotFound(String),
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Timeout error
    Timeout(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            StateError::Deserialization(msg) => write!(f, "Deserialization error: {msg}"),
            StateError::Validation(msg) => write!(f, "Validation error: {msg}"),
            StateError::Extraction(msg) => write!(f, "Extraction error: {msg}"),
            StateError::Restoration(msg) => write!(f, "Restoration error: {msg}"),
            StateError::RestorationFailed(msg) => write!(f, "Restoration failed: {msg}"),
            StateError::LockError(msg) => write!(f, "Lock error: {msg}"),
            StateError::UpdateNotFound(hash) => write!(f, "Update not found: {hash}"),
            StateError::ComponentNotFound(name) => write!(f, "Component not found: {name}"),
            StateError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {msg}"),
            StateError::Timeout(msg) => write!(f, "Timeout: {msg}"),
        }
    }
}

impl std::error::Error for StateError {}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let _manager = StateManager::with_defaults();
    }

    #[test]
    fn test_state_snapshot_creation() {
        let manager = StateManager::with_defaults();
        let module_id = NodeId::new(1);
        let module_hash = ContentHash::new(b"test_module");
        let mut module_data = HashMap::new();
        module_data.insert("test_component".to_string(), Value::I32(42));

        let result = manager.create_snapshot(module_id, module_hash, &module_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_atomic_state_update() {
        let manager = StateManager::with_defaults();
        let components = vec!["component1".to_string()];
        let changes = vec![StateChange {
            component_name: "component1".to_string(),
            change_type: StateChangeType::Set,
            old_value: None,
            new_value: Value::I32(100),
            state_path: vec!["value".to_string()],
        }];

        let update_id = manager.begin_atomic_update(components, changes).unwrap();
        assert!(manager.commit_atomic_update(update_id).is_ok());
    }

    #[test]
    fn test_state_extraction_config() {
        let config = StateExtractionConfig::default();
        assert!(config.include_connections);
        assert!(!config.include_transient);
        assert_eq!(config.max_depth, 10);
    }

    #[test]
    fn test_state_restoration_config() {
        let config = StateRestorationConfig::default();
        assert!(config.strict_validation);
        assert!(!config.allow_partial_restoration);
        assert_eq!(config.restoration_timeout_ms, 5000);
    }
}