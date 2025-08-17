//! Comprehensive logging system for MIR runtime
//! 
//! This module provides structured logging with automatic function spanning,
//! user-defined metrics, and exportable telemetry data.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{span, Level, Span};
use uuid::Uuid;

use crate::distributed::FunctionId;

/// Errors that can occur in the logging system
#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Failed to initialize logger: {0}")]
    InitializationError(String),
    #[error("Failed to export logs: {0}")]
    ExportError(String),
    #[error("Invalid log format: {0}")]
    InvalidFormat(String),
    #[error("Metric not found: {0}")]
    MetricNotFound(String),
    #[error("Invalid metric operation: {0}")]
    InvalidMetricOperation(String),
}

pub type LoggingResult<T> = Result<T, LoggingError>;

/// Log context for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub correlation_id: String,
    pub node_id: Option<String>,
    pub module_hash: Option<String>,
    pub function_id: Option<FunctionId>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl LogContext {
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            node_id: None,
            module_hash: None,
            function_id: None,
            custom_fields: HashMap::new(),
        }
    }

    pub fn with_node_id(mut self, node_id: String) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn with_module_hash(mut self, module_hash: String) -> Self {
        self.module_hash = Some(module_hash);
        self
    }

    pub fn with_function_id(mut self, function_id: FunctionId) -> Self {
        self.function_id = Some(function_id);
        self
    }

    pub fn with_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_fields.insert(key, value);
        self
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured logging trait
pub trait Logger: Send + Sync {
    fn trace(&self, message: &str, fields: &HashMap<String, serde_json::Value>);
    fn debug(&self, message: &str, fields: &HashMap<String, serde_json::Value>);
    fn info(&self, message: &str, fields: &HashMap<String, serde_json::Value>);
    fn warn(&self, message: &str, fields: &HashMap<String, serde_json::Value>);
    fn error(&self, message: &str, fields: &HashMap<String, serde_json::Value>);
    fn with_context(&self, context: LogContext) -> Box<dyn Logger>;
}

/// Default logger implementation using tracing
pub struct StructuredLogger {
    context: LogContext,
}

impl StructuredLogger {
    pub fn new() -> Self {
        Self {
            context: LogContext::new(),
        }
    }

    pub fn with_context(context: LogContext) -> Self {
        Self { context }
    }

    fn create_span(&self, level: Level, message: &str) -> Span {
        let span = match level {
            Level::TRACE => span!(Level::TRACE, "mir_log", message = message),
            Level::DEBUG => span!(Level::DEBUG, "mir_log", message = message),
            Level::INFO => span!(Level::INFO, "mir_log", message = message),
            Level::WARN => span!(Level::WARN, "mir_log", message = message),
            Level::ERROR => span!(Level::ERROR, "mir_log", message = message),
        };
        
        span.record("correlation_id", self.context.correlation_id.as_str());
        span.record("node_id", self.context.node_id.as_deref().unwrap_or("unknown"));
        span.record("module_hash", self.context.module_hash.as_deref().unwrap_or("unknown"));
        if let Some(function_id) = &self.context.function_id {
            span.record("function_id", format!("{function_id:?}").as_str());
        }

        // Add custom fields to span
        for (key, value) in &self.context.custom_fields {
            span.record(key.as_str(), tracing::field::display(value));
        }

        span
    }

    fn log_with_fields(&self, level: Level, message: &str, fields: &HashMap<String, serde_json::Value>) {
        let span = self.create_span(level, message);
        let _enter = span.enter();

        // Record additional fields
        for (key, value) in fields {
            span.record(key.as_str(), tracing::field::display(value));
        }

        match level {
            Level::TRACE => tracing::trace!("{}", message),
            Level::DEBUG => tracing::debug!("{}", message),
            Level::INFO => tracing::info!("{}", message),
            Level::WARN => tracing::warn!("{}", message),
            Level::ERROR => tracing::error!("{}", message),
        }
    }
}

impl Logger for StructuredLogger {
    fn trace(&self, message: &str, fields: &HashMap<String, serde_json::Value>) {
        self.log_with_fields(Level::TRACE, message, fields);
    }

