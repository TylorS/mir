//! Hot-Module-Reloading Coordinator
//!
//! This module implements Requirements 12, 13, and 14:
//! - HMR orchestration for update planning and execution
//! - UpdatePlan generation from change analysis
//! - Rollback mechanism for failed updates
//! - Validation and safety checks before updates

use crate::change_analysis::{ChangeAnalysis, ChangeAnalysisSystem, CompatibilityLevel};
use crate::module_store::{ModuleStore, ModuleStoreError};
use crate::state::{StateError, StateManager};
use crate::storage::ContentAddressableStore;
use mir_ast::{ASTNode, Module, NodeId};
use mir_types::ContentHash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// HMR coordinator for orchestrating hot-module-reloading updates
pub struct HMRCoordinator<S: ContentAddressableStore> {
    /// Change analysis system
    change_analyzer: ChangeAnalysisSystem<S>,
    /// State manager for preserving state
    state_manager: Arc<StateManager>,
    /// Module store for accessing modules
    module_store: ModuleStore<S>,
    /// Active update plans
    active_updates: Arc<Mutex<HashMap<ContentHash, UpdatePlan>>>,
    /// Update history for rollback
    update_history: Arc<RwLock<VecDeque<UpdateRecord>>>,
    /// Configuration
    config: HMRConfig,
    /// Validation engine
    validator: UpdateValidator,
    /// Rollback manager
    rollback_manager: RollbackManager,
}

/// Configuration for HMR coordinator
#[derive(Debug, Clone)]
pub struct HMRConfig {
    /// Maximum number of concurrent updates
    pub max_concurrent_updates: usize,
    /// Timeout for update operations
    pub update_timeout: Duration,
    /// Whether to perform pre-update validation
    pub enable_pre_validation: bool,
    /// Whether to create rollback snapshots
    pub enable_rollback_snapshots: bool,
    /// Maximum number of update records to keep
    pub max_update_history: usize,
    /// Safety level for updates
    pub safety_level: SafetyLevel,
    /// Whether to allow partial updates
    pub allow_partial_updates: bool,
}

/// Safety level for updates
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyLevel {
    /// Minimal safety checks
    Minimal,
    /// Standard safety checks
    Standard,
    /// Extensive safety checks
    Extensive,
    /// Paranoid safety checks
    Paranoid,
}

/// Plan for executing an update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlan {
    /// Unique ID for this update
    pub update_id: ContentHash,
    /// Module being updated
    pub module_hash: ContentHash,
    /// New module version
    pub new_module_hash: ContentHash,
    /// Change analysis results
    pub change_analysis: ChangeAnalysis,
    /// Steps to execute
    pub steps: Vec<UpdateStep>,
    /// Rollback plan
    pub rollback_plan: Option<RollbackPlan>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Estimated execution time
    pub estimated_duration: Duration,
    /// Priority level
    pub priority: UpdatePriority,
    /// Dependencies on other updates
    pub dependencies: Vec<ContentHash>,
    /// Status of the update
    pub status: UpdateStatus,
    /// Timestamps
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

/// Individual step in an update plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStep {
    /// Step ID
    pub step_id: String,
    /// Type of step
    pub step_type: UpdateStepType,
    /// Description of the step
    pub description: String,
    /// Modules affected by this step
    pub affected_modules: Vec<NodeId>,
    /// Prerequisites for this step
    pub prerequisites: Vec<String>,
    /// Whether this step can be rolled back
    pub is_rollbackable: bool,
    /// Estimated duration
    pub estimated_duration: Duration,
    /// Status of this step
    pub status: StepStatus,
}

/// Type of update step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStepType {
    /// Validate the update
    Validation,
    /// Create state snapshot
    CreateSnapshot,
    /// Stop affected modules
    StopModules,
    /// Update module code
    UpdateCode,
    /// Migrate state
    MigrateState,
    /// Restart modules
    RestartModules,
    /// Verify update success
    VerifyUpdate,
    /// Cleanup old resources
    Cleanup,
}

/// Status of an update step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
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

/// Priority level for updates
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UpdatePriority {
    /// Low priority update
    Low,
    /// Normal priority update
    Normal,
    /// High priority update
    High,
    /// Critical update (security, bug fixes)
    Critical,
}

/// Status of an update plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStatus {
    /// Update is planned but not started
    Planned,
    /// Update is currently executing
    Executing,
    /// Update completed successfully
    Completed,
    /// Update failed
    Failed(String),
    /// Update was cancelled
    Cancelled,
    /// Update was rolled back
    RolledBack,
}

