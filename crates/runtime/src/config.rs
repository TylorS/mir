//! Runtime configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub environment: Environment,
    pub hmr_enabled: bool,
    pub telemetry_enabled: bool,
    pub storage_backend: StorageBackend,
    pub hmr_config: HMRConfig,
}

/// HMR-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMRConfig {
    pub environment: Environment,
    pub update_policy: UpdatePolicy,
    pub validation_level: ValidationLevel,
    pub rollback_strategy: RollbackStrategy,
    pub consensus_timeout: Duration,
    pub state_preservation: StatePreservationConfig,
}

/// Environment-specific configuration with detailed settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development {
        auto_reload: bool,
        file_watcher_enabled: bool,
        aggressive_optimization: bool,
    },
    Production {
        require_explicit_deployment: bool,
        staged_rollout: bool,
        canary_percentage: f32,
    },
    Testing {
        deterministic_execution: bool,
        chaos_testing_enabled: bool,
    },
}

/// Update policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdatePolicy {
    /// Apply updates immediately when detected
    Automatic,
    /// Require explicit trigger
    Manual,
    /// Apply during maintenance windows
    Scheduled(Schedule),
    /// Require cluster consensus
    Consensus,
}

/// Schedule configuration for scheduled updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Cron expression for scheduling
    pub cron_expression: String,
    /// Time zone for schedule interpretation
    pub timezone: String,
    /// Maximum delay before forcing update
    pub max_delay: Duration,
}

/// Validation level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Basic syntax and type checking
    Minimal,
    /// Full compatibility analysis
    Standard,
    /// Deep validation including integration tests
    Extensive,
    /// Maximum validation with formal verification
    Paranoid,
}

/// Rollback strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStrategy {
    /// Automatic rollback on failure
    pub auto_rollback: bool,
    /// Timeout before triggering rollback
    pub rollback_timeout: Duration,
    /// Maximum number of rollback attempts
    pub max_rollback_attempts: u32,
    /// Strategy for handling rollback failures
    pub rollback_failure_strategy: RollbackFailureStrategy,
    /// Health checks to perform before rollback
    pub health_checks: Vec<HealthCheck>,
}

/// Strategy for handling rollback failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackFailureStrategy {
    /// Panic and stop the system
    Panic,
    /// Continue with degraded functionality
    Degrade,
    /// Attempt manual intervention
    ManualIntervention,
    /// Restart the entire system
    Restart,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Name of the health check
    pub name: String,
    /// Function to execute for health check
    pub check_function: String,
    /// Timeout for the health check
    pub timeout: Duration,
    /// Number of retries on failure
    pub retries: u32,
}

/// State preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePreservationConfig {
    /// Enable state preservation during updates
    pub enabled: bool,
    /// Maximum size of state to preserve (in bytes)
    pub max_state_size: usize,
    /// Compression level for state snapshots
    pub compression_level: u8,
    /// Encryption settings for state snapshots
    pub encryption: Option<EncryptionConfig>,
    /// Backup strategy for state preservation
    pub backup_strategy: BackupStrategy,
}

/// Encryption configuration for state snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption algorithm to use
    pub algorithm: String,
    /// Key derivation function
    pub key_derivation: String,
    /// Salt for key derivation
    pub salt: Vec<u8>,
}

/// Backup strategy for state preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStrategy {
    /// No backup
    None,
    /// Local file backup
    Local { path: String },
    /// Distributed backup across nodes
    Distributed { replication_factor: u32 },
    /// External storage backup
    External { endpoint: String, credentials: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    InMemory,
    FileSystem { path: String },
    Distributed { nodes: Vec<String> },
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            environment: Environment::Development {
                auto_reload: true,
                file_watcher_enabled: true,
                aggressive_optimization: false,
            },
            hmr_enabled: true,
            telemetry_enabled: true,
            storage_backend: StorageBackend::InMemory,
            hmr_config: HMRConfig::default(),
        }
    }
}

impl Default for HMRConfig {
    fn default() -> Self {
        HMRConfig {
            environment: Environment::Development {
                auto_reload: true,
                file_watcher_enabled: true,
                aggressive_optimization: false,
            },
            update_policy: UpdatePolicy::Automatic,
            validation_level: ValidationLevel::Standard,
            rollback_strategy: RollbackStrategy::default(),
            consensus_timeout: Duration::from_secs(30),
            state_preservation: StatePreservationConfig::default(),
        }
    }
}