    fn debug(&self, message: &str, fields: &HashMap<String, serde_json::Value>) {
        self.log_with_fields(Level::DEBUG, message, fields);
    }

    fn info(&self, message: &str, fields: &HashMap<String, serde_json::Value>) {
        self.log_with_fields(Level::INFO, message, fields);
    }

    fn warn(&self, message: &str, fields: &HashMap<String, serde_json::Value>) {
        self.log_with_fields(Level::WARN, message, fields);
    }

    fn error(&self, message: &str, fields: &HashMap<String, serde_json::Value>) {
        self.log_with_fields(Level::ERROR, message, fields);
    }

    fn with_context(&self, context: LogContext) -> Box<dyn Logger> {
        Box::new(StructuredLogger::with_context(context))
    }
}

impl Default for StructuredLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for automatic function spanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpanConfig {
    pub auto_span_all: bool,
    pub span_annotations: HashMap<FunctionId, SpanAnnotation>,
    pub sampling_rate: f64,
    pub include_arguments: bool,
    pub include_return_values: bool,
}

impl Default for FunctionSpanConfig {
    fn default() -> Self {
        Self {
            auto_span_all: false,
            span_annotations: HashMap::new(),
            sampling_rate: 1.0,
            include_arguments: false,
            include_return_values: false,
        }
    }
}

/// Annotation for function spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAnnotation {
    pub span_name: Option<String>,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

/// Events within spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub timestamp: Option<std::time::SystemTime>,
}

/// Automatic function spanning system
pub struct FunctionSpanner {
    config: FunctionSpanConfig,
    active_spans: Arc<Mutex<HashMap<FunctionId, Span>>>,
}

