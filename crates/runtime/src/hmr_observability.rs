//! HMR Observability System
//!
//! This module provides comprehensive observability for Hot-Module-Reloading processes,
//! including detailed telemetry, state migration tracking, error reporting, and
//! cluster-wide update status visibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

use crate::crdt::NodeId;
use crate::hmr_coordinator::{RollbackPlan, UpdateResult};
use crate::logging::{
    CounterId, GaugeId, HistogramId, LogContext, Logger, LoggingSystem, MetricsRegistry,
};

/// Errors that can occur in HMR observability
#[derive(Error, Debug)]
pub enum HMRObservabilityError {
    #[error("Failed to record HMR telemetry: {0}")]
    TelemetryError(String),
    #[error("Failed to track state migration: {0}")]
    MigrationTrackingError(String),
    #[error("Failed to report error: {0}")]
    ErrorReportingError(String),
    #[error("Failed to update cluster status: {0}")]
    ClusterStatusError(String),
    #[error("Process not found: {0}")]
    ProcessNotFound(String),
}

pub type HMRObservabilityResult<T> = Result<T, HMRObservabilityError>;

/// Unique identifier for HMR update processes
pub type UpdateProcessId = Uuid;

/// HMR process types for telemetry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HMRProcessType {
    ChangeDetection,
    DependencyAnalysis,
    CompatibilityCheck,
    StateExtraction,
    StateMigration,
    ModuleUpdate,
    StateRestoration,
    ValidationCheck,
    RollbackExecution,
    ClusterCoordination,
}

/// HMR process status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HMRProcessStatus {
    Started,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    RolledBack,
}

/// Detailed HMR process telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMRProcessTelemetry {
    pub process_id: UpdateProcessId,
    pub process_type: HMRProcessType,
    pub status: HMRProcessStatus,
    pub node_id: NodeId,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub affected_modules: Vec<String>,
    pub dependencies_analyzed: usize,
    pub state_migrations_performed: usize,
    pub validation_checks_passed: usize,
    pub validation_checks_failed: usize,
    pub error_details: Option<HMRErrorDetails>,
    pub performance_metrics: HMRPerformanceMetrics,
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

/// Performance metrics for HMR processes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct HMRPerformanceMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_bytes: Option<u64>,
    pub network_bytes_sent: Option<u64>,
    pub network_bytes_received: Option<u64>,
    pub disk_reads: Option<u64>,
    pub disk_writes: Option<u64>,
    pub cache_hits: Option<u64>,
    pub cache_misses: Option<u64>,
}


/// State migration tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMigrationTracking {
    pub migration_id: Uuid,
    pub process_id: UpdateProcessId,
    pub source_schema_hash: String,
    pub target_schema_hash: String,
    pub migration_type: MigrationType,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub records_processed: u64,
    pub records_migrated: u64,
    pub records_failed: u64,
    pub transformation_steps: Vec<TransformationStep>,
    pub error_details: Option<MigrationErrorDetails>,
}

/// Types of state migrations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationType {
    Automatic,
    UserDefined,
    BackwardCompatible,
    Breaking,
}

/// Individual transformation step in migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationStep {
    pub step_id: Uuid,
    pub step_name: String,
    pub step_type: TransformationStepType,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub records_processed: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Types of transformation steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransformationStepType {
    FieldAddition,
    FieldRemoval,
    FieldRename,
    FieldTransformation,
    DataValidation,
    IndexRebuild,
    CustomTransformation,
}

/// Migration error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub failed_record_ids: Vec<String>,
    pub recovery_suggestions: Vec<String>,
    pub rollback_available: bool,
}

/// Comprehensive error reporting for HMR processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMRErrorDetails {
    pub error_id: Uuid,
    pub error_type: HMRErrorType,
    pub error_message: String,
    pub error_code: Option<String>,
    pub stack_trace: Vec<StackFrame>,
    pub affected_components: Vec<String>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub related_errors: Vec<Uuid>,
    pub context: HashMap<String, serde_json::Value>,
}

/// Types of HMR errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HMRErrorType {
    CompatibilityError,
    MigrationError,
    ValidationError,
    NetworkError,
    ConsensusError,
    StateExtractionError,
    StateRestorationError,
    RollbackError,
    TimeoutError,
    ResourceError,
}

/// Stack frame for error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub module_hash: Option<String>,
}