impl Default for RollbackStrategy {
    fn default() -> Self {
        RollbackStrategy {
            auto_rollback: true,
            rollback_timeout: Duration::from_secs(60),
            max_rollback_attempts: 3,
            rollback_failure_strategy: RollbackFailureStrategy::Degrade,
            health_checks: vec![],
        }
    }
}

impl Default for StatePreservationConfig {
    fn default() -> Self {
        StatePreservationConfig {
            enabled: true,
            max_state_size: 100 * 1024 * 1024, // 100MB
            compression_level: 6,
            encryption: None,
            backup_strategy: BackupStrategy::None,
        }
    }
}

impl Environment {
    /// Check if this is a development environment
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development { .. })
    }

    /// Check if this is a production environment
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production { .. })
    }

    /// Check if this is a testing environment
    pub fn is_testing(&self) -> bool {
        matches!(self, Environment::Testing { .. })
    }

    /// Get auto-reload setting if applicable
    pub fn auto_reload(&self) -> bool {
        match self {
            Environment::Development { auto_reload, .. } => *auto_reload,
            _ => false,
        }
    }

    /// Get file watcher setting if applicable
    pub fn file_watcher_enabled(&self) -> bool {
        match self {
            Environment::Development { file_watcher_enabled, .. } => *file_watcher_enabled,
            _ => false,
        }
    }

    /// Get explicit deployment requirement if applicable
    pub fn require_explicit_deployment(&self) -> bool {
        match self {
            Environment::Production { require_explicit_deployment, .. } => *require_explicit_deployment,
            _ => false,
        }
    }

    /// Get staged rollout setting if applicable
    pub fn staged_rollout(&self) -> bool {
        match self {
            Environment::Production { staged_rollout, .. } => *staged_rollout,
            _ => false,
        }
    }

    /// Get canary percentage if applicable
    pub fn canary_percentage(&self) -> Option<f32> {
        match self {
            Environment::Production { canary_percentage, .. } => Some(*canary_percentage),
            _ => None,
        }
    }

    /// Get deterministic execution setting if applicable
    pub fn deterministic_execution(&self) -> bool {
        match self {
            Environment::Testing { deterministic_execution, .. } => *deterministic_execution,
            _ => false,
        }
    }

    /// Get chaos testing setting if applicable
    pub fn chaos_testing_enabled(&self) -> bool {
        match self {
            Environment::Testing { chaos_testing_enabled, .. } => *chaos_testing_enabled,
            _ => false,
        }
    }
}

impl UpdatePolicy {
    /// Check if updates should be applied automatically
    pub fn is_automatic(&self) -> bool {
        matches!(self, UpdatePolicy::Automatic)
    }

    /// Check if updates require manual trigger
    pub fn is_manual(&self) -> bool {
        matches!(self, UpdatePolicy::Manual)
    }

    /// Check if updates are scheduled
    pub fn is_scheduled(&self) -> bool {
        matches!(self, UpdatePolicy::Scheduled(_))
    }

    /// Check if updates require consensus
    pub fn requires_consensus(&self) -> bool {
        matches!(self, UpdatePolicy::Consensus)
    }

    /// Get schedule if applicable
    pub fn schedule(&self) -> Option<&Schedule> {
        match self {
            UpdatePolicy::Scheduled(schedule) => Some(schedule),
            _ => None,
        }
    }
}

impl ValidationLevel {
    /// Get the validation intensity as a numeric value
    pub fn intensity(&self) -> u8 {
        match self {
            ValidationLevel::Minimal => 1,
            ValidationLevel::Standard => 2,
            ValidationLevel::Extensive => 3,
            ValidationLevel::Paranoid => 4,
        }
    }

    /// Check if this level includes the given level
    pub fn includes(&self, other: &ValidationLevel) -> bool {
        self.intensity() >= other.intensity()
    }
}

/// Environment adapter for environment-specific behavior
pub trait EnvironmentAdapter {
    /// Configure the adapter for a specific environment
    fn configure_for_environment(&mut self, env: Environment) -> Result<(), ConfigError>;
    
    /// Validate update safety based on environment settings
    fn validate_update_safety(&self, update: &UpdatePlan) -> SafetyAssessment;
    