impl FunctionSpanner {
    pub fn new(config: FunctionSpanConfig) -> Self {
        Self {
            config,
            active_spans: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn should_span_function(&self, function_id: FunctionId) -> bool {
        if self.config.auto_span_all {
            return rand::random::<f64>() < self.config.sampling_rate;
        }
        
        self.config.span_annotations.contains_key(&function_id)
    }

    pub fn start_function_span(&self, function_id: FunctionId, args: Option<&[serde_json::Value]>) -> Option<Span> {
        if !self.should_span_function(function_id) {
            return None;
        }

        let annotation = self.config.span_annotations.get(&function_id);
        let span_name = annotation
            .and_then(|a| a.span_name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("function_call");

        let span = span!(
            Level::INFO,
            "function_span",
            function_id = ?function_id,
            span_name = span_name,
        );

        // Add custom attributes
        if let Some(annotation) = annotation {
            for (key, value) in &annotation.attributes {
                span.record(key.as_str(), value.as_str());
            }
        }

        // Include arguments if configured
        if self.config.include_arguments {
            if let Some(args) = args {
                span.record("arguments", tracing::field::display(serde_json::to_string(args).unwrap_or_default()));
            }
        }

        // Store active span
        if let Ok(mut active_spans) = self.active_spans.lock() {
            active_spans.insert(function_id, span.clone());
        }

        Some(span)
    }

    pub fn end_function_span(&self, function_id: FunctionId, return_value: Option<&serde_json::Value>) {
        if let Ok(mut active_spans) = self.active_spans.lock() {
            if let Some(span) = active_spans.remove(&function_id) {
                let _enter = span.enter();
                
                // Include return value if configured
                if self.config.include_return_values {
                    if let Some(return_value) = return_value {
                        span.record("return_value", tracing::field::display(serde_json::to_string(return_value).unwrap_or_default()));
                    }
                }

                // Add span events
                if let Some(annotation) = self.config.span_annotations.get(&function_id) {
                    for event in &annotation.events {
                        tracing::event!(Level::INFO, name = event.name, ?event.attributes);
                    }
                }
            }
        }
    }

    pub fn add_span_event(&self, function_id: FunctionId, event: SpanEvent) {
        if let Ok(active_spans) = self.active_spans.lock() {
            if let Some(span) = active_spans.get(&function_id) {
                let _enter = span.enter();
                tracing::event!(Level::INFO, name = event.name, ?event.attributes);
            }
        }
    }
}

/// Unique identifiers for metrics
pub type CounterId = Uuid;
pub type GaugeId = Uuid;
pub type HistogramId = Uuid;

/// Counter metric
#[derive(Debug, Clone)]
pub struct Counter {
    pub id: CounterId,
    pub name: String,
    pub description: String,
    pub value: Arc<Mutex<f64>>,
}

impl Counter {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            value: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn increment(&self, amount: f64) -> LoggingResult<()> {
        if let Ok(mut value) = self.value.lock() {
            *value += amount;
            Ok(())
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire counter lock".to_string()))
        }
    }

    pub fn get_value(&self) -> f64 {
        self.value.lock().map(|v| *v).unwrap_or(0.0)
    }
}

/// Gauge metric
#[derive(Debug, Clone)]
pub struct Gauge {
    pub id: GaugeId,
    pub name: String,
    pub description: String,
    pub value: Arc<Mutex<f64>>,
}

impl Gauge {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            value: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn set(&self, value: f64) -> LoggingResult<()> {
        if let Ok(mut current_value) = self.value.lock() {
            *current_value = value;
            Ok(())
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire gauge lock".to_string()))
        }
    }

    pub fn get_value(&self) -> f64 {
        self.value.lock().map(|v| *v).unwrap_or(0.0)
    }
}

/// Histogram metric
#[derive(Debug, Clone)]
pub struct Histogram {
    pub id: HistogramId,
    pub name: String,
    pub description: String,
    pub buckets: Vec<f64>,
    pub bucket_counts: Arc<Mutex<Vec<u64>>>,
    pub sum: Arc<Mutex<f64>>,
    pub count: Arc<Mutex<u64>>,
}

impl Histogram {
    pub fn new(name: String, description: String, buckets: Vec<f64>) -> Self {
        let bucket_count = buckets.len() + 1; // +1 for +Inf bucket
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            buckets,
            bucket_counts: Arc::new(Mutex::new(vec![0; bucket_count])),
            sum: Arc::new(Mutex::new(0.0)),
            count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn record(&self, value: f64) -> LoggingResult<()> {
        // Find appropriate bucket
        let bucket_index = self.buckets.iter()
            .position(|&bucket| value <= bucket)
            .unwrap_or(self.buckets.len());

        // Update bucket counts
        if let Ok(mut bucket_counts) = self.bucket_counts.lock() {
            if bucket_index < bucket_counts.len() {
                bucket_counts[bucket_index] += 1;
            }
        } else {
            return Err(LoggingError::InvalidMetricOperation("Failed to acquire bucket counts lock".to_string()));
        }

        // Update sum and count
        if let (Ok(mut sum), Ok(mut count)) = (self.sum.lock(), self.count.lock()) {
            *sum += value;
            *count += 1;
        } else {
            return Err(LoggingError::InvalidMetricOperation("Failed to acquire histogram locks".to_string()));
        }

        Ok(())
    }

    pub fn get_bucket_counts(&self) -> Vec<u64> {
        self.bucket_counts.lock().map(|v| v.clone()).unwrap_or_default()
    }

    pub fn get_sum(&self) -> f64 {
        self.sum.lock().map(|v| *v).unwrap_or(0.0)
    }

    pub fn get_count(&self) -> u64 {
        self.count.lock().map(|v| *v).unwrap_or(0)
    }
}

/// User-defined metrics registry
pub trait MetricsRegistry: Send + Sync {
    fn create_counter(&mut self, name: String, description: String) -> CounterId;
    fn create_gauge(&mut self, name: String, description: String) -> GaugeId;
    fn create_histogram(&mut self, name: String, description: String, buckets: Vec<f64>) -> HistogramId;
    fn increment_counter(&mut self, id: CounterId, value: f64, labels: &HashMap<String, String>) -> LoggingResult<()>;
    fn set_gauge(&mut self, id: GaugeId, value: f64, labels: &HashMap<String, String>) -> LoggingResult<()>;
    fn record_histogram(&mut self, id: HistogramId, value: f64, labels: &HashMap<String, String>) -> LoggingResult<()>;
    fn get_counter(&self, id: CounterId) -> Option<&Counter>;
    fn get_gauge(&self, id: GaugeId) -> Option<&Gauge>;
    fn get_histogram(&self, id: HistogramId) -> Option<&Histogram>;
    fn list_counters(&self) -> Vec<&Counter>;
    fn list_gauges(&self) -> Vec<&Gauge>;
    fn list_histograms(&self) -> Vec<&Histogram>;
}

/// Default metrics registry implementation
pub struct DefaultMetricsRegistry {
    counters: HashMap<CounterId, Counter>,
    gauges: HashMap<GaugeId, Gauge>,
    histograms: HashMap<HistogramId, Histogram>,
    labeled_metrics: HashMap<String, HashMap<String, serde_json::Value>>, // metric_id -> labels -> value
}

impl DefaultMetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            labeled_metrics: HashMap::new(),
        }
    }

    fn store_labels(&mut self, metric_id: &str, labels: &HashMap<String, String>) {
        if !labels.is_empty() {
            let labels_json: HashMap<String, serde_json::Value> = labels
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            self.labeled_metrics.insert(metric_id.to_string(), labels_json);
        }
    }
}

impl MetricsRegistry for DefaultMetricsRegistry {
    fn create_counter(&mut self, name: String, description: String) -> CounterId {
        let counter = Counter::new(name, description);
        let id = counter.id;
        self.counters.insert(id, counter);
        id
    }

