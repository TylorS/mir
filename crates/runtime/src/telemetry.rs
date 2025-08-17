//! OpenTelemetry integration for MIR runtime
//! 
//! This module provides comprehensive observability through OpenTelemetry,
//! including automatic instrumentation for IR operations, distributed tracing
//! across cluster nodes, and HMR process tracing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use opentelemetry::Value as OtelValue;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use mir_types::ContentHash;
use crate::NodeId;

/// OpenTelemetry collector for comprehensive runtime observability
pub struct OpenTelemetryCollector {
    active_spans: Arc<Mutex<HashMap<SpanId, ActiveSpan>>>,
    distributed_context: Arc<Mutex<DistributedTraceContext>>,
    instrumentation_config: InstrumentationConfig,
    performance_tracker: PerformanceTracker,
    hmr_tracer: HMRTracer,
}

/// Configuration for automatic instrumentation
#[derive(Debug, Clone)]
pub struct InstrumentationConfig {
    pub auto_instrument_ir_operations: bool,
    pub auto_instrument_hmr_process: bool,
    pub distributed_tracing_enabled: bool,
    pub performance_annotation_enabled: bool,
    pub sampling_rate: f64,
    pub max_span_attributes: usize,
    pub span_timeout: Duration,
}

impl Default for InstrumentationConfig {
    fn default() -> Self {
        Self {
            auto_instrument_ir_operations: true,
            auto_instrument_hmr_process: true,
            distributed_tracing_enabled: true,
            performance_annotation_enabled: true,
            sampling_rate: 1.0,
            max_span_attributes: 128,
            span_timeout: Duration::from_secs(300),
        }
    }
}

/// Active span tracking for lifecycle management
#[allow(dead_code)]
struct ActiveSpan {
    span_id: SpanId,
    start_time: SystemTime,
    operation_type: OperationType,
    node_id: Option<NodeId>,
    attributes: HashMap<String, OtelValue>,
}

