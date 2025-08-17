//! State migration system for hot-module-reloading
//!
//! This module implements Requirement 14:
//! - Automatic state migration for compatible changes
//! - User-defined migration function execution
//! - Migration failure handling and rollback
//! - Migration chain execution for multi-step updates

use mir_types::{ContentHash, Value, TypeHash};
use mir_ast::NodeId;
use crate::state::ComponentState;
use crate::change_analysis::SchemaChange;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// State migration system for handling schema evolution
pub struct StateMigrationSystem {
    /// Migration function registry
    migration_registry: Arc<Mutex<MigrationRegistry>>,
    /// Migration execution engine
    execution_engine: MigrationExecutionEngine,
    /// Migration history
    migration_history: Arc<Mutex<VecDeque<MigrationRecord>>>,
    /// Configuration
    config: MigrationConfig,
}

/// Registry of migration functions
pub struct MigrationRegistry {
    /// Migration functions indexed by schema transition
    migrations: HashMap<SchemaTransition, Box<dyn MigrationFunction>>,
    /// Automatic migration generators
    auto_generators: Vec<Box<dyn AutoMigrationGenerator>>,
    /// Migration chains for multi-step migrations
    #[allow(dead_code)]
    migration_chains: HashMap<SchemaTransition, MigrationChain>,
}

/// Key for identifying a schema transition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaTransition {
    /// Source schema hash
    pub from_schema: TypeHash,
    /// Target schema hash
    pub to_schema: TypeHash,
    /// Component type being migrated
    pub component_type: String,
}

/// Configuration for state migration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Timeout for migration operations
    pub migration_timeout: Duration,
    /// Whether to allow automatic migrations
    pub enable_auto_migration: bool,
    /// Whether to validate migrated state
    pub validate_migrated_state: bool,
    /// Maximum number of migration steps in a chain
    pub max_migration_chain_length: usize,
    /// Whether to preserve original state during migration
    pub preserve_original_state: bool,
    /// Maximum number of migration records to keep
    pub max_migration_history: usize,
}

/// Trait for migration functions
pub trait MigrationFunction: Send + Sync {
    /// Name of this migration function
    fn name(&self) -> &str;
    
    /// Migrate state from old schema to new schema
    fn migrate(
        &self,
        old_state: &ComponentState,
        old_schema: TypeHash,
        new_schema: TypeHash,
        context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError>;
    
    /// Check if this migration can handle the given transition
    fn can_migrate(&self, transition: &SchemaTransition) -> bool;
    
    /// Get the estimated time for this migration
    fn estimated_duration(&self, state_size: usize) -> Duration;
    
    /// Whether this migration is reversible
    fn is_reversible(&self) -> bool;
}

/// Trait for automatic migration generators
pub trait AutoMigrationGenerator: Send + Sync {
    /// Name of this generator
    fn name(&self) -> &str;
    
    /// Generate a migration function for the given transition
    fn generate_migration(
        &self,
        transition: &SchemaTransition,
        schema_change: &SchemaChange,
    ) -> Result<Box<dyn MigrationFunction>, MigrationError>;
    
    /// Check if this generator can handle the given change
    fn can_generate(&self, schema_change: &SchemaChange) -> bool;
}

/// Context provided to migration functions
#[derive(Debug, Clone)]
pub struct MigrationContext {
    /// Module ID being migrated
    pub module_id: NodeId,
    /// Migration ID for tracking
    pub migration_id: ContentHash,
    /// Additional context data
    pub context_data: HashMap<String, Value>,
    /// Migration configuration
    pub config: MigrationConfig,
    /// Timestamp when migration started
    pub started_at: u64,
}

/// Chain of migrations for multi-step updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationChain {
    /// Chain ID
    pub chain_id: ContentHash,
    /// Steps in the migration chain
    pub steps: Vec<MigrationStep>,
    /// Whether the entire chain is reversible
    pub is_reversible: bool,
    /// Estimated total duration
    pub estimated_duration: Duration,
}