    fn create_gauge(&mut self, name: String, description: String) -> GaugeId {
        let gauge = Gauge::new(name, description);
        let id = gauge.id;
        self.gauges.insert(id, gauge);
        id
    }

    fn create_histogram(&mut self, name: String, description: String, buckets: Vec<f64>) -> HistogramId {
        let histogram = Histogram::new(name, description, buckets);
        let id = histogram.id;
        self.histograms.insert(id, histogram);
        id
    }

    fn increment_counter(&mut self, id: CounterId, value: f64, labels: &HashMap<String, String>) -> LoggingResult<()> {
        self.store_labels(&id.to_string(), labels);
        if let Some(counter) = self.counters.get(&id) {
            counter.increment(value)
        } else {
            Err(LoggingError::MetricNotFound(format!("Counter with id {id} not found")))
        }
    }

    fn set_gauge(&mut self, id: GaugeId, value: f64, labels: &HashMap<String, String>) -> LoggingResult<()> {
        self.store_labels(&id.to_string(), labels);
        if let Some(gauge) = self.gauges.get(&id) {
            gauge.set(value)
        } else {
            Err(LoggingError::MetricNotFound(format!("Gauge with id {id} not found")))
        }
    }

    fn record_histogram(&mut self, id: HistogramId, value: f64, labels: &HashMap<String, String>) -> LoggingResult<()> {
        self.store_labels(&id.to_string(), labels);
        if let Some(histogram) = self.histograms.get(&id) {
            histogram.record(value)
        } else {
            Err(LoggingError::MetricNotFound(format!("Histogram with id {id} not found")))
        }
    }

    fn get_counter(&self, id: CounterId) -> Option<&Counter> {
        self.counters.get(&id)
    }

    fn get_gauge(&self, id: GaugeId) -> Option<&Gauge> {
        self.gauges.get(&id)
    }

    fn get_histogram(&self, id: HistogramId) -> Option<&Histogram> {
        self.histograms.get(&id)
    }

    fn list_counters(&self) -> Vec<&Counter> {
        self.counters.values().collect()
    }

    fn list_gauges(&self) -> Vec<&Gauge> {
        self.gauges.values().collect()
    }

    fn list_histograms(&self) -> Vec<&Histogram> {
        self.histograms.values().collect()
    }
}

impl Default for DefaultMetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Export formats for logs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
    Structured,
    OpenTelemetry,
}