/// Recovery actions for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub action_type: RecoveryActionType,
    pub description: String,
    pub automatic: bool,
    pub estimated_time: Option<Duration>,
}

/// Types of recovery actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryActionType {
    Retry,
    Rollback,
    SkipComponent,
    ManualIntervention,
    RestartProcess,
    FallbackMode,
}

/// Cluster-wide update status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterUpdateStatus {
    pub update_id: UpdateProcessId,
    pub overall_status: ClusterUpdateOverallStatus,
    pub start_time: SystemTime,
    pub estimated_completion: Option<SystemTime>,
    pub progress_percentage: f64,
    pub node_statuses: HashMap<NodeId, NodeUpdateStatus>,
    pub consensus_status: ConsensusStatus,
    pub rollback_plan: Option<RollbackPlan>,
    pub health_checks: Vec<HealthCheckResult>,
}

/// Overall cluster update status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClusterUpdateOverallStatus {
    Planning,
    Coordinating,
    Executing,
    Validating,
    Completing,
    Completed,
    Failed,
    RollingBack,
    RolledBack,
}

/// Status of individual nodes in cluster update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUpdateStatus {
    pub node_id: NodeId,
    pub status: NodeUpdateState,
    pub progress_percentage: f64,
    pub current_step: Option<String>,
    pub last_heartbeat: SystemTime,
    pub error_details: Option<HMRErrorDetails>,
    pub performance_metrics: HMRPerformanceMetrics,
}

/// Individual node update states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeUpdateState {
    Waiting,
    Preparing,
    Updating,
    Validating,
    Completed,
    Failed,
    RollingBack,
    RolledBack,
}

/// Consensus status for distributed updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStatus {
    pub consensus_reached: bool,
    pub participating_nodes: Vec<NodeId>,
    pub votes_received: HashMap<NodeId, ConsensusVote>,
    pub consensus_round: u64,
    pub timeout_remaining: Option<Duration>,
}

/// Consensus vote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    pub node_id: NodeId,
    pub vote: bool,
    pub timestamp: SystemTime,
    pub reasoning: Option<String>,
}

/// Health check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_name: String,
    pub status: HealthCheckStatus,
    pub timestamp: SystemTime,
    pub details: Option<String>,
    pub metrics: HashMap<String, f64>,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthCheckStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// HMR observability system trait
pub trait HMRObservabilitySystem: Send + Sync {
    /// Start tracking an HMR process
    fn start_process_tracking(
        &mut self,
        process_type: HMRProcessType,
        node_id: NodeId,
    ) -> HMRObservabilityResult<UpdateProcessId>;

    /// Update process status
    fn update_process_status(
        &mut self,
        process_id: UpdateProcessId,
        status: HMRProcessStatus,
    ) -> HMRObservabilityResult<()>;

    /// Record process completion
    fn complete_process(
        &mut self,
        process_id: UpdateProcessId,
        result: &UpdateResult,
    ) -> HMRObservabilityResult<()>;

    /// Record process failure
    fn fail_process(
        &mut self,
        process_id: UpdateProcessId,
        error: HMRErrorDetails,
    ) -> HMRObservabilityResult<()>;

    /// Track state migration
    fn track_state_migration(
        &mut self,
        migration: StateMigrationTracking,
    ) -> HMRObservabilityResult<()>;

    /// Record transformation step
    fn record_transformation_step(
        &mut self,
        migration_id: Uuid,
        step: TransformationStep,
    ) -> HMRObservabilityResult<()>;

    /// Report HMR error with stack trace
    fn report_error(&mut self, error: HMRErrorDetails) -> HMRObservabilityResult<()>;

    /// Update cluster-wide status
    fn update_cluster_status(&mut self, status: ClusterUpdateStatus) -> HMRObservabilityResult<()>;

    /// Get process telemetry
    fn get_process_telemetry(
        &self,
        process_id: UpdateProcessId,
    ) -> HMRObservabilityResult<HMRProcessTelemetry>;

    /// Get cluster status
    fn get_cluster_status(
        &self,
        update_id: UpdateProcessId,
    ) -> HMRObservabilityResult<ClusterUpdateStatus>;

    /// Get migration tracking info
    fn get_migration_tracking(
        &self,
        migration_id: Uuid,
    ) -> HMRObservabilityResult<StateMigrationTracking>;

    /// List active processes
    fn list_active_processes(&self) -> Vec<UpdateProcessId>;

