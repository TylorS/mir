//! MIR Runtime - Core runtime services and coordination
//! 
//! This crate provides the runtime services for MIR, including:
//! - Content-addressable storage
//! - State management
//! - Distributed coordination primitives
//! - OpenTelemetry integration

pub mod storage;
pub mod schema_storage;
pub mod module_store;
pub mod state;
pub mod distributed;
pub mod crdt;
pub mod telemetry;
pub mod config;
pub mod change_analysis;
pub mod hmr_coordinator;
pub mod state_migration;
pub mod migration_functions;
pub mod consensus;
pub mod distributed_hmr;
pub mod ffi;
pub mod logging;
pub mod hmr_observability;
pub mod source_mapping;
pub mod debug_support;
pub mod profiling;
pub mod error_reporting;
pub mod dev_tools;

pub use storage::{ContentAddressableStore, InMemoryStore, RefCountingStore, StorageStats};
pub use schema_storage::{
    SchemaAwareStore, VersionedValue, ValueMetadata, StorageResult, QueryParams,
    MigrationPath, MigrationStep, MigrationType, SchemaStats, GCResult, StorageError
};
pub use module_store::{
    ModuleStore, ModuleMetadata, ModuleVersion, ExportInfo, ImportInfo,
    ModuleStoreResult, ModuleQueryParams, CompatibilityReport, ModuleStoreError
};
pub use state::{
    StateManager, StateSnapshot, StateExtractionResult, StateRestorationResult,
    ComponentState, ComponentType, StateExtractionConfig, StateRestorationConfig,
    AtomicStateUpdate, StateChange, StateChangeType, StateError, StateExtractor,
    StateRestorer, SnapshotMetadata, StateDependency, DependencyType as StateDependencyType,
    MissingComponentStrategy
};
pub use distributed::{
    DistributedRuntime, NodeInfo, NodeStatus, TaskId, FunctionId, ScheduleId, MessageId, SubscriptionId,
    DistributedResult, DistributedError, TaskInfo, TaskStatus, EventLoop, BasicEventLoop,
    DistributedTask, TaskPriority, RetryPolicy, ScheduleStatus, Scheduler, BasicScheduler,
    ScheduleInfo, OrderingGuarantee, QueueItem, Queue, BasicQueue, TopicConfig, RetentionPolicy,
    PartitioningStrategy, Message, Subscription, PubSubChannel, BasicPubSubChannel
};
pub use crdt::{
    NodeId, LogicalTimestamp, VersionVector, PartialOrdering, UniqueTag, ElementId, OperationId,
    CRDTResult, CRDTError, GCounter, PNCounter, GSet, TwoPhaseSet, ORSet, LWWRegister, MVRegister
};
pub use telemetry::{
    OpenTelemetryCollector, InstrumentationConfig, OperationType, HMRProcessType,
    CoordinationType, StateOperationType, PerformanceStats, OperationStats, 
    TelemetryError, SpanId, TraceId, HMRProcessId, BottleneckType, ImpactLevel,
    DependencyType as TelemetryDependencyType, TraceContext
};
pub use config::{
    RuntimeConfig, HMRConfig, Environment, UpdatePolicy, Schedule, ValidationLevel, 
    RollbackStrategy, RollbackFailureStrategy, HealthCheck, StatePreservationConfig,
    EncryptionConfig, BackupStrategy, EnvironmentAdapter, BasicEnvironmentAdapter,
    SafetyAssessment, RiskLevel, ValidationCheck, DeploymentStrategy, EnvironmentConstraint,
    ConstraintType, RollbackFeasibility, ConfigError
};
pub use change_analysis::{
    ChangeAnalysisSystem, ChangeAnalysis, AffectedModule, ImpactType, AffectedItem,
    ItemType, ChangeType, SchemaChange, SchemaChangeType, CompatibilityLevel,
    ChangeOptimization, OptimizationType, PerformanceImpact, FunctionPurityAnalyzer,
    PurityAnalysis, SideEffect, ExternalDependency, DependencyType as AnalysisDependencyType,
    SchemaCompatibilityChecker, DependencyGraph, DependencyNode, DependencyEdge,
    DependencyEdgeType, ChangeAnalysisError
};
pub use hmr_coordinator::{
    HMRCoordinator, SafetyLevel, UpdatePlan, UpdateStep, UpdateStepType,
    StepStatus, UpdatePriority, UpdateStatus, RollbackPlan, RollbackStep, RollbackStepType,
    UpdateResult, StateChangeRecord, UpdateRecord, ValidationResult, ValidationSeverity,
    UpdateValidator, ValidationRule, RollbackManager, RollbackConfig, RollbackExecution,
    RollbackStatus, HMRError, ValidationError
};
pub use state_migration::{
    StateMigrationSystem, MigrationRegistry, SchemaTransition, MigrationConfig,
    MigrationFunction, AutoMigrationGenerator, MigrationContext, MigrationChain,
    MigrationStep as StateMigrationStep, MigrationStepStatus, MigrationResult, MigrationRecord,
    MigrationExecutionEngine, MigrationExecution, MigrationExecutionStatus,
    MigrationError
};
pub use migration_functions::{
    AddFieldMigration, RemoveFieldMigration, RenameFieldMigration, TransformFieldMigration,
    IdentityMigration, BackwardCompatibleGenerator, AddFieldGenerator, UserDefinedMigration,
    BuiltinMigrations
};
pub use consensus::{
    ConsensusProtocol, BasicConsensusProtocol, ConsensusNode, ConsensusConfig, ConsensusError,
    ConsensusResult, ProposalId, RoundId, NodeStatus as ConsensusNodeStatus, ProposalType, Proposal, Vote, VoteRecord,
    ProposalStatus, ProposalInfo
};
pub use distributed_hmr::{
    DistributedHMRCoordinator, BasicDistributedHMRCoordinator, DistributedHMRError, DistributedHMRResult,
    UpdateId, DistributedUpdateStatus, DistributedUpdateResult, DistributedUpdate,
    RollingUpdateStrategy, NodeUpdateStatus, NodeUpdateState, ClusterHealthStatus, OverallHealth
};
pub use ffi::{
    FFIBridge, BasicFFIBridge, FFIBinding, FFISignature, FFIParameter, CallingConvention,
    FFIImplementation, NativeFunction, ExternalFunction, HostCallback, FFIFunctionMetadata,
    SideEffect as FFISideEffect, FFIFunctionId, HostEnvironmentId, HostEnvironment, HostCapability, HostState,
    ConnectionState, ResourceState, FFIStateSnapshot, FFIFunctionState, FFIStateCoordinator,
    CompatibilityResult, FFIError, FFIErrorHandler, BasicFFIErrorHandler, MultiHostCompatibilityManager,
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState, FallbackStrategy, CustomFallbackHandler,
    DegradedBehavior, FFIRecoveryResult, UpdateRecoveryResult, UpdateContext, MigrationStep as FFIMigrationStep,
    MigrationStepType, RollbackStrategy as FFIRollbackStrategy, RecoveryAction, HostRecoveryOption, CompatibilityIssue,
    CompatibilityIssueType, CompatibilitySeverity, CompatibilityResolution, ResolutionStep,
    ResolutionStepType, CompatibilityResolutionStrategy, RecoveryRecord, RecoveryStatistics
};
pub use logging::{
    Logger, StructuredLogger, LogContext, LoggingError, LoggingResult, FunctionSpanConfig,
    SpanAnnotation, SpanEvent, FunctionSpanner, CounterId, GaugeId, HistogramId, Counter,
    Gauge, Histogram, MetricsRegistry, DefaultMetricsRegistry, LogFormat, MetricsFormat,
    TraceFormat, ExportDestination, NetworkEndpoint, CustomExporter, LogEntry, MetricsSnapshot,
    CounterSnapshot, GaugeSnapshot, HistogramSnapshot, TraceEntry, SpanLog, ObservabilityExporter,
    DefaultObservabilityExporter, LoggingSystem
};
pub use hmr_observability::{
    HMRObservabilityError, HMRObservabilityResult, UpdateProcessId, HMRProcessType as ObservabilityHMRProcessType, HMRProcessStatus,
    HMRProcessTelemetry, HMRPerformanceMetrics, StateMigrationTracking, MigrationType as ObservabilityMigrationType,
    TransformationStep, TransformationStepType, MigrationErrorDetails, HMRErrorDetails,
    HMRErrorType, StackFrame, RecoveryAction as ObservabilityRecoveryAction, RecoveryActionType, ClusterUpdateStatus,
    ClusterUpdateOverallStatus, NodeUpdateStatus as ObservabilityNodeUpdateStatus, NodeUpdateState as ObservabilityNodeUpdateState, ConsensusStatus, ConsensusVote,
    HealthCheckResult, HealthCheckStatus, HMRObservabilitySystem, DefaultHMRObservabilitySystem
};
pub use source_mapping::{
    SourceMap, SourceMapping, SourcePosition, GeneratedPosition, SourceMapRegistry
};
pub use debug_support::{
    DebugInfo, StackFrame as DebugStackFrame, VariableInfo, Breakpoint, DebugSession,
    DebugEvent, DebugEventHandler
};
pub use profiling::{
    ProfileData, SourceProfileEntry, SourceProfiler, ProfilingSettings, ProfileExportFormat
};
pub use error_reporting::{
    SourceMappedError, ErrorKind, ErrorContext, SourceMappedErrorReporter,
    ErrorReportingSettings, ErrorHandler, ConsoleErrorHandler, FileErrorHandler
};
pub use dev_tools::{
    DevToolsIntegration, DevToolsConfig, FileChangeEvent, FileChangeType, FileChangeMetadata,
    FileSystemWatcher, BuildToolIntegration, BuildConfig, BuildResult, BuildArtifact, ArtifactType,
    BuildStatus, VersionControlIntegration, CommitInfo, FileChange, VCChangeType, BranchInfo,
    CICDIntegration, DeploymentConfig, DeploymentStrategy as DevToolsDeploymentStrategy, DeploymentResult, DeploymentStatus,
    DeploymentInfo, PipelineConfig, PipelineTrigger, PipelineStage, HealthCheckConfig,
    FileWatchError, BuildError, VCError, CICDError, WebpackIntegration, ViteIntegration, GitIntegration
};

// Re-export ContentHash from types crate for convenience
pub use mir_types::ContentHash;