/// Individual step in a migration chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Step ID
    pub step_id: String,
    /// Schema transition for this step
    pub transition: SchemaTransition,
    /// Migration function name
    pub migration_function: String,
    /// Whether this step is reversible
    pub is_reversible: bool,
    /// Estimated duration
    pub estimated_duration: Duration,
    /// Status of this step
    pub status: MigrationStepStatus,
}

/// Status of a migration step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStepStatus {
    /// Step is pending
    Pending,
    /// Step is running
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed(String),
    /// Step was skipped
    Skipped,
    /// Step was rolled back
    RolledBack,
}

/// Result of a migration operation
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Migration ID
    pub migration_id: ContentHash,
    /// Whether migration was successful
    pub success: bool,
    /// Migrated state
    pub migrated_state: Option<ComponentState>,
    /// Steps that were executed
    pub executed_steps: Vec<String>,
    /// Steps that failed
    pub failed_steps: Vec<(String, String)>,
    /// Time taken for migration
    pub execution_time: Duration,
    /// Warnings generated during migration
    pub warnings: Vec<String>,
    /// Whether rollback was performed
    pub was_rolled_back: bool,
}

/// Record of a migration for history tracking
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    /// Migration result
    pub result: MigrationResult,
    /// Original state snapshot
    pub original_state: ComponentState,
    /// Schema transition
    pub transition: SchemaTransition,
    /// Migration context
    pub context: MigrationContext,
    /// Timestamp when recorded
    pub recorded_at: u64,
}

/// Migration execution engine
pub struct MigrationExecutionEngine {
    /// Active migrations
    active_migrations: Arc<Mutex<HashMap<ContentHash, MigrationExecution>>>,
    /// Configuration
    #[allow(dead_code)]
    config: MigrationConfig,
}

/// Execution state of a migration
#[derive(Debug, Clone)]
pub struct MigrationExecution {
    /// Migration context
    pub context: MigrationContext,
    /// Migration chain being executed
    pub chain: Option<MigrationChain>,
    /// Current step being executed
    pub current_step: usize,
    /// Start time
    pub started_at: Instant,
    /// Status
    pub status: MigrationExecutionStatus,
    /// Intermediate states for rollback
    pub intermediate_states: Vec<ComponentState>,
}

/// Status of migration execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationExecutionStatus {
    /// Migration is running
    Running,
    /// Migration completed successfully
    Completed,
    /// Migration failed
    Failed(String),
    /// Migration was cancelled
    Cancelled,
    /// Migration was rolled back
    RolledBack,
}

impl StateMigrationSystem {
    /// Create a new state migration system
    pub fn new(config: MigrationConfig) -> Self {
        StateMigrationSystem {
            migration_registry: Arc::new(Mutex::new(MigrationRegistry::new())),
            execution_engine: MigrationExecutionEngine::new(config.clone()),
            migration_history: Arc::new(Mutex::new(VecDeque::new())),
            config,
        }
    }

    /// Register a migration function
    pub fn register_migration(
        &mut self,
        transition: SchemaTransition,
        migration: Box<dyn MigrationFunction>,
    ) -> Result<(), MigrationError> {
        let mut registry = self.migration_registry.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
        
        registry.migrations.insert(transition, migration);
        Ok(())
    }

    /// Register an automatic migration generator
    pub fn register_auto_generator(
        &mut self,
        generator: Box<dyn AutoMigrationGenerator>,
    ) -> Result<(), MigrationError> {
        let mut registry = self.migration_registry.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
        
        registry.auto_generators.push(generator);
        Ok(())
    }