/// Export formats for metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MetricsFormat {
    Prometheus,
    Json,
    OpenTelemetry,
}

/// Export formats for traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceFormat {
    Jaeger,
    Zipkin,
    OpenTelemetry,
}

/// Export destinations
#[derive(Debug, Clone)]
pub enum ExportDestination {
    File(std::path::PathBuf),
    Network(NetworkEndpoint),
    Console,
    Custom(String), // Custom exporter identifier
}

/// Network endpoint for exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub path: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Custom exporter trait
pub trait CustomExporter: Send + Sync {
    fn export_logs(&self, logs: &[LogEntry], format: LogFormat) -> LoggingResult<()>;
    fn export_metrics(&self, metrics: &MetricsSnapshot, format: MetricsFormat) -> LoggingResult<()>;
    fn export_traces(&self, traces: &[TraceEntry], format: TraceFormat) -> LoggingResult<()>;
}

/// Log entry for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: std::time::SystemTime,
    pub level: String,
    pub message: String,
    pub context: LogContext,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Metrics snapshot for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: std::time::SystemTime,
    pub counters: Vec<CounterSnapshot>,
    pub gauges: Vec<GaugeSnapshot>,
    pub histograms: Vec<HistogramSnapshot>,
}

/// Counter snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Gauge snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Histogram snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub buckets: Vec<f64>,
    pub bucket_counts: Vec<u64>,
    pub sum: f64,
    pub count: u64,
    pub labels: HashMap<String, String>,
}

/// Trace entry for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: std::time::SystemTime,
    pub end_time: Option<std::time::SystemTime>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

/// Span log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    pub timestamp: std::time::SystemTime,
    pub fields: HashMap<String, String>,
}

/// Exportable logging and metrics trait
pub trait ObservabilityExporter: Send + Sync {
    fn export_logs(&self, format: LogFormat, destination: ExportDestination) -> LoggingResult<()>;
    fn export_metrics(&self, format: MetricsFormat, destination: ExportDestination) -> LoggingResult<()>;
    fn export_traces(&self, format: TraceFormat, destination: ExportDestination) -> LoggingResult<()>;
    fn register_custom_exporter(&mut self, name: String, exporter: Box<dyn CustomExporter>) -> LoggingResult<()>;
    fn get_logs_snapshot(&self, since: Option<std::time::SystemTime>) -> LoggingResult<Vec<LogEntry>>;
    fn get_metrics_snapshot(&self) -> LoggingResult<MetricsSnapshot>;
    fn get_traces_snapshot(&self, since: Option<std::time::SystemTime>) -> LoggingResult<Vec<TraceEntry>>;
}

/// Default observability exporter implementation
pub struct DefaultObservabilityExporter {
    log_buffer: Arc<Mutex<Vec<LogEntry>>>,
    metrics_registry: Arc<Mutex<DefaultMetricsRegistry>>,
    trace_buffer: Arc<Mutex<Vec<TraceEntry>>>,
    custom_exporters: HashMap<String, Box<dyn CustomExporter>>,
    max_buffer_size: usize,
}