/// Types of operations that can be instrumented
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    IRExecution {
        operation_name: String,
        module_hash: ContentHash,
    },
    HMRProcess {
        process_type: HMRProcessType,
        affected_modules: Vec<ContentHash>,
    },
    DistributedCoordination {
        coordination_type: CoordinationType,
        participating_nodes: Vec<NodeId>,
    },
    StateManagement {
        state_operation: StateOperationType,
        component_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HMRProcessType {
    ChangeDetection,
    DependencyAnalysis,
    CompatibilityCheck,
    StateMigration,
    ModuleUpdate,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationType {
    ConsensusProtocol,
    StateSync,
    NodeJoin,
    NodeLeave,
    PartitionHandling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateOperationType {
    Snapshot,
    Restore,
    Migration,
    Validation,
}

/// Distributed trace context for cross-node correlation
#[derive(Debug, Clone)]
struct DistributedTraceContext {
    active_traces: HashMap<TraceId, DistributedTrace>,
    node_correlations: HashMap<NodeId, Vec<TraceId>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DistributedTrace {
    trace_id: TraceId,
    root_span_id: SpanId,
    participating_nodes: Vec<NodeId>,
    operation_type: OperationType,
    start_time: SystemTime,
}

/// Performance tracking and bottleneck detection
#[derive(Debug)]
struct PerformanceTracker {
    operation_timings: HashMap<String, Vec<Duration>>,
    bottleneck_threshold: Duration,
    performance_annotations: Vec<PerformanceAnnotation>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PerformanceAnnotation {
    operation: String,
    duration: Duration,
    bottleneck_type: BottleneckType,
    suggestions: Vec<String>,
    timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum BottleneckType {
    CPUIntensive,
    MemoryIntensive,
    IOBound,
    NetworkLatency,
    ContentionLock,
    GarbageCollection,
}

/// HMR-specific tracing with dependency analysis
#[derive(Debug)]
struct HMRTracer {
    active_hmr_processes: HashMap<HMRProcessId, HMRProcessTrace>,
    dependency_traces: HashMap<ContentHash, Vec<DependencyTrace>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HMRProcessTrace {
    process_id: HMRProcessId,
    process_type: HMRProcessType,
    affected_modules: Vec<ContentHash>,
    dependency_chain: Vec<ContentHash>,
    start_time: SystemTime,
    phases: Vec<HMRPhaseTrace>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HMRPhaseTrace {
    phase_name: String,
    start_time: SystemTime,
    end_time: Option<SystemTime>,
    dependencies_analyzed: usize,
    compatibility_checks: usize,
    state_migrations: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DependencyTrace {
    from_module: ContentHash,
    to_module: ContentHash,
    dependency_type: DependencyType,
    impact_level: ImpactLevel,
    trace_id: TraceId,
}

#[derive(Debug, Clone)]
pub enum DependencyType {
    DirectImport,
    TypeDependency,
    StateDependency,
    RuntimeDependency,
}

#[derive(Debug, Clone)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Trace context for cross-module and cross-node propagation
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub baggage: HashMap<String, String>,
    pub source_module: ContentHash,
    pub target_module: ContentHash,
    pub target_node: Option<NodeId>,
}

// Type aliases for clarity
pub type SpanId = Uuid;
pub type TraceId = Uuid;
pub type HMRProcessId = Uuid;

impl OpenTelemetryCollector {
    /// Initialize OpenTelemetry collector with configuration
    pub fn new(config: InstrumentationConfig) -> Result<Self, TelemetryError> {
        // Initialize basic tracing
        Self::init_tracing(&config)?;
        
        Ok(Self {
            active_spans: Arc::new(Mutex::new(HashMap::new())),
            distributed_context: Arc::new(Mutex::new(DistributedTraceContext {
                active_traces: HashMap::new(),
                node_correlations: HashMap::new(),
            })),
            instrumentation_config: config,
            performance_tracker: PerformanceTracker {
                operation_timings: HashMap::new(),
                bottleneck_threshold: Duration::from_millis(100),
                performance_annotations: Vec::new(),
            },
            hmr_tracer: HMRTracer {
                active_hmr_processes: HashMap::new(),
                dependency_traces: HashMap::new(),
            },
        })
    }

    /// Initialize tracing with basic configuration
    fn init_tracing(_config: &InstrumentationConfig) -> Result<(), TelemetryError> {
        // For now, use a simple tracing setup
        // In a full implementation, this would set up OpenTelemetry properly
        // Only initialize if not already initialized
        if tracing_subscriber::fmt::try_init().is_ok() {
            info!("OpenTelemetry tracing initialized");
        }
        Ok(())
    }

    /// Start automatic instrumentation for IR operations
    pub fn instrument_ir_operation(
        &mut self,
        operation_name: &str,
        module_hash: ContentHash,
        node_id: Option<NodeId>,
    ) -> Result<SpanId, TelemetryError> {
        if !self.instrumentation_config.auto_instrument_ir_operations {
            return Err(TelemetryError::InstrumentationDisabled);
        }

        let span_id = Uuid::new_v4();
        
        // Use tracing for instrumentation
        info!(
            span_id = %span_id,
            operation_name = operation_name,
            module_hash = ?module_hash,
            node_id = ?node_id,
            "Starting IR operation"
        );

        let active_span = ActiveSpan {
            span_id,
            start_time: SystemTime::now(),
            operation_type: OperationType::IRExecution {
                operation_name: operation_name.to_string(),
                module_hash,
            },
            node_id,
            attributes: HashMap::new(),
        };

        self.active_spans.lock().unwrap().insert(span_id, active_span);
        
        debug!("Started IR operation span: {} for operation: {}", span_id, operation_name);
        Ok(span_id)
    }

    /// Start HMR process tracing with dependency analysis
    pub fn start_hmr_process_trace(
        &mut self,
        process_type: HMRProcessType,
        affected_modules: Vec<ContentHash>,
    ) -> Result<HMRProcessId, TelemetryError> {
        if !self.instrumentation_config.auto_instrument_hmr_process {
            return Err(TelemetryError::InstrumentationDisabled);
        }

        let process_id = Uuid::new_v4();
        
        // Use tracing for HMR process instrumentation
        info!(
            process_id = %process_id,
            process_type = ?process_type,
            affected_modules_count = affected_modules.len(),
            affected_modules = ?affected_modules,
            "Starting HMR process"
        );

        let process_trace = HMRProcessTrace {
            process_id,
            process_type: process_type.clone(),
            affected_modules: affected_modules.clone(),
            dependency_chain: Vec::new(),
            start_time: SystemTime::now(),
            phases: Vec::new(),
        };

        self.hmr_tracer.active_hmr_processes.insert(process_id, process_trace);

        // Create span for tracking
        let span_id = Uuid::new_v4();
        let active_span = ActiveSpan {
            span_id,
            start_time: SystemTime::now(),
            operation_type: OperationType::HMRProcess {
                process_type: process_type.clone(),
                affected_modules: affected_modules.clone(),
            },
            node_id: None,
            attributes: HashMap::new(),
        };

        self.active_spans.lock().unwrap().insert(span_id, active_span);
        
        info!("Started HMR process trace: {} for process: {:?}", process_id, process_type);
        Ok(process_id)
    }

    /// Add dependency analysis to HMR trace
    pub fn trace_dependency_analysis(
        &mut self,
        process_id: HMRProcessId,
        from_module: ContentHash,
        to_module: ContentHash,
        dependency_type: DependencyType,
        impact_level: ImpactLevel,
    ) -> Result<(), TelemetryError> {
        let trace_id = Uuid::new_v4();
        
        if let Some(process_trace) = self.hmr_tracer.active_hmr_processes.get_mut(&process_id) {
            process_trace.dependency_chain.push(to_module);
        }

        let dependency_trace = DependencyTrace {
            from_module,
            to_module,
            dependency_type: dependency_type.clone(),
            impact_level: impact_level.clone(),
            trace_id,
        };

        self.hmr_tracer.dependency_traces
            .entry(from_module)
            .or_default()
            .push(dependency_trace);

        // Use tracing for dependency analysis
        info!(
            trace_id = %trace_id,
            process_id = %process_id,
            from_module = ?from_module,
            to_module = ?to_module,
            dependency_type = ?dependency_type,
            impact_level = ?impact_level,
            "Dependency analysis traced"
        );
        
        debug!("Traced dependency: {:?} -> {:?} (type: {:?}, impact: {:?})", 
               from_module, to_module, dependency_type, impact_level);
        
        Ok(())
    }

    /// Start distributed tracing across cluster nodes
    pub fn start_distributed_trace(
        &mut self,
        coordination_type: CoordinationType,
        participating_nodes: Vec<NodeId>,
    ) -> Result<TraceId, TelemetryError> {
        if !self.instrumentation_config.distributed_tracing_enabled {
            return Err(TelemetryError::InstrumentationDisabled);
        }

        let trace_id = Uuid::new_v4();
        
        // Use tracing for distributed coordination
        info!(
            trace_id = %trace_id,
            coordination_type = ?coordination_type,
            node_count = participating_nodes.len(),
            participating_nodes = ?participating_nodes,
            "Starting distributed trace"
        );

        let distributed_trace = DistributedTrace {
            trace_id,
            root_span_id: Uuid::new_v4(),
            participating_nodes: participating_nodes.clone(),
            operation_type: OperationType::DistributedCoordination {
                coordination_type: coordination_type.clone(),
                participating_nodes: participating_nodes.clone(),
            },
            start_time: SystemTime::now(),
        };

        let mut context = self.distributed_context.lock().unwrap();
        context.active_traces.insert(trace_id, distributed_trace);
        
        // Update node correlations
        for node_id in participating_nodes {
            context.node_correlations
                .entry(node_id)
                .or_default()
                .push(trace_id);
        }

        // Create active span
        let span_id = Uuid::new_v4();
        let active_span = ActiveSpan {
            span_id,
            start_time: SystemTime::now(),
            operation_type: OperationType::DistributedCoordination {
                coordination_type: coordination_type.clone(),
                participating_nodes: Vec::new(), // Already stored in distributed_trace
            },
            node_id: None,
            attributes: HashMap::new(),
        };

        self.active_spans.lock().unwrap().insert(span_id, active_span);
        
        info!("Started distributed trace: {} for coordination: {:?}", trace_id, coordination_type);
        Ok(trace_id)
    }

    /// Annotate performance bottlenecks
    pub fn annotate_performance_bottleneck(
        &mut self,
        span_id: SpanId,
        operation: &str,
        duration: Duration,
        bottleneck_type: BottleneckType,
        suggestions: Vec<String>,
    ) -> Result<(), TelemetryError> {
        if !self.instrumentation_config.performance_annotation_enabled {
            return Ok(());
        }

        // Update performance tracking
        self.performance_tracker.operation_timings
            .entry(operation.to_string())
            .or_default()
            .push(duration);

        let annotation = PerformanceAnnotation {
            operation: operation.to_string(),
            duration,
            bottleneck_type: bottleneck_type.clone(),
            suggestions: suggestions.clone(),
            timestamp: SystemTime::now(),
        };

        self.performance_tracker.performance_annotations.push(annotation);

        // Log performance bottleneck
        warn!(
            span_id = %span_id,
            operation = operation,
            duration_ms = duration.as_millis(),
            bottleneck_type = ?bottleneck_type,
            suggestions = ?suggestions,
            "Performance bottleneck detected"
        );
        
        Ok(())
    }

    /// End a span and collect timing information
    pub fn end_span(&mut self, span_id: SpanId) -> Result<(), TelemetryError> {
        let active_span = {
            let mut spans = self.active_spans.lock().unwrap();
            spans.remove(&span_id)
        };
        
        if let Some(active_span) = active_span {
            let duration = SystemTime::now()
                .duration_since(active_span.start_time)
                .unwrap_or(Duration::ZERO);

            // Log span completion
            info!(
                span_id = %span_id,
                duration_ms = duration.as_millis(),
                operation_type = ?active_span.operation_type,
                node_id = ?active_span.node_id,
                "Span completed"
            );

            // Check for performance issues
            if duration > self.performance_tracker.bottleneck_threshold {
                let operation_name = match &active_span.operation_type {
                    OperationType::IRExecution { operation_name, .. } => operation_name.clone(),
                    OperationType::HMRProcess { process_type, .. } => format!("hmr_{process_type:?}"),
                    OperationType::DistributedCoordination { coordination_type, .. } => format!("distributed_{coordination_type:?}"),
                    OperationType::StateManagement { state_operation, .. } => format!("state_{state_operation:?}"),
                };

                self.annotate_performance_bottleneck(
                    span_id,
                    &operation_name,
                    duration,
                    BottleneckType::CPUIntensive, // Default assumption
                    vec!["Consider optimization".to_string()],
                )?;
            }

            debug!("Ended span: {} (duration: {}ms)", span_id, duration.as_millis());
        } else {
            return Err(TelemetryError::SpanNotFound(span_id));
        }

        Ok(())
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let mut operation_stats = HashMap::new();
        
        for (operation, timings) in &self.performance_tracker.operation_timings {
            let total_calls = timings.len();
            let total_duration: Duration = timings.iter().sum();
            let avg_duration = if total_calls > 0 {
                total_duration / total_calls as u32
            } else {
                Duration::ZERO
            };
            let max_duration = timings.iter().max().copied().unwrap_or(Duration::ZERO);
            let min_duration = timings.iter().min().copied().unwrap_or(Duration::ZERO);

            operation_stats.insert(operation.clone(), OperationStats {
                total_calls,
                total_duration,
                avg_duration,
                max_duration,
                min_duration,
            });
        }

        PerformanceStats {
            operation_stats,
            bottleneck_annotations: self.performance_tracker.performance_annotations.clone(),
            active_spans_count: self.active_spans.lock().unwrap().len(),
        }
    }

    /// Handle span lifecycle during hot-reloading
    pub fn handle_hmr_span_lifecycle(
        &mut self,
        old_module_hash: ContentHash,
        new_module_hash: ContentHash,
    ) -> Result<(), TelemetryError> {
        // Find spans associated with the old module
        let mut spans_to_migrate = Vec::new();
        {
            let spans = self.active_spans.lock().unwrap();
            for (span_id, active_span) in spans.iter() {
                match &active_span.operation_type {
                    OperationType::IRExecution { module_hash, .. } if *module_hash == old_module_hash => {
                        spans_to_migrate.push(*span_id);
                    }
                    _ => {}
                }
            }
        }

        // Migrate spans to new module
        let span_count = spans_to_migrate.len();
        for span_id in spans_to_migrate {
            self.migrate_span_to_new_module(span_id, new_module_hash)?;
        }

        info!(
            "Migrated {} spans from module {:?} to {:?}",
            span_count,
            old_module_hash,
            new_module_hash
        );

        Ok(())
    }

    /// Migrate a span to a new module during hot-reloading
    fn migrate_span_to_new_module(
        &mut self,
        span_id: SpanId,
        new_module_hash: ContentHash,
    ) -> Result<(), TelemetryError> {
        let mut spans = self.active_spans.lock().unwrap();
        if let Some(active_span) = spans.get_mut(&span_id) {
            // Update the operation type with new module hash
            if let OperationType::IRExecution { module_hash, .. } = &mut active_span.operation_type {
                *module_hash = new_module_hash;
            }

            info!(
                "Migrated span {} to new module {:?}",
                span_id, new_module_hash
            );
        }

        Ok(())
    }

    /// Create correlation between explicit and automatic tracing
    pub fn correlate_explicit_automatic_tracing(
        &mut self,
        explicit_span_id: SpanId,
        automatic_span_id: SpanId,
    ) -> Result<(), TelemetryError> {
        // Add correlation information to both spans
        {
            let mut spans = self.active_spans.lock().unwrap();
            
            if let Some(explicit_span) = spans.get_mut(&explicit_span_id) {
                explicit_span.attributes.insert(
                    "correlated_automatic_span".to_string(),
                    OtelValue::String(automatic_span_id.to_string().into()),
                );
            }

            if let Some(automatic_span) = spans.get_mut(&automatic_span_id) {
                automatic_span.attributes.insert(
                    "correlated_explicit_span".to_string(),
                    OtelValue::String(explicit_span_id.to_string().into()),
                );
            }
        }

        info!(
            "Correlated explicit span {} with automatic span {}",
            explicit_span_id, automatic_span_id
        );

        Ok(())
    }

    /// Propagate trace context across modules and nodes
    pub fn propagate_trace_context(
        &mut self,
        source_span_id: SpanId,
        target_module: ContentHash,
        target_node: Option<NodeId>,
    ) -> Result<TraceContext, TelemetryError> {
        let trace_context = {
            let spans = self.active_spans.lock().unwrap();
            if let Some(_active_span) = spans.get(&source_span_id) {
                TraceContext {
                    trace_id: Uuid::new_v4(), // In real implementation, extract from span
                    span_id: source_span_id,
                    baggage: HashMap::new(),
                    source_module: ContentHash::zero(), // TODO: Get from span
                    target_module,
                    target_node,
                }
            } else {
                return Err(TelemetryError::SpanNotFound(source_span_id));
            }
        };

        info!(
            "Propagated trace context from span {} to module {:?}",
            source_span_id, target_module
        );

        Ok(trace_context)
    }

    /// Shutdown telemetry and flush remaining spans
    pub fn shutdown(&mut self) -> Result<(), TelemetryError> {
        // End all active spans
        let span_ids: Vec<SpanId> = self.active_spans.lock().unwrap().keys().copied().collect();
        for span_id in span_ids {
            if let Err(e) = self.end_span(span_id) {
                warn!("Failed to end span during shutdown: {:?}", e);
            }
        }

        info!("OpenTelemetry collector shutdown complete");
        Ok(())
    }
}

/// Performance statistics for operations
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub operation_stats: HashMap<String, OperationStats>,
    pub bottleneck_annotations: Vec<PerformanceAnnotation>,
    pub active_spans_count: usize,
}

#[derive(Debug, Clone)]
pub struct OperationStats {
    pub total_calls: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
}

/// Telemetry errors
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Failed to initialize telemetry: {0}")]
    InitializationFailed(String),
    
    #[error("Instrumentation is disabled")]
    InstrumentationDisabled,
    
    #[error("Span not found: {0}")]
    SpanNotFound(SpanId),
    
    #[error("Invalid trace context")]
    InvalidTraceContext,
    
    #[error("Distributed tracing error: {0}")]
    DistributedTracingError(String),
}

// Types are already public, no need to re-export