    /// Migrate state for compatible changes
    pub fn migrate_state_automatic(
        &mut self,
        old_state: &ComponentState,
        schema_change: &SchemaChange,
        context: MigrationContext,
    ) -> Result<MigrationResult, MigrationError> {
        if !self.config.enable_auto_migration {
            return Err(MigrationError::AutoMigrationDisabled);
        }

        let transition = SchemaTransition {
            from_schema: old_state.schema_hash,
            to_schema: TypeHash::new(schema_change.new_version), // Convert ContentHash to TypeHash
            component_type: old_state.component_name.clone(),
        };

        // Try to find existing migration function
        {
            let registry = self.migration_registry.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
            
            if let Some(migration_fn) = registry.migrations.get(&transition) {
                return self.execute_single_migration(old_state, migration_fn.as_ref(), &transition, context);
            }
        }

        // Try to generate automatic migration
        let migration_fn = self.generate_automatic_migration(&transition, schema_change)?;
        
        // Execute the migration first
        let result = self.execute_single_migration(old_state, migration_fn.as_ref(), &transition, context);
        
        // Register the generated migration for future use if successful
        if result.as_ref().map(|r| r.success).unwrap_or(false) {
            let mut registry = self.migration_registry.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
            registry.migrations.insert(transition.clone(), migration_fn);
        }

        result
    }

    /// Execute user-defined migration function
    pub fn migrate_state_user_defined(
        &mut self,
        old_state: &ComponentState,
        transition: &SchemaTransition,
        context: MigrationContext,
    ) -> Result<MigrationResult, MigrationError> {
        let registry = self.migration_registry.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
        
        let migration_fn = registry.migrations.get(transition)
            .ok_or_else(|| MigrationError::MigrationNotFound(transition.clone()))?;

        self.execute_single_migration(old_state, migration_fn.as_ref(), transition, context)
    }

    /// Execute migration chain for multi-step updates
    pub fn execute_migration_chain(
        &mut self,
        old_state: &ComponentState,
        chain: &MigrationChain,
        context: MigrationContext,
    ) -> Result<MigrationResult, MigrationError> {
        let migration_id = context.migration_id;
        let start_time = Instant::now();

        // Create migration execution
        let execution = MigrationExecution {
            context: context.clone(),
            chain: Some(chain.clone()),
            current_step: 0,
            started_at: start_time,
            status: MigrationExecutionStatus::Running,
            intermediate_states: vec![old_state.clone()],
        };

        // Mark migration as active
        {
            let mut active_migrations = self.execution_engine.active_migrations.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
            active_migrations.insert(migration_id, execution);
        }

        let mut current_state = old_state.clone();
        let mut executed_steps = Vec::new();
        let mut failed_steps = Vec::new();
        let mut warnings = Vec::new();
        let mut was_rolled_back = false;

        // Execute each step in the chain
        for (i, step) in chain.steps.iter().enumerate() {
            // Update current step
            {
                let mut active_migrations = self.execution_engine.active_migrations.lock()
                    .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
                
                if let Some(execution) = active_migrations.get_mut(&migration_id) {
                    execution.current_step = i;
                }
            }

            // Find migration function for this step
            let registry = self.migration_registry.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
            
            let migration_fn = registry.migrations.get(&step.transition)
                .ok_or_else(|| MigrationError::MigrationNotFound(step.transition.clone()))?;

            // Execute the migration step
            match migration_fn.migrate(&current_state, step.transition.from_schema, step.transition.to_schema, &context) {
                Ok(migrated_state) => {
                    // Store intermediate state for potential rollback
                    {
                        let mut active_migrations = self.execution_engine.active_migrations.lock()
                            .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
                        
                        if let Some(execution) = active_migrations.get_mut(&migration_id) {
                            execution.intermediate_states.push(migrated_state.clone());
                        }
                    }

                    current_state = migrated_state;
                    executed_steps.push(step.step_id.clone());
                }
                Err(e) => {
                    failed_steps.push((step.step_id.clone(), e.to_string()));
                    
                    // Attempt rollback if possible
                    if step.is_reversible && chain.is_reversible {
                        drop(registry); // Release the lock before calling rollback
                        match self.rollback_migration_chain(migration_id, i) {
                            Ok(_) => {
                                was_rolled_back = true;
                                warnings.push("Migration chain rolled back due to step failure".to_string());
                            }
                            Err(rollback_err) => {
                                warnings.push(format!("Rollback failed: {rollback_err}"));
                            }
                        }
                    }
                    
                    break;
                }
            }
        }

        // Update execution status
        {
            let mut active_migrations = self.execution_engine.active_migrations.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
            
            if let Some(execution) = active_migrations.get_mut(&migration_id) {
                execution.status = if failed_steps.is_empty() {
                    MigrationExecutionStatus::Completed
                } else if was_rolled_back {
                    MigrationExecutionStatus::RolledBack
                } else {
                    MigrationExecutionStatus::Failed(format!("Failed steps: {}", 
                        failed_steps.iter().map(|(id, _)| id).cloned().collect::<Vec<_>>().join(", ")))
                };
            }
        }

        let execution_time = start_time.elapsed();
        let success = failed_steps.is_empty();

        let result = MigrationResult {
            migration_id,
            success,
            migrated_state: if success { Some(current_state) } else { None },
            executed_steps,
            failed_steps,
            execution_time,
            warnings,
            was_rolled_back,
        };

        // Record the migration
        self.record_migration(result.clone(), old_state.clone(), &chain.steps[0].transition, context)?;

        // Remove from active migrations
        {
            let mut active_migrations = self.execution_engine.active_migrations.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
            active_migrations.remove(&migration_id);
        }

        Ok(result)
    }