    /// Determine if auto-reload should be enabled for a change
    fn should_auto_reload(&self, change: &ChangeAnalysis) -> bool;
    
    /// Get rollback timeout based on environment
    fn get_rollback_timeout(&self) -> Duration;
    
    /// Get deployment strategy recommendation
    fn recommend_deployment_strategy(&self, update: &UpdatePlan) -> DeploymentStrategy;
}

/// Basic implementation of EnvironmentAdapter
#[derive(Debug, Clone)]
pub struct BasicEnvironmentAdapter {
    environment: Environment,
    hmr_config: HMRConfig,
}

impl BasicEnvironmentAdapter {
    pub fn new(environment: Environment, hmr_config: HMRConfig) -> Self {
        Self {
            environment,
            hmr_config,
        }
    }
}

impl EnvironmentAdapter for BasicEnvironmentAdapter {
    fn configure_for_environment(&mut self, env: Environment) -> Result<(), ConfigError> {
        self.environment = env.clone();
        
        // Adjust HMR config based on environment
        match &env {
            Environment::Development { auto_reload, .. } => {
                self.hmr_config.update_policy = if *auto_reload {
                    UpdatePolicy::Automatic
                } else {
                    UpdatePolicy::Manual
                };
                self.hmr_config.validation_level = ValidationLevel::Standard;
            },
            Environment::Production { require_explicit_deployment, .. } => {
                self.hmr_config.update_policy = if *require_explicit_deployment {
                    UpdatePolicy::Manual
                } else {
                    UpdatePolicy::Consensus
                };
                self.hmr_config.validation_level = ValidationLevel::Extensive;
            },
            Environment::Testing { .. } => {
                self.hmr_config.update_policy = UpdatePolicy::Manual;
                self.hmr_config.validation_level = ValidationLevel::Paranoid;
            },
        }
        
        Ok(())
    }
    
    fn validate_update_safety(&self, update: &UpdatePlan) -> SafetyAssessment {
        let risk_level = self.assess_risk_level(update);
        let required_validations = self.get_required_validations(&risk_level);
        let recommended_strategy = self.get_deployment_strategy(&risk_level);
        
        SafetyAssessment {
            risk_level,
            required_validations,
            recommended_strategy,
            environment_constraints: self.get_environment_constraints(),
            estimated_downtime: self.estimate_downtime(update),
            rollback_feasibility: self.assess_rollback_feasibility(update),
        }
    }
    
    fn should_auto_reload(&self, change: &ChangeAnalysis) -> bool {
        match &self.environment {
            Environment::Development { auto_reload, .. } => {
                *auto_reload && self.is_safe_for_auto_reload(change)
            },
            Environment::Production { .. } => false,
            Environment::Testing { .. } => false,
        }
    }
    
    fn get_rollback_timeout(&self) -> Duration {
        match &self.environment {
            Environment::Development { .. } => Duration::from_secs(30),
            Environment::Production { .. } => Duration::from_secs(300), // 5 minutes
            Environment::Testing { .. } => Duration::from_secs(60),
        }
    }
    
    fn recommend_deployment_strategy(&self, update: &UpdatePlan) -> DeploymentStrategy {
        let risk_level = self.assess_risk_level(update);
        self.get_deployment_strategy(&risk_level)
    }
}