impl DefaultObservabilityExporter {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            log_buffer: Arc::new(Mutex::new(Vec::new())),
            metrics_registry: Arc::new(Mutex::new(DefaultMetricsRegistry::new())),
            trace_buffer: Arc::new(Mutex::new(Vec::new())),
            custom_exporters: HashMap::new(),
            max_buffer_size,
        }
    }

    pub fn add_log_entry(&self, entry: LogEntry) -> LoggingResult<()> {
        if let Ok(mut buffer) = self.log_buffer.lock() {
            buffer.push(entry);
            
            // Maintain buffer size limit
            if buffer.len() > self.max_buffer_size {
                let excess = buffer.len() - self.max_buffer_size;
                buffer.drain(0..excess);
            }
            
            Ok(())
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire log buffer lock".to_string()))
        }
    }

    pub fn add_trace_entry(&self, entry: TraceEntry) -> LoggingResult<()> {
        if let Ok(mut buffer) = self.trace_buffer.lock() {
            buffer.push(entry);
            
            // Maintain buffer size limit
            if buffer.len() > self.max_buffer_size {
                let excess = buffer.len() - self.max_buffer_size;
                buffer.drain(0..excess);
            }
            
            Ok(())
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire trace buffer lock".to_string()))
        }
    }

    fn export_to_file(&self, data: &str, path: &std::path::Path) -> LoggingResult<()> {
        std::fs::write(path, data)
            .map_err(|e| LoggingError::ExportError(format!("Failed to write to file: {e}")))
    }

    fn export_to_console(&self, data: &str) -> LoggingResult<()> {
        println!("{data}");
        Ok(())
    }

    fn format_logs(&self, logs: &[LogEntry], format: LogFormat) -> LoggingResult<String> {
        match format {
            LogFormat::Json => {
                serde_json::to_string_pretty(logs)
                    .map_err(|e| LoggingError::InvalidFormat(format!("JSON serialization failed: {e}")))
            }
            LogFormat::Text => {
                let mut output = String::new();
                for log in logs {
                    output.push_str(&format!(
                        "[{}] {} - {} (correlation_id: {})\n",
                        log.timestamp.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        log.level,
                        log.message,
                        log.context.correlation_id
                    ));
                }
                Ok(output)
            }
            LogFormat::Structured => {
                let mut output = String::new();
                for log in logs {
                    output.push_str(&format!(
                        "timestamp={:?} level={} message=\"{}\" correlation_id={} node_id={} module_hash={} function_id={:?}",
                        log.timestamp,
                        log.level,
                        log.message,
                        log.context.correlation_id,
                        log.context.node_id.as_deref().unwrap_or("unknown"),
                        log.context.module_hash.as_deref().unwrap_or("unknown"),
                        log.context.function_id
                    ));
                    
                    for (key, value) in &log.fields {
                        output.push_str(&format!(" {key}={value}"));
                    }
                    
                    output.push('\n');
                }
                Ok(output)
            }
            LogFormat::OpenTelemetry => {
                // Convert to OpenTelemetry format (simplified)
                serde_json::to_string_pretty(logs)
                    .map_err(|e| LoggingError::InvalidFormat(format!("OpenTelemetry serialization failed: {e}")))
            }
        }
    }

    fn format_metrics(&self, metrics: &MetricsSnapshot, format: MetricsFormat) -> LoggingResult<String> {
        match format {
            MetricsFormat::Json => {
                serde_json::to_string_pretty(metrics)
                    .map_err(|e| LoggingError::InvalidFormat(format!("JSON serialization failed: {e}")))
            }
            MetricsFormat::Prometheus => {
                let mut output = String::new();
                
                // Export counters
                for counter in &metrics.counters {
                    output.push_str(&format!("# HELP {} {}\n", counter.name, counter.description));
                    output.push_str(&format!("# TYPE {} counter\n", counter.name));
                    output.push_str(&format!("{} {}\n", counter.name, counter.value));
                }
                
                // Export gauges
                for gauge in &metrics.gauges {
                    output.push_str(&format!("# HELP {} {}\n", gauge.name, gauge.description));
                    output.push_str(&format!("# TYPE {} gauge\n", gauge.name));
                    output.push_str(&format!("{} {}\n", gauge.name, gauge.value));
                }
                
                // Export histograms
                for histogram in &metrics.histograms {
                    output.push_str(&format!("# HELP {} {}\n", histogram.name, histogram.description));
                    output.push_str(&format!("# TYPE {} histogram\n", histogram.name));
                    
                    for (i, &bucket) in histogram.buckets.iter().enumerate() {
                        output.push_str(&format!("{}{{le=\"{}\"}} {}\n", 
                            histogram.name, bucket, histogram.bucket_counts[i]));
                    }
                    
                    output.push_str(&format!("{}{{le=\"+Inf\"}} {}\n", 
                        histogram.name, histogram.bucket_counts.last().unwrap_or(&0)));
                    output.push_str(&format!("{}_sum {}\n", histogram.name, histogram.sum));
                    output.push_str(&format!("{}_count {}\n", histogram.name, histogram.count));
                }
                
                Ok(output)
            }
            MetricsFormat::OpenTelemetry => {
                // Convert to OpenTelemetry format (simplified)
                serde_json::to_string_pretty(metrics)
                    .map_err(|e| LoggingError::InvalidFormat(format!("OpenTelemetry serialization failed: {e}")))
            }
        }
    }
}