    /// Handle migration failure and rollback
    pub fn handle_migration_failure(
        &mut self,
        migration_id: ContentHash,
        error: &MigrationError,
    ) -> Result<(), MigrationError> {
        let mut active_migrations = self.execution_engine.active_migrations.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
        
        if let Some(execution) = active_migrations.get_mut(&migration_id) {
            execution.status = MigrationExecutionStatus::Failed(error.to_string());
            
            // Attempt rollback if chain is reversible
            if let Some(chain) = &execution.chain {
                if chain.is_reversible {
                    let current_step = execution.current_step;
                    drop(active_migrations); // Release lock before calling rollback
                    return self.rollback_migration_chain(migration_id, current_step);
                }
            }
        }

        Ok(())
    }

    /// Create migration chain for multi-step updates
    pub fn create_migration_chain(
        &self,
        schema_changes: &[SchemaChange],
        component_type: &str,
    ) -> Result<MigrationChain, MigrationError> {
        if schema_changes.len() > self.config.max_migration_chain_length {
            return Err(MigrationError::ChainTooLong(schema_changes.len()));
        }

        let chain_id = ContentHash::new(format!("{:?}_{}", schema_changes, current_timestamp()).as_bytes());
        let mut steps = Vec::new();
        let mut total_duration = Duration::from_millis(0);
        let mut is_reversible = true;

        for (i, change) in schema_changes.iter().enumerate() {
            let transition = SchemaTransition {
                from_schema: if i == 0 {
                    TypeHash::new(change.old_version.unwrap_or(ContentHash::zero()))
                } else {
                    TypeHash::new(schema_changes[i - 1].new_version)
                },
                to_schema: TypeHash::new(change.new_version),
                component_type: component_type.to_string(),
            };

            // Find migration function for this transition
            let registry = self.migration_registry.lock()
                .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
            
            let migration_fn = registry.migrations.get(&transition)
                .ok_or_else(|| MigrationError::MigrationNotFound(transition.clone()))?;

            let step_duration = migration_fn.estimated_duration(1000); // Estimate based on 1KB state
            total_duration += step_duration;

            if !migration_fn.is_reversible() {
                is_reversible = false;
            }

            steps.push(MigrationStep {
                step_id: format!("step_{i}"),
                transition,
                migration_function: migration_fn.name().to_string(),
                is_reversible: migration_fn.is_reversible(),
                estimated_duration: step_duration,
                status: MigrationStepStatus::Pending,
            });
        }

        Ok(MigrationChain {
            chain_id,
            steps,
            is_reversible,
            estimated_duration: total_duration,
        })
    }