impl BasicEnvironmentAdapter {
    fn assess_risk_level(&self, update: &UpdatePlan) -> RiskLevel {
        // Analyze the update to determine risk level
        let has_breaking_changes = update.steps.iter().any(|step| {
            matches!(step.step_type, UpdateStepType::UpdateCode) && 
            step.description.contains("breaking")
        });
        
        let affects_core_runtime = update.steps.iter().any(|step| {
            step.description.contains("runtime") || step.description.contains("coordinator")
        });
        
        let has_state_migration = update.steps.iter().any(|step| {
            matches!(step.step_type, UpdateStepType::MigrateState)
        });
        
        match (&self.environment, has_breaking_changes, affects_core_runtime, has_state_migration) {
            (Environment::Production { .. }, true, _, _) => RiskLevel::High,
            (Environment::Production { .. }, _, true, _) => RiskLevel::Critical,
            (_, _, true, _) => RiskLevel::High,
            (_, true, _, _) => RiskLevel::Medium,
            (_, _, _, true) => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }
    
    fn get_required_validations(&self, risk_level: &RiskLevel) -> Vec<ValidationCheck> {
        let mut validations = vec![
            ValidationCheck {
                name: "Syntax Check".to_string(),
                description: "Validate code syntax and basic structure".to_string(),
                timeout: Duration::from_secs(10),
                required: true,
            },
            ValidationCheck {
                name: "Type Check".to_string(),
                description: "Validate type compatibility".to_string(),
                timeout: Duration::from_secs(30),
                required: true,
            },
        ];
        
        match risk_level {
            RiskLevel::Low => {},
            RiskLevel::Medium => {
                validations.push(ValidationCheck {
                    name: "Integration Test".to_string(),
                    description: "Run integration tests".to_string(),
                    timeout: Duration::from_secs(120),
                    required: true,
                });
            },
            RiskLevel::High => {
                validations.push(ValidationCheck {
                    name: "Integration Test".to_string(),
                    description: "Run integration tests".to_string(),
                    timeout: Duration::from_secs(120),
                    required: true,
                });
                validations.push(ValidationCheck {
                    name: "Performance Test".to_string(),
                    description: "Validate performance impact".to_string(),
                    timeout: Duration::from_secs(300),
                    required: true,
                });
            },
            RiskLevel::Critical => {
                validations.push(ValidationCheck {
                    name: "Integration Test".to_string(),
                    description: "Run integration tests".to_string(),
                    timeout: Duration::from_secs(120),
                    required: true,
                });
                validations.push(ValidationCheck {
                    name: "Performance Test".to_string(),
                    description: "Validate performance impact".to_string(),
                    timeout: Duration::from_secs(300),
                    required: true,
                });
                validations.push(ValidationCheck {
                    name: "Security Audit".to_string(),
                    description: "Perform security validation".to_string(),
                    timeout: Duration::from_secs(600),
                    required: true,
                });
                validations.push(ValidationCheck {
                    name: "Formal Verification".to_string(),
                    description: "Formal verification of critical paths".to_string(),
                    timeout: Duration::from_secs(1800),
                    required: false,
                });
            },
        }
        
        validations
    }
    
    fn get_deployment_strategy(&self, risk_level: &RiskLevel) -> DeploymentStrategy {
        match (&self.environment, risk_level) {
            (Environment::Development { .. }, _) => DeploymentStrategy::Direct,
            (Environment::Testing { .. }, _) => DeploymentStrategy::Direct,
            (Environment::Production { staged_rollout: true, canary_percentage, .. }, RiskLevel::Low) => {
                DeploymentStrategy::Canary {
                    percentage: *canary_percentage,
                    duration: Duration::from_secs(300),
                }
            },
            (Environment::Production { staged_rollout: true, .. }, _) => {
                DeploymentStrategy::BlueGreen {
                    validation_duration: Duration::from_secs(600),
                }
            },
            (Environment::Production { .. }, RiskLevel::Critical) => {
                DeploymentStrategy::Manual {
                    approval_required: true,
                    validation_steps: self.get_required_validations(risk_level),
                }
            },
            (Environment::Production { .. }, _) => {
                DeploymentStrategy::Rolling {
                    batch_size: 1,
                    delay_between_batches: Duration::from_secs(60),
                }
            },
        }
    }
    
    fn get_environment_constraints(&self) -> Vec<EnvironmentConstraint> {
        match &self.environment {
            Environment::Development { .. } => vec![
                EnvironmentConstraint {
                    name: "Fast Iteration".to_string(),
                    description: "Prioritize speed over safety".to_string(),
                    constraint_type: ConstraintType::Performance,
                },
            ],
            Environment::Production { .. } => vec![
                EnvironmentConstraint {
                    name: "Zero Downtime".to_string(),
                    description: "Must maintain service availability".to_string(),
                    constraint_type: ConstraintType::Availability,
                },
                EnvironmentConstraint {
                    name: "Data Integrity".to_string(),
                    description: "Must preserve all data during updates".to_string(),
                    constraint_type: ConstraintType::Safety,
                },
            ],
            Environment::Testing { deterministic_execution, .. } => {
                let mut constraints = vec![
                    EnvironmentConstraint {
                        name: "Reproducibility".to_string(),
                        description: "Updates must be reproducible".to_string(),
                        constraint_type: ConstraintType::Reliability,
                    },
                ];
                
                if *deterministic_execution {
                    constraints.push(EnvironmentConstraint {
                        name: "Determinism".to_string(),
                        description: "Execution must be deterministic".to_string(),
                        constraint_type: ConstraintType::Reliability,
                    });
                }
                
                constraints
            },
        }
    }
    
    fn estimate_downtime(&self, update: &UpdatePlan) -> Duration {
        let base_time = Duration::from_secs(5); // Base overhead
        let per_step_time = Duration::from_secs(2);
        let total_steps_time = per_step_time * update.steps.len() as u32;
        
        let complexity_multiplier = match self.assess_risk_level(update) {
            RiskLevel::Low => 1.0,
            RiskLevel::Medium => 1.5,
            RiskLevel::High => 2.0,
            RiskLevel::Critical => 3.0,
        };
        
        let estimated = base_time + total_steps_time;
        Duration::from_secs((estimated.as_secs() as f64 * complexity_multiplier) as u64)
    }
    
    fn assess_rollback_feasibility(&self, update: &UpdatePlan) -> RollbackFeasibility {
        let has_irreversible_changes = update.steps.iter().any(|step| {
            matches!(step.step_type, UpdateStepType::MigrateState) &&
            step.description.contains("irreversible")
        });
        
        if has_irreversible_changes {
            RollbackFeasibility::Difficult {
                reason: "Contains irreversible data migrations".to_string(),
                estimated_time: Duration::from_secs(1800), // 30 minutes
            }
        } else {
            RollbackFeasibility::Easy {
                estimated_time: Duration::from_secs(60),
            }
        }
    }
    
    fn is_safe_for_auto_reload(&self, change: &ChangeAnalysis) -> bool {
        // Only allow auto-reload for low-risk changes
        change.compatibility == CompatibilityLevel::FullyCompatible &&
        !change.schema_changes.iter().any(|sc| matches!(sc.change_type, SchemaChangeType::Breaking))
    }
}

/// Safety assessment for update risk evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAssessment {
    pub risk_level: RiskLevel,
    pub required_validations: Vec<ValidationCheck>,
    pub recommended_strategy: DeploymentStrategy,
    pub environment_constraints: Vec<EnvironmentConstraint>,
    pub estimated_downtime: Duration,
    pub rollback_feasibility: RollbackFeasibility,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Pure function changes, documentation updates
    Low,
    /// Schema-compatible changes, new features
    Medium,
    /// Breaking changes, state migrations
    High,
    /// Core runtime changes, distributed coordination
    Critical,
}