impl ObservabilityExporter for DefaultObservabilityExporter {
    fn export_logs(&self, format: LogFormat, destination: ExportDestination) -> LoggingResult<()> {
        let logs = self.get_logs_snapshot(None)?;
        let formatted_data = self.format_logs(&logs, format)?;
        
        match destination {
            ExportDestination::File(path) => self.export_to_file(&formatted_data, &path),
            ExportDestination::Console => self.export_to_console(&formatted_data),
            ExportDestination::Network(_endpoint) => {
                // TODO: Implement network export
                Err(LoggingError::ExportError("Network export not yet implemented".to_string()))
            }
            ExportDestination::Custom(name) => {
                if let Some(exporter) = self.custom_exporters.get(&name) {
                    exporter.export_logs(&logs, format)
                } else {
                    Err(LoggingError::ExportError(format!("Custom exporter '{name}' not found")))
                }
            }
        }
    }

    fn export_metrics(&self, format: MetricsFormat, destination: ExportDestination) -> LoggingResult<()> {
        let metrics = self.get_metrics_snapshot()?;
        let formatted_data = self.format_metrics(&metrics, format)?;
        
        match destination {
            ExportDestination::File(path) => self.export_to_file(&formatted_data, &path),
            ExportDestination::Console => self.export_to_console(&formatted_data),
            ExportDestination::Network(_endpoint) => {
                // TODO: Implement network export
                Err(LoggingError::ExportError("Network export not yet implemented".to_string()))
            }
            ExportDestination::Custom(name) => {
                if let Some(exporter) = self.custom_exporters.get(&name) {
                    exporter.export_metrics(&metrics, format)
                } else {
                    Err(LoggingError::ExportError(format!("Custom exporter '{name}' not found")))
                }
            }
        }
    }

    fn export_traces(&self, format: TraceFormat, destination: ExportDestination) -> LoggingResult<()> {
        let traces = self.get_traces_snapshot(None)?;
        let formatted_data = serde_json::to_string_pretty(&traces)
            .map_err(|e| LoggingError::InvalidFormat(format!("Trace serialization failed: {e}")))?;
        
        match destination {
            ExportDestination::File(path) => self.export_to_file(&formatted_data, &path),
            ExportDestination::Console => self.export_to_console(&formatted_data),
            ExportDestination::Network(_endpoint) => {
                // TODO: Implement network export
                Err(LoggingError::ExportError("Network export not yet implemented".to_string()))
            }
            ExportDestination::Custom(name) => {
                if let Some(exporter) = self.custom_exporters.get(&name) {
                    exporter.export_traces(&traces, format)
                } else {
                    Err(LoggingError::ExportError(format!("Custom exporter '{name}' not found")))
                }
            }
        }
    }

    fn register_custom_exporter(&mut self, name: String, exporter: Box<dyn CustomExporter>) -> LoggingResult<()> {
        self.custom_exporters.insert(name, exporter);
        Ok(())
    }

    fn get_logs_snapshot(&self, since: Option<std::time::SystemTime>) -> LoggingResult<Vec<LogEntry>> {
        if let Ok(buffer) = self.log_buffer.lock() {
            let logs = if let Some(since_time) = since {
                buffer.iter()
                    .filter(|entry| entry.timestamp >= since_time)
                    .cloned()
                    .collect()
            } else {
                buffer.clone()
            };
            Ok(logs)
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire log buffer lock".to_string()))
        }
    }