    /// Get migration history
    pub fn get_migration_history(&self, limit: Option<usize>) -> Result<Vec<MigrationRecord>, MigrationError> {
        let history = self.migration_history.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire migration history lock".to_string()))?;
        
        let limit = limit.unwrap_or(history.len());
        Ok(history.iter().take(limit).cloned().collect())
    }

    // Private helper methods

    fn execute_single_migration(
        &self,
        old_state: &ComponentState,
        migration_fn: &dyn MigrationFunction,
        transition: &SchemaTransition,
        context: MigrationContext,
    ) -> Result<MigrationResult, MigrationError> {
        let start_time = Instant::now();
        let migration_id = context.migration_id;

        match migration_fn.migrate(old_state, transition.from_schema, transition.to_schema, &context) {
            Ok(migrated_state) => {
                let execution_time = start_time.elapsed();
                
                let result = MigrationResult {
                    migration_id,
                    success: true,
                    migrated_state: Some(migrated_state),
                    executed_steps: vec![migration_fn.name().to_string()],
                    failed_steps: Vec::new(),
                    execution_time,
                    warnings: Vec::new(),
                    was_rolled_back: false,
                };

                Ok(result)
            }
            Err(e) => {
                let execution_time = start_time.elapsed();
                
                let result = MigrationResult {
                    migration_id,
                    success: false,
                    migrated_state: None,
                    executed_steps: Vec::new(),
                    failed_steps: vec![(migration_fn.name().to_string(), e.to_string())],
                    execution_time,
                    warnings: Vec::new(),
                    was_rolled_back: false,
                };

                Ok(result)
            }
        }
    }

    fn generate_automatic_migration(
        &self,
        transition: &SchemaTransition,
        schema_change: &SchemaChange,
    ) -> Result<Box<dyn MigrationFunction>, MigrationError> {
        let registry = self.migration_registry.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire migration registry lock".to_string()))?;
        
        for generator in &registry.auto_generators {
            if generator.can_generate(schema_change) {
                return generator.generate_migration(transition, schema_change);
            }
        }

        Err(MigrationError::NoAutoMigrationAvailable(transition.clone()))
    }

    fn rollback_migration_chain(
        &mut self,
        migration_id: ContentHash,
        failed_step: usize,
    ) -> Result<(), MigrationError> {
        let mut active_migrations = self.execution_engine.active_migrations.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire active migrations lock".to_string()))?;
        
        if let Some(execution) = active_migrations.get_mut(&migration_id) {
            // Restore to the state before the failed step
            if failed_step < execution.intermediate_states.len() {
                execution.status = MigrationExecutionStatus::RolledBack;
                // In a real implementation, we would restore the actual state
                return Ok(());
            }
        }

        Err(MigrationError::RollbackFailed("No intermediate state available for rollback".to_string()))
    }

    fn record_migration(
        &mut self,
        result: MigrationResult,
        original_state: ComponentState,
        transition: &SchemaTransition,
        context: MigrationContext,
    ) -> Result<(), MigrationError> {
        let record = MigrationRecord {
            result,
            original_state,
            transition: transition.clone(),
            context,
            recorded_at: current_timestamp(),
        };

        let mut history = self.migration_history.lock()
            .map_err(|_| MigrationError::LockError("Failed to acquire migration history lock".to_string()))?;
        
        history.push_front(record);
        
        // Limit history size
        while history.len() > self.config.max_migration_history {
            history.pop_back();
        }

        Ok(())
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        MigrationRegistry {
            migrations: HashMap::new(),
            auto_generators: Vec::new(),
            migration_chains: HashMap::new(),
        }
    }
}