    /// Get error history
    fn get_error_history(&self, limit: Option<usize>) -> Vec<HMRErrorDetails>;
}

/// Default implementation of HMR observability system
pub struct DefaultHMRObservabilitySystem {
    process_telemetry: Arc<Mutex<HashMap<UpdateProcessId, HMRProcessTelemetry>>>,
    migration_tracking: Arc<Mutex<HashMap<Uuid, StateMigrationTracking>>>,
    cluster_status: Arc<Mutex<HashMap<UpdateProcessId, ClusterUpdateStatus>>>,
    error_history: Arc<Mutex<Vec<HMRErrorDetails>>>,
    logging_system: Arc<LoggingSystem>,

    // Metrics
    process_counter: CounterId,
    migration_counter: CounterId,
    error_counter: CounterId,
    process_duration_histogram: HistogramId,
    migration_duration_histogram: HistogramId,
    cluster_health_gauge: GaugeId,
}

impl DefaultHMRObservabilitySystem {
    pub fn new(logging_system: Arc<LoggingSystem>) -> HMRObservabilityResult<Self> {
        let registry = logging_system.get_metrics_registry();
        let mut metrics_registry = registry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire metrics registry: {e}"
            ))
        })?;

        let process_counter = metrics_registry.create_counter(
            "hmr_processes_total".to_string(),
            "Total number of HMR processes started".to_string(),
        );

        let migration_counter = metrics_registry.create_counter(
            "hmr_migrations_total".to_string(),
            "Total number of state migrations performed".to_string(),
        );

        let error_counter = metrics_registry.create_counter(
            "hmr_errors_total".to_string(),
            "Total number of HMR errors encountered".to_string(),
        );

        let process_duration_histogram = metrics_registry.create_histogram(
            "hmr_process_duration_seconds".to_string(),
            "Duration of HMR processes in seconds".to_string(),
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 300.0],
        );

        let migration_duration_histogram = metrics_registry.create_histogram(
            "hmr_migration_duration_seconds".to_string(),
            "Duration of state migrations in seconds".to_string(),
            vec![0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0],
        );

        let cluster_health_gauge = metrics_registry.create_gauge(
            "hmr_cluster_health_score".to_string(),
            "Overall cluster health score (0-1)".to_string(),
        );

        Ok(Self {
            process_telemetry: Arc::new(Mutex::new(HashMap::new())),
            migration_tracking: Arc::new(Mutex::new(HashMap::new())),
            cluster_status: Arc::new(Mutex::new(HashMap::new())),
            error_history: Arc::new(Mutex::new(Vec::new())),
            logging_system,
            process_counter,
            migration_counter,
            error_counter,
            process_duration_histogram,
            migration_duration_histogram,
            cluster_health_gauge,
        })
    }

    fn log_with_context(&self, context: LogContext) -> Box<dyn Logger> {
        self.logging_system.with_context(context)
    }

    fn record_process_metric(&self, process_type: &HMRProcessType) -> HMRObservabilityResult<()> {
        let registry = self.logging_system.get_metrics_registry();
        let mut metrics_registry = registry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire metrics registry: {e}"
            ))
        })?;

        let mut labels = HashMap::new();
        labels.insert("process_type".to_string(), format!("{process_type:?}"));

        metrics_registry
            .increment_counter(self.process_counter, 1.0, &labels)
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!(
                    "Failed to increment process counter: {e}"
                ))
            })
    }

    fn record_migration_metric(
        &self,
        migration_type: &MigrationType,
    ) -> HMRObservabilityResult<()> {
        let registry = self.logging_system.get_metrics_registry();
        let mut metrics_registry = registry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire metrics registry: {e}"
            ))
        })?;

        let mut labels = HashMap::new();
        labels.insert(
            "migration_type".to_string(),
            format!("{migration_type:?}"),
        );

        metrics_registry
            .increment_counter(self.migration_counter, 1.0, &labels)
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!(
                    "Failed to increment migration counter: {e}"
                ))
            })
    }

    fn record_error_metric(&self, error_type: &HMRErrorType) -> HMRObservabilityResult<()> {
        let registry = self.logging_system.get_metrics_registry();
        let mut metrics_registry = registry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire metrics registry: {e}"
            ))
        })?;

        let mut labels = HashMap::new();
        labels.insert("error_type".to_string(), format!("{error_type:?}"));

        metrics_registry
            .increment_counter(self.error_counter, 1.0, &labels)
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!(
                    "Failed to increment error counter: {e}"
                ))
            })
    }

    fn record_duration_metric(
        &self,
        histogram_id: HistogramId,
        duration: Duration,
        labels: &HashMap<String, String>,
    ) -> HMRObservabilityResult<()> {
        let registry = self.logging_system.get_metrics_registry();
        let mut metrics_registry = registry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire metrics registry: {e}"
            ))
        })?;

        metrics_registry
            .record_histogram(histogram_id, duration.as_secs_f64(), labels)
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!("Failed to record duration: {e}"))
            })
    }

    fn update_cluster_health(&self, health_score: f64) -> HMRObservabilityResult<()> {
        let registry = self.logging_system.get_metrics_registry();
        let mut metrics_registry = registry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire metrics registry: {e}"
            ))
        })?;

        metrics_registry
            .set_gauge(self.cluster_health_gauge, health_score, &HashMap::new())
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!(
                    "Failed to update cluster health: {e}"
                ))
            })
    }
}