/// Plan for rolling back an update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Rollback ID
    pub rollback_id: ContentHash,
    /// Steps to execute for rollback
    pub steps: Vec<RollbackStep>,
    /// State snapshots to restore
    pub state_snapshots: HashMap<NodeId, ContentHash>,
    /// Module versions to restore
    pub module_versions: HashMap<NodeId, ContentHash>,
    /// Whether rollback is automatic or manual
    pub is_automatic: bool,
    /// Timeout for rollback operations
    pub timeout: Duration,
}

/// Individual step in a rollback plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step ID
    pub step_id: String,
    /// Type of rollback step
    pub step_type: RollbackStepType,
    /// Description
    pub description: String,
    /// Modules affected
    pub affected_modules: Vec<NodeId>,
    /// Status
    pub status: StepStatus,
}

/// Type of rollback step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackStepType {
    /// Stop new modules
    StopNewModules,
    /// Restore old module code
    RestoreCode,
    /// Restore state from snapshot
    RestoreState,
    /// Restart old modules
    RestartOldModules,
    /// Verify rollback success
    VerifyRollback,
    /// Cleanup failed update artifacts
    CleanupFailedUpdate,
}

/// Result of executing an update
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Update ID
    pub update_id: ContentHash,
    /// Whether the update was successful
    pub success: bool,
    /// Steps that were executed
    pub executed_steps: Vec<String>,
    /// Steps that failed
    pub failed_steps: Vec<(String, String)>,
    /// Time taken for the update
    pub execution_time: Duration,
    /// State changes made
    pub state_changes: Vec<StateChangeRecord>,
    /// Warnings generated during update
    pub warnings: Vec<String>,
    /// Whether rollback was performed
    pub was_rolled_back: bool,
}

/// Record of a state change during update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeRecord {
    /// Module that was changed
    pub module_id: NodeId,
    /// Type of change
    pub change_type: String,
    /// Old state hash
    pub old_state_hash: Option<ContentHash>,
    /// New state hash
    pub new_state_hash: ContentHash,
    /// Timestamp of change
    pub timestamp: u64,
}

/// Record of an update for history tracking
#[derive(Debug, Clone)]
pub struct UpdateRecord {
    /// Update plan
    pub plan: UpdatePlan,
    /// Result of the update
    pub result: UpdateResult,
    /// Timestamp when record was created
    pub recorded_at: u64,
}

/// Validation result for an update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validator that produced this result
    pub validator_name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Severity level for validation results
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Information only
    Info,
    /// Warning that should be noted
    Warning,
    /// Error that prevents update
    Error,
    /// Critical error that could cause system failure
    Critical,
}

/// Update validator for safety checks
pub struct UpdateValidator {
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,
    /// Configuration
    config: ValidationConfig,
}

/// Configuration for validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to fail on warnings
    pub fail_on_warnings: bool,
    /// Maximum allowed breaking changes
    pub max_breaking_changes: usize,
    /// Whether to validate dependencies
    pub validate_dependencies: bool,
    /// Timeout for validation operations
    pub validation_timeout: Duration,
}

/// Trait for validation rules
pub trait ValidationRule: Send + Sync {
    /// Name of this validation rule
    fn name(&self) -> &str;

    /// Validate an update plan
    fn validate(
        &self,
        plan: &UpdatePlan,
        module: &Module,
    ) -> Result<ValidationResult, ValidationError>;

    /// Whether this rule is critical (failure prevents update)
    fn is_critical(&self) -> bool;
}

/// Rollback manager for handling failed updates
pub struct RollbackManager {
    /// Active rollback operations
    active_rollbacks: Arc<Mutex<HashMap<ContentHash, RollbackExecution>>>,
    /// Configuration
    #[allow(dead_code)]
    config: RollbackConfig,
}

/// Configuration for rollback operations
#[derive(Debug, Clone)]
pub struct RollbackConfig {
    /// Timeout for rollback operations
    pub rollback_timeout: Duration,
    /// Whether to automatically rollback on failure
    pub auto_rollback: bool,
    /// Maximum number of rollback attempts
    pub max_rollback_attempts: usize,
    /// Whether to preserve failed update artifacts
    pub preserve_failed_artifacts: bool,
}

/// Execution state of a rollback operation
#[derive(Debug, Clone)]
pub struct RollbackExecution {
    /// Rollback plan being executed
    pub plan: RollbackPlan,
    /// Current step being executed
    pub current_step: usize,
    /// Start time
    pub started_at: Instant,
    /// Number of attempts
    pub attempts: usize,
    /// Status
    pub status: RollbackStatus,
}

/// Status of a rollback operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackStatus {
    /// Rollback is running
    Running,
    /// Rollback completed successfully
    Completed,
    /// Rollback failed
    Failed(String),
    /// Rollback was cancelled
    Cancelled,
}