/// Validation check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub description: String,
    pub timeout: Duration,
    pub required: bool,
}

/// Deployment strategy recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    /// Direct deployment without staging
    Direct,
    /// Canary deployment with percentage rollout
    Canary {
        percentage: f32,
        duration: Duration,
    },
    /// Blue-green deployment
    BlueGreen {
        validation_duration: Duration,
    },
    /// Rolling deployment
    Rolling {
        batch_size: usize,
        delay_between_batches: Duration,
    },
    /// Manual deployment with approvals
    Manual {
        approval_required: bool,
        validation_steps: Vec<ValidationCheck>,
    },
}

/// Environment constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConstraint {
    pub name: String,
    pub description: String,
    pub constraint_type: ConstraintType,
}

/// Type of environment constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Performance,
    Availability,
    Safety,
    Reliability,
    Security,
}

/// Rollback feasibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackFeasibility {
    Easy {
        estimated_time: Duration,
    },
    Difficult {
        reason: String,
        estimated_time: Duration,
    },
    Impossible {
        reason: String,
    },
}

/// Configuration error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    InvalidEnvironment(String),
    InvalidPolicy(String),
    InvalidValidationLevel(String),
    InvalidTimeout(String),
    MissingConfiguration(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidEnvironment(msg) => write!(f, "Invalid environment: {msg}"),
            ConfigError::InvalidPolicy(msg) => write!(f, "Invalid policy: {msg}"),
            ConfigError::InvalidValidationLevel(msg) => write!(f, "Invalid validation level: {msg}"),
            ConfigError::InvalidTimeout(msg) => write!(f, "Invalid timeout: {msg}"),
            ConfigError::MissingConfiguration(msg) => write!(f, "Missing configuration: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

// Import required types for the adapter implementation
use crate::change_analysis::{ChangeAnalysis, CompatibilityLevel, SchemaChangeType};
use crate::hmr_coordinator::{UpdatePlan, UpdateStepType};