impl HMRObservabilitySystem for DefaultHMRObservabilitySystem {
    fn start_process_tracking(
        &mut self,
        process_type: HMRProcessType,
        node_id: NodeId,
    ) -> HMRObservabilityResult<UpdateProcessId> {
        let process_id = Uuid::new_v4();
        let start_time = SystemTime::now();

        let telemetry = HMRProcessTelemetry {
            process_id,
            process_type: process_type.clone(),
            status: HMRProcessStatus::Started,
            node_id,
            start_time,
            end_time: None,
            duration: None,
            affected_modules: Vec::new(),
            dependencies_analyzed: 0,
            state_migrations_performed: 0,
            validation_checks_passed: 0,
            validation_checks_failed: 0,
            error_details: None,
            performance_metrics: HMRPerformanceMetrics::default(),
            custom_attributes: HashMap::new(),
        };

        // Store telemetry
        self.process_telemetry
            .lock()
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!(
                    "Failed to acquire telemetry lock: {e}"
                ))
            })?
            .insert(process_id, telemetry);

        // Record metrics
        self.record_process_metric(&process_type)?;

        // Log process start
        let context = LogContext::new()
            .with_node_id(node_id.to_string())
            .with_field(
                "process_id".to_string(),
                serde_json::Value::String(process_id.to_string()),
            )
            .with_field(
                "process_type".to_string(),
                serde_json::Value::String(format!("{process_type:?}")),
            );

        let logger = self.log_with_context(context);
        let mut fields = HashMap::new();
        fields.insert(
            "process_id".to_string(),
            serde_json::Value::String(process_id.to_string()),
        );
        fields.insert(
            "process_type".to_string(),
            serde_json::Value::String(format!("{process_type:?}")),
        );

        logger.info("HMR process started", &fields);

        Ok(process_id)
    }

    fn update_process_status(
        &mut self,
        process_id: UpdateProcessId,
        status: HMRProcessStatus,
    ) -> HMRObservabilityResult<()> {
        let mut telemetry_map = self.process_telemetry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire telemetry lock: {e}"
            ))
        })?;

        if let Some(telemetry) = telemetry_map.get_mut(&process_id) {
            telemetry.status = status.clone();

            // Log status update
            let context = LogContext::new()
                .with_node_id(telemetry.node_id.to_string())
                .with_field(
                    "process_id".to_string(),
                    serde_json::Value::String(process_id.to_string()),
                );

            let logger = self.log_with_context(context);
            let mut fields = HashMap::new();
            fields.insert(
                "process_id".to_string(),
                serde_json::Value::String(process_id.to_string()),
            );
            fields.insert(
                "status".to_string(),
                serde_json::Value::String(format!("{status:?}")),
            );

            logger.info("HMR process status updated", &fields);

            Ok(())
        } else {
            Err(HMRObservabilityError::ProcessNotFound(
                process_id.to_string(),
            ))
        }
    }

    fn complete_process(
        &mut self,
        process_id: UpdateProcessId,
        _result: &UpdateResult,
    ) -> HMRObservabilityResult<()> {
        let end_time = SystemTime::now();

        let mut telemetry_map = self.process_telemetry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire telemetry lock: {e}"
            ))
        })?;

        if let Some(telemetry) = telemetry_map.get_mut(&process_id) {
            telemetry.status = HMRProcessStatus::Completed;
            telemetry.end_time = Some(end_time);
            telemetry.duration = end_time.duration_since(telemetry.start_time).ok();

            // Record duration metric
            if let Some(duration) = telemetry.duration {
                let mut labels = HashMap::new();
                labels.insert(
                    "process_type".to_string(),
                    format!("{:?}", telemetry.process_type),
                );
                labels.insert("status".to_string(), "completed".to_string());

                self.record_duration_metric(self.process_duration_histogram, duration, &labels)?;
            }

            // Log completion
            let context = LogContext::new()
                .with_node_id(telemetry.node_id.to_string())
                .with_field(
                    "process_id".to_string(),
                    serde_json::Value::String(process_id.to_string()),
                );

            let logger = self.log_with_context(context);
            let mut fields = HashMap::new();
            fields.insert(
                "process_id".to_string(),
                serde_json::Value::String(process_id.to_string()),
            );
            fields.insert(
                "duration_ms".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(
                        telemetry.duration.unwrap_or_default().as_millis() as f64,
                    )
                    .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );

            logger.info("HMR process completed successfully", &fields);

            Ok(())
        } else {
            Err(HMRObservabilityError::ProcessNotFound(
                process_id.to_string(),
            ))
        }
    }

    fn fail_process(
        &mut self,
        process_id: UpdateProcessId,
        error: HMRErrorDetails,
    ) -> HMRObservabilityResult<()> {
        let end_time = SystemTime::now();

        let mut telemetry_map = self.process_telemetry.lock().map_err(|e| {
            HMRObservabilityError::TelemetryError(format!(
                "Failed to acquire telemetry lock: {e}"
            ))
        })?;

        if let Some(telemetry) = telemetry_map.get_mut(&process_id) {
            telemetry.status = HMRProcessStatus::Failed;
            telemetry.end_time = Some(end_time);
            telemetry.duration = end_time.duration_since(telemetry.start_time).ok();
            telemetry.error_details = Some(error.clone());

            // Record error metric
            self.record_error_metric(&error.error_type)?;

            // Record duration metric
            if let Some(duration) = telemetry.duration {
                let mut labels = HashMap::new();
                labels.insert(
                    "process_type".to_string(),
                    format!("{:?}", telemetry.process_type),
                );
                labels.insert("status".to_string(), "failed".to_string());

                self.record_duration_metric(self.process_duration_histogram, duration, &labels)?;
            }

            // Add to error history
            self.error_history
                .lock()
                .map_err(|e| {
                    HMRObservabilityError::ErrorReportingError(format!(
                        "Failed to acquire error history lock: {e}"
                    ))
                })?
                .push(error.clone());

            // Log failure
            let context = LogContext::new()
                .with_node_id(telemetry.node_id.to_string())
                .with_field(
                    "process_id".to_string(),
                    serde_json::Value::String(process_id.to_string()),
                );

            let logger = self.log_with_context(context);
            let mut fields = HashMap::new();
            fields.insert(
                "process_id".to_string(),
                serde_json::Value::String(process_id.to_string()),
            );
            fields.insert(
                "error_type".to_string(),
                serde_json::Value::String(format!("{:?}", error.error_type)),
            );
            fields.insert(
                "error_message".to_string(),
                serde_json::Value::String(error.error_message.clone()),
            );

            logger.error("HMR process failed", &fields);

            Ok(())
        } else {
            Err(HMRObservabilityError::ProcessNotFound(
                process_id.to_string(),
            ))
        }
    }

    fn track_state_migration(
        &mut self,
        migration: StateMigrationTracking,
    ) -> HMRObservabilityResult<()> {
        // Record migration metric
        self.record_migration_metric(&migration.migration_type)?;

        // Record duration if completed
        if let Some(duration) = migration.duration {
            let mut labels = HashMap::new();
            labels.insert(
                "migration_type".to_string(),
                format!("{:?}", migration.migration_type),
            );

            self.record_duration_metric(self.migration_duration_histogram, duration, &labels)?;
        }

        // Store migration tracking
        self.migration_tracking
            .lock()
            .map_err(|e| {
                HMRObservabilityError::MigrationTrackingError(format!(
                    "Failed to acquire migration tracking lock: {e}"
                ))
            })?
            .insert(migration.migration_id, migration.clone());

        // Log migration
        let context = LogContext::new()
            .with_field(
                "migration_id".to_string(),
                serde_json::Value::String(migration.migration_id.to_string()),
            )
            .with_field(
                "process_id".to_string(),
                serde_json::Value::String(migration.process_id.to_string()),
            );

        let logger = self.log_with_context(context);
        let mut fields = HashMap::new();
        fields.insert(
            "migration_id".to_string(),
            serde_json::Value::String(migration.migration_id.to_string()),
        );
        fields.insert(
            "migration_type".to_string(),
            serde_json::Value::String(format!("{:?}", migration.migration_type)),
        );
        fields.insert(
            "records_processed".to_string(),
            serde_json::Value::Number(serde_json::Number::from(migration.records_processed)),
        );
        fields.insert(
            "records_migrated".to_string(),
            serde_json::Value::Number(serde_json::Number::from(migration.records_migrated)),
        );

        logger.info("State migration tracked", &fields);

        Ok(())
    }

    fn record_transformation_step(
        &mut self,
        migration_id: Uuid,
        step: TransformationStep,
    ) -> HMRObservabilityResult<()> {
        let mut migration_map = self.migration_tracking.lock().map_err(|e| {
            HMRObservabilityError::MigrationTrackingError(format!(
                "Failed to acquire migration tracking lock: {e}"
            ))
        })?;

        if let Some(migration) = migration_map.get_mut(&migration_id) {
            migration.transformation_steps.push(step.clone());

            // Log transformation step
            let context = LogContext::new()
                .with_field(
                    "migration_id".to_string(),
                    serde_json::Value::String(migration_id.to_string()),
                )
                .with_field(
                    "step_id".to_string(),
                    serde_json::Value::String(step.step_id.to_string()),
                );

            let logger = self.log_with_context(context);
            let mut fields = HashMap::new();
            fields.insert(
                "step_name".to_string(),
                serde_json::Value::String(step.step_name.clone()),
            );
            fields.insert(
                "step_type".to_string(),
                serde_json::Value::String(format!("{:?}", step.step_type)),
            );
            fields.insert("success".to_string(), serde_json::Value::Bool(step.success));
            fields.insert(
                "records_processed".to_string(),
                serde_json::Value::Number(serde_json::Number::from(step.records_processed)),
            );

            if step.success {
                logger.info("Transformation step completed", &fields);
            } else {
                fields.insert(
                    "error_message".to_string(),
                    serde_json::Value::String(step.error_message.unwrap_or_default()),
                );
                logger.error("Transformation step failed", &fields);
            }

            Ok(())
        } else {
            Err(HMRObservabilityError::MigrationTrackingError(format!(
                "Migration {migration_id} not found"
            )))
        }
    }

    fn report_error(&mut self, error: HMRErrorDetails) -> HMRObservabilityResult<()> {
        // Record error metric
        self.record_error_metric(&error.error_type)?;

        // Add to error history
        self.error_history
            .lock()
            .map_err(|e| {
                HMRObservabilityError::ErrorReportingError(format!(
                    "Failed to acquire error history lock: {e}"
                ))
            })?
            .push(error.clone());

        // Log error with full details
        let context = LogContext::new().with_field(
            "error_id".to_string(),
            serde_json::Value::String(error.error_id.to_string()),
        );

        let logger = self.log_with_context(context);
        let mut fields = HashMap::new();
        fields.insert(
            "error_type".to_string(),
            serde_json::Value::String(format!("{:?}", error.error_type)),
        );
        fields.insert(
            "error_message".to_string(),
            serde_json::Value::String(error.error_message.clone()),
        );
        fields.insert(
            "affected_components".to_string(),
            serde_json::Value::Array(
                error
                    .affected_components
                    .iter()
                    .map(|c| serde_json::Value::String(c.clone()))
                    .collect(),
            ),
        );
        fields.insert(
            "stack_trace".to_string(),
            serde_json::to_value(&error.stack_trace).unwrap_or_default(),
        );

        logger.error("HMR error reported", &fields);

        Ok(())
    }

    fn update_cluster_status(&mut self, status: ClusterUpdateStatus) -> HMRObservabilityResult<()> {
        // Calculate cluster health score
        let health_score = self.calculate_cluster_health(&status);
        self.update_cluster_health(health_score)?;

        // Store cluster status
        self.cluster_status
            .lock()
            .map_err(|e| {
                HMRObservabilityError::ClusterStatusError(format!(
                    "Failed to acquire cluster status lock: {e}"
                ))
            })?
            .insert(status.update_id, status.clone());

        // Log cluster status update
        let context = LogContext::new().with_field(
            "update_id".to_string(),
            serde_json::Value::String(status.update_id.to_string()),
        );

        let logger = self.log_with_context(context);
        let mut fields = HashMap::new();
        fields.insert(
            "overall_status".to_string(),
            serde_json::Value::String(format!("{:?}", status.overall_status)),
        );
        fields.insert(
            "progress_percentage".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(status.progress_percentage)
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        fields.insert(
            "node_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(status.node_statuses.len())),
        );
        fields.insert(
            "consensus_reached".to_string(),
            serde_json::Value::Bool(status.consensus_status.consensus_reached),
        );

        logger.info("Cluster update status updated", &fields);

        Ok(())
    }

    fn get_process_telemetry(
        &self,
        process_id: UpdateProcessId,
    ) -> HMRObservabilityResult<HMRProcessTelemetry> {
        self.process_telemetry
            .lock()
            .map_err(|e| {
                HMRObservabilityError::TelemetryError(format!(
                    "Failed to acquire telemetry lock: {e}"
                ))
            })?
            .get(&process_id)
            .cloned()
            .ok_or_else(|| HMRObservabilityError::ProcessNotFound(process_id.to_string()))
    }

    fn get_cluster_status(
        &self,
        update_id: UpdateProcessId,
    ) -> HMRObservabilityResult<ClusterUpdateStatus> {
        self.cluster_status
            .lock()
            .map_err(|e| {
                HMRObservabilityError::ClusterStatusError(format!(
                    "Failed to acquire cluster status lock: {e}"
                ))
            })?
            .get(&update_id)
            .cloned()
            .ok_or_else(|| HMRObservabilityError::ProcessNotFound(update_id.to_string()))
    }

    fn get_migration_tracking(
        &self,
        migration_id: Uuid,
    ) -> HMRObservabilityResult<StateMigrationTracking> {
        self.migration_tracking
            .lock()
            .map_err(|e| {
                HMRObservabilityError::MigrationTrackingError(format!(
                    "Failed to acquire migration tracking lock: {e}"
                ))
            })?
            .get(&migration_id)
            .cloned()
            .ok_or_else(|| {
                HMRObservabilityError::MigrationTrackingError(format!(
                    "Migration {migration_id} not found"
                ))
            })
    }

    fn list_active_processes(&self) -> Vec<UpdateProcessId> {
        self.process_telemetry
            .lock()
            .map(|telemetry_map| {
                telemetry_map
                    .iter()
                    .filter(|(_, telemetry)| {
                        matches!(
                            telemetry.status,
                            HMRProcessStatus::Started | HMRProcessStatus::InProgress
                        )
                    })
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_error_history(&self, limit: Option<usize>) -> Vec<HMRErrorDetails> {
        self.error_history
            .lock()
            .map(|history| {
                let mut errors = history.clone();
                errors.reverse(); // Most recent first

                if let Some(limit) = limit {
                    errors.truncate(limit);
                }

                errors
            })
            .unwrap_or_default()
    }
}

impl DefaultHMRObservabilitySystem {
    fn calculate_cluster_health(&self, status: &ClusterUpdateStatus) -> f64 {
        if status.node_statuses.is_empty() {
            return 0.0;
        }

        let healthy_nodes = status
            .node_statuses
            .values()
            .filter(|node_status| {
                matches!(
                    node_status.status,
                    NodeUpdateState::Completed
                        | NodeUpdateState::Waiting
                        | NodeUpdateState::Preparing
                )
            })
            .count();

        let total_nodes = status.node_statuses.len();

        // Base health on node status
        let node_health = healthy_nodes as f64 / total_nodes as f64;

        // Adjust for consensus status
        let consensus_factor = if status.consensus_status.consensus_reached {
            1.0
        } else {
            0.8
        };

        // Adjust for overall status
        let status_factor = match status.overall_status {
            ClusterUpdateOverallStatus::Completed => 1.0,
            ClusterUpdateOverallStatus::Executing | ClusterUpdateOverallStatus::Validating => 0.9,
            ClusterUpdateOverallStatus::Planning | ClusterUpdateOverallStatus::Coordinating => 0.8,
            ClusterUpdateOverallStatus::Failed | ClusterUpdateOverallStatus::RollingBack => 0.3,
            ClusterUpdateOverallStatus::RolledBack => 0.5,
            ClusterUpdateOverallStatus::Completing => 0.95,
        };

        (node_health * consensus_factor * status_factor).clamp(0.0, 1.0)
    }
}