impl<S: ContentAddressableStore> HMRCoordinator<S> {
    /// Create a new HMR coordinator
    pub fn new(
        change_analyzer: ChangeAnalysisSystem<S>,
        state_manager: Arc<StateManager>,
        module_store: ModuleStore<S>,
        config: HMRConfig,
    ) -> Self {
        HMRCoordinator {
            change_analyzer,
            state_manager,
            module_store,
            active_updates: Arc::new(Mutex::new(HashMap::new())),
            update_history: Arc::new(RwLock::new(VecDeque::new())),
            config,
            validator: UpdateValidator::new(ValidationConfig::default()),
            rollback_manager: RollbackManager::new(RollbackConfig::default()),
        }
    }

    /// Analyze a change and create an update plan
    pub fn plan_update(
        &mut self,
        old_hash: ContentHash,
        new_module: &Module,
    ) -> Result<UpdatePlan, HMRError> {
        // Perform change analysis
        let change_analysis = self.change_analyzer.analyze_change(old_hash, new_module)?;

        // Generate update ID
        let update_id =
            ContentHash::new(format!("{:?}{}", change_analysis, current_timestamp()).as_bytes());

        // Create update steps based on analysis
        let steps = self.generate_update_steps(&change_analysis, new_module)?;

        // Create rollback plan if enabled
        let rollback_plan = if self.config.enable_rollback_snapshots {
            Some(self.create_rollback_plan(&change_analysis, old_hash)?)
        } else {
            None
        };

        // Estimate execution time
        let estimated_duration = self.estimate_execution_time(&steps);

        // Determine priority
        let priority = self.determine_update_priority(&change_analysis);

        // Create the update plan
        let plan = UpdatePlan {
            update_id,
            module_hash: old_hash,
            new_module_hash: new_module.content_hash(),
            change_analysis,
            steps,
            rollback_plan,
            validation_results: Vec::new(),
            estimated_duration,
            priority,
            dependencies: Vec::new(), // TODO: Analyze dependencies
            status: UpdateStatus::Planned,
            created_at: current_timestamp(),
            started_at: None,
            completed_at: None,
        };

        Ok(plan)
    }