    fn get_metrics_snapshot(&self) -> LoggingResult<MetricsSnapshot> {
        if let Ok(registry) = self.metrics_registry.lock() {
            let counters = registry.list_counters()
                .iter()
                .map(|counter| CounterSnapshot {
                    id: counter.id.to_string(),
                    name: counter.name.clone(),
                    description: counter.description.clone(),
                    value: counter.get_value(),
                    labels: HashMap::new(), // TODO: Extract labels from labeled_metrics
                })
                .collect();

            let gauges = registry.list_gauges()
                .iter()
                .map(|gauge| GaugeSnapshot {
                    id: gauge.id.to_string(),
                    name: gauge.name.clone(),
                    description: gauge.description.clone(),
                    value: gauge.get_value(),
                    labels: HashMap::new(), // TODO: Extract labels from labeled_metrics
                })
                .collect();

            let histograms = registry.list_histograms()
                .iter()
                .map(|histogram| HistogramSnapshot {
                    id: histogram.id.to_string(),
                    name: histogram.name.clone(),
                    description: histogram.description.clone(),
                    buckets: histogram.buckets.clone(),
                    bucket_counts: histogram.get_bucket_counts(),
                    sum: histogram.get_sum(),
                    count: histogram.get_count(),
                    labels: HashMap::new(), // TODO: Extract labels from labeled_metrics
                })
                .collect();

            Ok(MetricsSnapshot {
                timestamp: std::time::SystemTime::now(),
                counters,
                gauges,
                histograms,
            })
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire metrics registry lock".to_string()))
        }
    }

    fn get_traces_snapshot(&self, since: Option<std::time::SystemTime>) -> LoggingResult<Vec<TraceEntry>> {
        if let Ok(buffer) = self.trace_buffer.lock() {
            let traces = if let Some(since_time) = since {
                buffer.iter()
                    .filter(|entry| entry.start_time >= since_time)
                    .cloned()
                    .collect()
            } else {
                buffer.clone()
            };
            Ok(traces)
        } else {
            Err(LoggingError::InvalidMetricOperation("Failed to acquire trace buffer lock".to_string()))
        }
    }
}

/// Comprehensive logging system that combines all components
pub struct LoggingSystem {
    pub logger: Box<dyn Logger>,
    pub function_spanner: FunctionSpanner,
    pub metrics_registry: Arc<Mutex<DefaultMetricsRegistry>>,
    pub exporter: DefaultObservabilityExporter,
}

impl LoggingSystem {
    pub fn new(span_config: FunctionSpanConfig, max_buffer_size: usize) -> Self {
        let metrics_registry = Arc::new(Mutex::new(DefaultMetricsRegistry::new()));
        
        Self {
            logger: Box::new(StructuredLogger::new()),
            function_spanner: FunctionSpanner::new(span_config),
            metrics_registry: metrics_registry.clone(),
            exporter: DefaultObservabilityExporter::new(max_buffer_size),
        }
    }

    pub fn with_context(&self, context: LogContext) -> Box<dyn Logger> {
        self.logger.with_context(context)
    }

    pub fn get_metrics_registry(&self) -> Arc<Mutex<DefaultMetricsRegistry>> {
        self.metrics_registry.clone()
    }

    pub fn start_function_span(&self, function_id: FunctionId, args: Option<&[serde_json::Value]>) -> Option<Span> {
        self.function_spanner.start_function_span(function_id, args)
    }

    pub fn end_function_span(&self, function_id: FunctionId, return_value: Option<&serde_json::Value>) {
        self.function_spanner.end_function_span(function_id, return_value)
    }

    pub fn export_logs(&self, format: LogFormat, destination: ExportDestination) -> LoggingResult<()> {
        self.exporter.export_logs(format, destination)
    }

    pub fn export_metrics(&self, format: MetricsFormat, destination: ExportDestination) -> LoggingResult<()> {
        self.exporter.export_metrics(format, destination)
    }

    pub fn export_traces(&self, format: TraceFormat, destination: ExportDestination) -> LoggingResult<()> {
        self.exporter.export_traces(format, destination)
    }
}

impl Default for LoggingSystem {
    fn default() -> Self {
        Self::new(FunctionSpanConfig::default(), 10000)
    }
}