impl MigrationExecutionEngine {
    /// Create a new migration execution engine
    pub fn new(config: MigrationConfig) -> Self {
        MigrationExecutionEngine {
            active_migrations: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        MigrationConfig {
            migration_timeout: Duration::from_secs(60),
            enable_auto_migration: true,
            validate_migrated_state: true,
            max_migration_chain_length: 10,
            preserve_original_state: true,
            max_migration_history: 100,
        }
    }
}

/// Errors that can occur during migration
#[derive(Debug, Clone)]
pub enum MigrationError {
    /// Migration function not found
    MigrationNotFound(SchemaTransition),
    /// No automatic migration available
    NoAutoMigrationAvailable(SchemaTransition),
    /// Automatic migration is disabled
    AutoMigrationDisabled,
    /// Migration chain is too long
    ChainTooLong(usize),
    /// Migration execution failed
    ExecutionFailed(String),
    /// Rollback failed
    RollbackFailed(String),
    /// State validation failed
    ValidationFailed(String),
    /// Lock error
    LockError(String),
    /// Timeout error
    Timeout,
    /// Invalid input
    InvalidInput(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::MigrationNotFound(transition) => {
                write!(f, "Migration not found for transition: {transition:?}")
            }
            MigrationError::NoAutoMigrationAvailable(transition) => {
                write!(f, "No automatic migration available for transition: {transition:?}")
            }
            MigrationError::AutoMigrationDisabled => {
                write!(f, "Automatic migration is disabled")
            }
            MigrationError::ChainTooLong(length) => {
                write!(f, "Migration chain too long: {length}")
            }
            MigrationError::ExecutionFailed(msg) => {
                write!(f, "Migration execution failed: {msg}")
            }
            MigrationError::RollbackFailed(msg) => {
                write!(f, "Migration rollback failed: {msg}")
            }
            MigrationError::ValidationFailed(msg) => {
                write!(f, "Migration validation failed: {msg}")
            }
            MigrationError::LockError(msg) => {
                write!(f, "Lock error: {msg}")
            }
            MigrationError::Timeout => {
                write!(f, "Migration timeout")
            }
            MigrationError::InvalidInput(msg) => {
                write!(f, "Invalid input: {msg}")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ComponentType;

    fn create_test_component_state() -> ComponentState {
        ComponentState {
            component_name: "test_component".to_string(),
            component_type: ComponentType::StatefulFunction,
            values: HashMap::new(),
            connections: Vec::new(),
            is_stateful: true,
            schema_hash: TypeHash::new(ContentHash::new(b"test_schema")),
        }
    }

    fn create_test_schema_change() -> SchemaChange {
        SchemaChange {
            schema_hash: ContentHash::new(b"new_schema"),
            old_version: Some(ContentHash::new(b"old_schema")),
            new_version: ContentHash::new(b"new_schema"),
            change_type: SchemaChangeType::BackwardCompatible,
            migration_path: None,
            affected_modules: Vec::new(),
        }
    }

    #[test]
    fn test_migration_system_creation() {
        let config = MigrationConfig::default();
        let _system = StateMigrationSystem::new(config);
    }

    #[test]
    fn test_schema_transition_creation() {
        let transition = SchemaTransition {
            from_schema: TypeHash::new(ContentHash::new(b"old")),
            to_schema: TypeHash::new(ContentHash::new(b"new")),
            component_type: "test".to_string(),
        };

        assert_eq!(transition.component_type, "test");
    }

    #[test]
    fn test_migration_chain_creation() {
        let config = MigrationConfig::default();
        let system = StateMigrationSystem::new(config);
        
        let schema_changes = vec![create_test_schema_change()];
        let result = system.create_migration_chain(&schema_changes, "test_component");
        
        // This will fail because no migration functions are registered,
        // but it tests the chain creation logic
        assert!(result.is_err());
    }

    #[test]
    fn test_migration_config_defaults() {
        let config = MigrationConfig::default();
        assert!(config.enable_auto_migration);
        assert!(config.validate_migrated_state);
        assert_eq!(config.max_migration_chain_length, 10);
    }
}