    /// Execute an update plan
    pub fn execute_update(&mut self, mut plan: UpdatePlan) -> Result<UpdateResult, HMRError> {
        let start_time = Instant::now();
        let update_id = plan.update_id;

        // Check if we can execute this update
        self.check_update_capacity()?;

        // Validate the update if enabled
        if self.config.enable_pre_validation {
            let new_module = self
                .module_store
                .retrieve_module_by_hash(plan.new_module_hash, Default::default())?
                .ok_or(HMRError::ModuleNotFound(plan.new_module_hash))?;

            plan.validation_results = self.validator.validate_update(&plan, &new_module)?;

            // Check if validation passed
            if !self.validation_passed(&plan.validation_results) {
                return Err(HMRError::ValidationFailed(
                    plan.validation_results
                        .iter()
                        .filter(|r| !r.passed)
                        .map(|r| r.message.clone())
                        .collect::<Vec<_>>()
                        .join("; "),
                ));
            }
        }

        // Mark update as active
        {
            let mut active_updates = self.active_updates.lock().map_err(|_| {
                HMRError::LockError("Failed to acquire active updates lock".to_string())
            })?;
            active_updates.insert(update_id, plan.clone());
        }

        // Update plan status
        plan.status = UpdateStatus::Executing;
        plan.started_at = Some(current_timestamp());

        // Execute steps
        let mut executed_steps = Vec::new();
        let mut failed_steps = Vec::new();
        let mut state_changes = Vec::new();
        let mut warnings = Vec::new();
        let mut was_rolled_back = false;

        let num_steps = plan.steps.len();
        for i in 0..num_steps {
            // Update step status
            plan.steps[i].status = StepStatus::Running;

            // Clone the step to avoid borrowing issues
            let step_clone = plan.steps[i].clone();

            match self.execute_step(&step_clone, &plan) {
                Ok(step_result) => {
                    plan.steps[i].status = StepStatus::Completed;
                    executed_steps.push(step_clone.step_id.clone());

                    // Collect state changes
                    state_changes.extend(step_result.state_changes);
                    warnings.extend(step_result.warnings);
                }
                Err(e) => {
                    plan.steps[i].status = StepStatus::Failed(e.to_string());
                    failed_steps.push((step_clone.step_id.clone(), e.to_string()));

                    // If this is a critical step, attempt rollback
                    if step_clone.is_rollbackable && plan.rollback_plan.is_some() {
                        match self.execute_rollback(&plan) {
                            Ok(_) => {
                                was_rolled_back = true;
                                // Mark remaining steps as rolled back
                                for j in (i + 1)..num_steps {
                                    plan.steps[j].status = StepStatus::RolledBack;
                                }
                                break;
                            }
                            Err(rollback_err) => {
                                warnings.push(format!("Rollback failed: {rollback_err}"));
                            }
                        }
                    }

                    if !self.config.allow_partial_updates {
                        break;
                    }
                }
            }
        }

        // Update plan status
        plan.status = if failed_steps.is_empty() {
            UpdateStatus::Completed
        } else if was_rolled_back {
            UpdateStatus::RolledBack
        } else {
            UpdateStatus::Failed(format!(
                "Failed steps: {}",
                failed_steps
                    .iter()
                    .map(|(id, _)| id)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        };

        plan.completed_at = Some(current_timestamp());

        // Remove from active updates
        {
            let mut active_updates = self.active_updates.lock().map_err(|_| {
                HMRError::LockError("Failed to acquire active updates lock".to_string())
            })?;
            active_updates.remove(&update_id);
        }

        let execution_time = start_time.elapsed();
        let success = failed_steps.is_empty();

        let result = UpdateResult {
            update_id,
            success,
            executed_steps,
            failed_steps,
            execution_time,
            state_changes,
            warnings,
            was_rolled_back,
        };

        // Record the update in history
        self.record_update(plan, result.clone())?;

        Ok(result)
    }

    /// Execute a rollback plan
    pub fn execute_rollback(&mut self, plan: &UpdatePlan) -> Result<(), HMRError> {
        let rollback_plan = plan
            .rollback_plan
            .as_ref()
            .ok_or(HMRError::NoRollbackPlan(plan.update_id))?;

        self.rollback_manager.execute_rollback(
            rollback_plan.clone(),
            &self.state_manager,
            &self.module_store,
        )
    }

    /// Get the status of an active update
    pub fn get_update_status(
        &self,
        update_id: ContentHash,
    ) -> Result<Option<UpdateStatus>, HMRError> {
        let active_updates = self.active_updates.lock().map_err(|_| {
            HMRError::LockError("Failed to acquire active updates lock".to_string())
        })?;

        Ok(active_updates
            .get(&update_id)
            .map(|plan| plan.status.clone()))
    }

    /// Cancel an active update
    pub fn cancel_update(&mut self, update_id: ContentHash) -> Result<(), HMRError> {
        let mut active_updates = self.active_updates.lock().map_err(|_| {
            HMRError::LockError("Failed to acquire active updates lock".to_string())
        })?;

        if let Some(mut plan) = active_updates.remove(&update_id) {
            plan.status = UpdateStatus::Cancelled;

            // If rollback is available, execute it
            if plan.rollback_plan.is_some() {
                drop(active_updates); // Release the lock
                self.execute_rollback(&plan)?;
            }

            Ok(())
        } else {
            Err(HMRError::UpdateNotFound(update_id))
        }
    }

    /// Get update history
    pub fn get_update_history(&self, limit: Option<usize>) -> Result<Vec<UpdateRecord>, HMRError> {
        let history = self.update_history.read().map_err(|_| {
            HMRError::LockError("Failed to acquire update history lock".to_string())
        })?;

        let limit = limit.unwrap_or(history.len());
        Ok(history.iter().take(limit).cloned().collect())
    }

    // Private helper methods

    fn generate_update_steps(
        &self,
        analysis: &ChangeAnalysis,
        _module: &Module,
    ) -> Result<Vec<UpdateStep>, HMRError> {
        let mut steps = Vec::new();
        let mut step_counter = 0;

        // Always start with validation
        steps.push(UpdateStep {
            step_id: format!("validate_{step_counter}"),
            step_type: UpdateStepType::Validation,
            description: "Validate update compatibility and safety".to_string(),
            affected_modules: analysis
                .affected_modules
                .iter()
                .map(|m| m.module_id)
                .collect(),
            prerequisites: Vec::new(),
            is_rollbackable: false,
            estimated_duration: Duration::from_millis(100),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Create snapshot if state migration is needed
        if analysis.migration_required {
            steps.push(UpdateStep {
                step_id: format!("snapshot_{step_counter}"),
                step_type: UpdateStepType::CreateSnapshot,
                description: "Create state snapshot for migration".to_string(),
                affected_modules: analysis
                    .affected_modules
                    .iter()
                    .map(|m| m.module_id)
                    .collect(),
                prerequisites: vec![format!("validate_{}", step_counter - 1)],
                is_rollbackable: true,
                estimated_duration: Duration::from_millis(500),
                status: StepStatus::Pending,
            });
            step_counter += 1;
        }

        // Stop affected modules
        if !analysis.affected_modules.is_empty() {
            steps.push(UpdateStep {
                step_id: format!("stop_{step_counter}"),
                step_type: UpdateStepType::StopModules,
                description: "Stop affected modules".to_string(),
                affected_modules: analysis
                    .affected_modules
                    .iter()
                    .map(|m| m.module_id)
                    .collect(),
                prerequisites: if analysis.migration_required {
                    vec![format!("snapshot_{}", step_counter - 1)]
                } else {
                    vec![format!("validate_{}", step_counter - 1)]
                },
                is_rollbackable: true,
                estimated_duration: Duration::from_millis(200),
                status: StepStatus::Pending,
            });
            step_counter += 1;
        }

        // Update code
        steps.push(UpdateStep {
            step_id: format!("update_{step_counter}"),
            step_type: UpdateStepType::UpdateCode,
            description: "Update module code".to_string(),
            affected_modules: vec![content_hash_to_node_id(analysis.changed_module_hash)],
            prerequisites: vec![format!("stop_{}", step_counter - 1)],
            is_rollbackable: true,
            estimated_duration: Duration::from_millis(300),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Migrate state if needed
        if analysis.migration_required {
            steps.push(UpdateStep {
                step_id: format!("migrate_{step_counter}"),
                step_type: UpdateStepType::MigrateState,
                description: "Migrate state to new schema".to_string(),
                affected_modules: analysis
                    .affected_modules
                    .iter()
                    .map(|m| m.module_id)
                    .collect(),
                prerequisites: vec![format!("update_{}", step_counter - 1)],
                is_rollbackable: true,
                estimated_duration: Duration::from_millis(1000),
                status: StepStatus::Pending,
            });
            step_counter += 1;
        }

        // Restart modules
        steps.push(UpdateStep {
            step_id: format!("restart_{step_counter}"),
            step_type: UpdateStepType::RestartModules,
            description: "Restart updated modules".to_string(),
            affected_modules: analysis
                .affected_modules
                .iter()
                .map(|m| m.module_id)
                .collect(),
            prerequisites: if analysis.migration_required {
                vec![format!("migrate_{}", step_counter - 1)]
            } else {
                vec![format!("update_{}", step_counter - 1)]
            },
            is_rollbackable: true,
            estimated_duration: Duration::from_millis(400),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Verify update
        steps.push(UpdateStep {
            step_id: format!("verify_{step_counter}"),
            step_type: UpdateStepType::VerifyUpdate,
            description: "Verify update was successful".to_string(),
            affected_modules: analysis
                .affected_modules
                .iter()
                .map(|m| m.module_id)
                .collect(),
            prerequisites: vec![format!("restart_{}", step_counter - 1)],
            is_rollbackable: false,
            estimated_duration: Duration::from_millis(200),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Cleanup
        steps.push(UpdateStep {
            step_id: format!("cleanup_{step_counter}"),
            step_type: UpdateStepType::Cleanup,
            description: "Cleanup old resources".to_string(),
            affected_modules: Vec::new(),
            prerequisites: vec![format!("verify_{}", step_counter - 1)],
            is_rollbackable: false,
            estimated_duration: Duration::from_millis(100),
            status: StepStatus::Pending,
        });

        Ok(steps)
    }

    fn create_rollback_plan(
        &self,
        analysis: &ChangeAnalysis,
        old_hash: ContentHash,
    ) -> Result<RollbackPlan, HMRError> {
        let rollback_id = ContentHash::new(
            format!(
                "rollback_{:?}_{}",
                analysis.changed_module_hash,
                current_timestamp()
            )
            .as_bytes(),
        );

        let mut steps = Vec::new();
        let mut step_counter = 0;

        // Stop new modules
        steps.push(RollbackStep {
            step_id: format!("stop_new_{step_counter}"),
            step_type: RollbackStepType::StopNewModules,
            description: "Stop newly updated modules".to_string(),
            affected_modules: analysis
                .affected_modules
                .iter()
                .map(|m| m.module_id)
                .collect(),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Restore old code
        steps.push(RollbackStep {
            step_id: format!("restore_code_{step_counter}"),
            step_type: RollbackStepType::RestoreCode,
            description: "Restore old module code".to_string(),
            affected_modules: vec![content_hash_to_node_id(analysis.changed_module_hash)],
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Restore state if migration was performed
        if analysis.migration_required {
            steps.push(RollbackStep {
                step_id: format!("restore_state_{step_counter}"),
                step_type: RollbackStepType::RestoreState,
                description: "Restore state from snapshot".to_string(),
                affected_modules: analysis
                    .affected_modules
                    .iter()
                    .map(|m| m.module_id)
                    .collect(),
                status: StepStatus::Pending,
            });
            step_counter += 1;
        }

        // Restart old modules
        steps.push(RollbackStep {
            step_id: format!("restart_old_{step_counter}"),
            step_type: RollbackStepType::RestartOldModules,
            description: "Restart old modules".to_string(),
            affected_modules: analysis
                .affected_modules
                .iter()
                .map(|m| m.module_id)
                .collect(),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Verify rollback
        steps.push(RollbackStep {
            step_id: format!("verify_rollback_{step_counter}"),
            step_type: RollbackStepType::VerifyRollback,
            description: "Verify rollback was successful".to_string(),
            affected_modules: analysis
                .affected_modules
                .iter()
                .map(|m| m.module_id)
                .collect(),
            status: StepStatus::Pending,
        });
        step_counter += 1;

        // Cleanup failed update
        steps.push(RollbackStep {
            step_id: format!("cleanup_failed_{step_counter}"),
            step_type: RollbackStepType::CleanupFailedUpdate,
            description: "Cleanup failed update artifacts".to_string(),
            affected_modules: Vec::new(),
            status: StepStatus::Pending,
        });

        let mut state_snapshots = HashMap::new();
        let mut module_versions = HashMap::new();

        // Collect snapshots and versions for rollback
        for affected_module in &analysis.affected_modules {
            // In a real implementation, we would get the actual snapshot hash
            state_snapshots.insert(affected_module.module_id, ContentHash::zero());
            module_versions.insert(affected_module.module_id, old_hash);
        }

        Ok(RollbackPlan {
            rollback_id,
            steps,
            state_snapshots,
            module_versions,
            is_automatic: true,
            timeout: Duration::from_secs(30),
        })
    }

    fn estimate_execution_time(&self, steps: &[UpdateStep]) -> Duration {
        steps.iter().map(|step| step.estimated_duration).sum()
    }

    fn determine_update_priority(&self, analysis: &ChangeAnalysis) -> UpdatePriority {
        match analysis.compatibility {
            CompatibilityLevel::FullyCompatible => UpdatePriority::Low,
            CompatibilityLevel::BackwardCompatible => UpdatePriority::Normal,
            CompatibilityLevel::RequiresMigration => UpdatePriority::High,
            CompatibilityLevel::Breaking => UpdatePriority::Critical,
        }
    }

    fn check_update_capacity(&self) -> Result<(), HMRError> {
        let active_updates = self.active_updates.lock().map_err(|_| {
            HMRError::LockError("Failed to acquire active updates lock".to_string())
        })?;

        if active_updates.len() >= self.config.max_concurrent_updates {
            return Err(HMRError::UpdateCapacityExceeded(active_updates.len()));
        }

        Ok(())
    }

    fn validation_passed(&self, results: &[ValidationResult]) -> bool {
        for result in results {
            if !result.passed {
                match result.severity {
                    ValidationSeverity::Error | ValidationSeverity::Critical => return false,
                    ValidationSeverity::Warning if self.validator.config.fail_on_warnings => {
                        return false
                    }
                    _ => continue,
                }
            }
        }
        true
    }

    fn execute_step(
        &mut self,
        step: &UpdateStep,
        _plan: &UpdatePlan,
    ) -> Result<StepExecutionResult, HMRError> {
        // Simplified step execution - in a real implementation, this would
        // perform the actual operations based on step type
        match step.step_type {
            UpdateStepType::Validation => {
                // Perform validation
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::CreateSnapshot => {
                // Create state snapshot
                // In real implementation, would call state_manager.create_snapshot()
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::StopModules => {
                // Stop modules
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::UpdateCode => {
                // Update module code
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::MigrateState => {
                // Migrate state
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::RestartModules => {
                // Restart modules
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::VerifyUpdate => {
                // Verify update
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
            UpdateStepType::Cleanup => {
                // Cleanup
                Ok(StepExecutionResult {
                    state_changes: Vec::new(),
                    warnings: Vec::new(),
                })
            }
        }
    }

    fn record_update(&mut self, plan: UpdatePlan, result: UpdateResult) -> Result<(), HMRError> {
        let record = UpdateRecord {
            plan,
            result,
            recorded_at: current_timestamp(),
        };

        let mut history = self.update_history.write().map_err(|_| {
            HMRError::LockError("Failed to acquire update history lock".to_string())
        })?;

        history.push_front(record);

        // Limit history size
        while history.len() > self.config.max_update_history {
            history.pop_back();
        }

        Ok(())
    }
}

/// Result of executing a single step
#[derive(Debug, Clone)]
struct StepExecutionResult {
    /// State changes made during this step
    state_changes: Vec<StateChangeRecord>,
    /// Warnings generated during this step
    warnings: Vec<String>,
}

impl UpdateValidator {
    /// Create a new update validator
    pub fn new(config: ValidationConfig) -> Self {
        UpdateValidator {
            rules: Vec::new(),
            config,
        }
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate an update plan
    pub fn validate_update(
        &self,
        plan: &UpdatePlan,
        module: &Module,
    ) -> Result<Vec<ValidationResult>, ValidationError> {
        let mut results = Vec::new();

        for rule in &self.rules {
            match rule.validate(plan, module) {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(ValidationResult {
                        validator_name: rule.name().to_string(),
                        passed: false,
                        message: format!("Validation rule failed: {e}"),
                        severity: ValidationSeverity::Error,
                        suggested_actions: Vec::new(),
                    });
                }
            }
        }

        Ok(results)
    }
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new(config: RollbackConfig) -> Self {
        RollbackManager {
            active_rollbacks: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Execute a rollback plan
    pub fn execute_rollback<S: ContentAddressableStore>(
        &mut self,
        plan: RollbackPlan,
        _state_manager: &StateManager,
        _module_store: &ModuleStore<S>,
    ) -> Result<(), HMRError> {
        let rollback_id = plan.rollback_id;

        let execution = RollbackExecution {
            plan: plan.clone(),
            current_step: 0,
            started_at: Instant::now(),
            attempts: 1,
            status: RollbackStatus::Running,
        };

        // Mark rollback as active
        {
            let mut active_rollbacks = self.active_rollbacks.lock().map_err(|_| {
                HMRError::LockError("Failed to acquire active rollbacks lock".to_string())
            })?;
            active_rollbacks.insert(rollback_id, execution);
        }

        // Execute rollback steps
        for step in &plan.steps {
            match self.execute_rollback_step(step) {
                Ok(_) => {
                    // Step completed successfully
                }
                Err(e) => {
                    // Mark rollback as failed
                    let mut active_rollbacks = self.active_rollbacks.lock().map_err(|_| {
                        HMRError::LockError("Failed to acquire active rollbacks lock".to_string())
                    })?;

                    if let Some(execution) = active_rollbacks.get_mut(&rollback_id) {
                        execution.status = RollbackStatus::Failed(e.to_string());
                    }

                    return Err(HMRError::RollbackFailed(e.to_string()));
                }
            }
        }

        // Mark rollback as completed
        {
            let mut active_rollbacks = self.active_rollbacks.lock().map_err(|_| {
                HMRError::LockError("Failed to acquire active rollbacks lock".to_string())
            })?;

            if let Some(execution) = active_rollbacks.get_mut(&rollback_id) {
                execution.status = RollbackStatus::Completed;
            }
        }

        Ok(())
    }

    fn execute_rollback_step(&self, _step: &RollbackStep) -> Result<(), HMRError> {
        // Simplified rollback step execution
        // In a real implementation, this would perform the actual rollback operations
        Ok(())
    }
}

impl Default for HMRConfig {
    fn default() -> Self {
        HMRConfig {
            max_concurrent_updates: 5,
            update_timeout: Duration::from_secs(300), // 5 minutes
            enable_pre_validation: true,
            enable_rollback_snapshots: true,
            max_update_history: 100,
            safety_level: SafetyLevel::Standard,
            allow_partial_updates: false,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        ValidationConfig {
            fail_on_warnings: false,
            max_breaking_changes: 5,
            validate_dependencies: true,
            validation_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for RollbackConfig {
    fn default() -> Self {
        RollbackConfig {
            rollback_timeout: Duration::from_secs(60),
            auto_rollback: true,
            max_rollback_attempts: 3,
            preserve_failed_artifacts: true,
        }
    }
}

/// Errors that can occur during HMR operations
#[derive(Debug, Clone)]
pub enum HMRError {
    /// Change analysis error
    ChangeAnalysis(String),
    /// State management error
    StateManagement(String),
    /// Module store error
    ModuleStore(String),
    /// Validation failed
    ValidationFailed(String),
    /// Update not found
    UpdateNotFound(ContentHash),
    /// Module not found
    ModuleNotFound(ContentHash),
    /// No rollback plan available
    NoRollbackPlan(ContentHash),
    /// Rollback failed
    RollbackFailed(String),
    /// Update capacity exceeded
    UpdateCapacityExceeded(usize),
    /// Lock error
    LockError(String),
    /// Timeout error
    Timeout(String),
    /// Invalid configuration
    InvalidConfiguration(String),
}

impl std::fmt::Display for HMRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HMRError::ChangeAnalysis(msg) => write!(f, "Change analysis error: {msg}"),
            HMRError::StateManagement(msg) => write!(f, "State management error: {msg}"),
            HMRError::ModuleStore(msg) => write!(f, "Module store error: {msg}"),
            HMRError::ValidationFailed(msg) => write!(f, "Validation failed: {msg}"),
            HMRError::UpdateNotFound(hash) => write!(f, "Update not found: {hash}"),
            HMRError::ModuleNotFound(hash) => write!(f, "Module not found: {hash}"),
            HMRError::NoRollbackPlan(hash) => write!(f, "No rollback plan for update: {hash}"),
            HMRError::RollbackFailed(msg) => write!(f, "Rollback failed: {msg}"),
            HMRError::UpdateCapacityExceeded(count) => {
                write!(f, "Update capacity exceeded: {count}")
            }
            HMRError::LockError(msg) => write!(f, "Lock error: {msg}"),
            HMRError::Timeout(msg) => write!(f, "Timeout: {msg}"),
            HMRError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {msg}"),
        }
    }
}

impl std::error::Error for HMRError {}

/// Validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Rule execution failed
    RuleExecutionFailed(String),
    /// Invalid input
    InvalidInput(String),
    /// Timeout
    Timeout,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::RuleExecutionFailed(msg) => {
                write!(f, "Rule execution failed: {msg}")
            }
            ValidationError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            ValidationError::Timeout => write!(f, "Validation timeout"),
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<crate::change_analysis::ChangeAnalysisError> for HMRError {
    fn from(err: crate::change_analysis::ChangeAnalysisError) -> Self {
        HMRError::ChangeAnalysis(err.to_string())
    }
}

impl From<StateError> for HMRError {
    fn from(err: StateError) -> Self {
        HMRError::StateManagement(err.to_string())
    }
}

impl From<ModuleStoreError> for HMRError {
    fn from(err: ModuleStoreError) -> Self {
        HMRError::ModuleStore(err.to_string())
    }
}

impl From<ValidationError> for HMRError {
    fn from(err: ValidationError) -> Self {
        HMRError::ValidationFailed(err.to_string())
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Helper function to convert ContentHash to NodeId
fn content_hash_to_node_id(hash: ContentHash) -> NodeId {
    // In a real implementation, this would be a proper conversion
    // For now, we'll use a simplified approach
    NodeId::new(hash.as_bytes()[0] as u64)
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
    fn test_hmr_coordinator_creation() {
        let storage = InMemoryStore::new();
        let module_store = ModuleStore::new(storage.clone());
        let change_analyzer = ChangeAnalysisSystem::new(ModuleStore::new(storage.clone()));
        let state_manager = Arc::new(StateManager::with_defaults());
        let config = HMRConfig::default();

        let _coordinator =
            HMRCoordinator::new(change_analyzer, state_manager, module_store, config);
    }

    #[test]
    fn test_update_plan_creation() {
        let storage = InMemoryStore::new();
        let module_store = ModuleStore::new(storage.clone());
        let change_analyzer = ChangeAnalysisSystem::new(ModuleStore::new(storage.clone()));
        let state_manager = Arc::new(StateManager::with_defaults());
        let config = HMRConfig::default();

        let mut coordinator =
            HMRCoordinator::new(change_analyzer, state_manager, module_store, config);

        let old_hash = ContentHash::new(b"old_module");
        let new_module = create_test_module(1, "test_module");

        let result = coordinator.plan_update(old_hash, &new_module);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.module_hash, old_hash);
        assert_eq!(plan.new_module_hash, new_module.content_hash());
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_rollback_plan_creation() {
        let storage = InMemoryStore::new();
        let module_store = ModuleStore::new(storage.clone());
        let change_analyzer = ChangeAnalysisSystem::new(ModuleStore::new(storage.clone()));
        let state_manager = Arc::new(StateManager::with_defaults());
        let config = HMRConfig::default();

        let coordinator = HMRCoordinator::new(change_analyzer, state_manager, module_store, config);

        let old_hash = ContentHash::new(b"old_module");
        let analysis = ChangeAnalysis {
            changed_module_hash: ContentHash::new(b"new_module"),
            affected_modules: Vec::new(),
            schema_changes: Vec::new(),
            compatibility: CompatibilityLevel::FullyCompatible,
            migration_required: false,
            optimizations: Vec::new(),
            dependency_changes: Vec::new(),
        };

        let result = coordinator.create_rollback_plan(&analysis, old_hash);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.steps.is_empty());
        assert!(plan.is_automatic);
    }

    #[test]
    fn test_update_priority_determination() {
        let storage = InMemoryStore::new();
        let module_store = ModuleStore::new(storage.clone());
        let change_analyzer = ChangeAnalysisSystem::new(ModuleStore::new(storage.clone()));
        let state_manager = Arc::new(StateManager::with_defaults());
        let config = HMRConfig::default();

        let coordinator = HMRCoordinator::new(change_analyzer, state_manager, module_store, config);

        let analysis = ChangeAnalysis {
            changed_module_hash: ContentHash::new(b"test"),
            affected_modules: Vec::new(),
            schema_changes: Vec::new(),
            compatibility: CompatibilityLevel::Breaking,
            migration_required: true,
            optimizations: Vec::new(),
            dependency_changes: Vec::new(),
        };

        let priority = coordinator.determine_update_priority(&analysis);
        assert_eq!(priority, UpdatePriority::Critical);
    }
}
