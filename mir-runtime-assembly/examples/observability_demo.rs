//! Observability Demo
//!
//! This example demonstrates the comprehensive observability features of the MIR runtime,
//! including OpenTelemetry integration, structured logging, and metrics collection.

use mir_runtime_assembly::{
    MirRuntime, RuntimeConfig, Environment, InstrumentationConfig
};
use std::time::Duration;
use tracing::{info, warn, error, debug, span, Level};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize comprehensive logging and tracing
    setup_observability().await?;

    info!("Starting Observability Demo");

    // Create runtime configuration with full observability enabled
    let telemetry_config = InstrumentationConfig {
        auto_instrument: true,
        sampling_rate: 1.0, // Sample 100% for demo
        export_endpoint: "http://localhost:14268/api/traces".to_string(),
        service_name: "mir-runtime-demo".to_string(),
        service_version: "0.1.0".to_string(),
        enable_metrics: true,
        enable_logging: true,
        log_level: "debug".to_string(),
    };

    let config = RuntimeConfig::builder()
        .with_environment(Environment::Development {
            auto_reload: true,
            file_watcher_enabled: true,
            aggressive_optimization: false,
        })
        .with_telemetry_config(telemetry_config)
        .with_backend("wasm")
        .with_backend("js")
        .build()?;

    // Create and start the runtime
    let mut runtime = MirRuntime::new(config).await?;
    
    // Start runtime with tracing
    let _span = span!(Level::INFO, "runtime_startup").entered();
    info!("Starting runtime with full observability");
    runtime.start().await?;
    
    info!("Runtime started successfully");

    // Demonstrate various observability features
    demonstrate_structured_logging().await;
    demonstrate_distributed_tracing(&runtime).await?;
    demonstrate_metrics_collection(&runtime).await?;
    demonstrate_error_reporting(&runtime).await?;
    demonstrate_performance_monitoring(&runtime).await?;

    // Show runtime health and statistics
    show_runtime_health(&runtime).await;

    // Graceful shutdown with observability
    let _shutdown_span = span!(Level::INFO, "runtime_shutdown").entered();
    info!("Shutting down runtime");
    runtime.shutdown().await?;
    
    info!("Observability demo completed successfully!");
    
    // Give time for telemetry to be exported
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    Ok(())
}

/// Set up comprehensive observability stack
async fn setup_observability() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with multiple layers
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    info!("Observability stack initialized");
    Ok(())
}

/// Demonstrate structured logging capabilities
async fn demonstrate_structured_logging() {
    let _span = span!(Level::INFO, "structured_logging_demo").entered();
    
    info!("=== Structured Logging Demo ===");
    
    // Basic structured logging
    info!(
        user_id = "user123",
        action = "login",
        ip_address = "192.168.1.100",
        "User login successful"
    );
    
    // Logging with nested context
    let user_span = span!(Level::INFO, "user_session", user_id = "user123");
    let _enter = user_span.enter();
    
    debug!("Processing user request");
    info!(request_id = "req456", endpoint = "/api/data", "API request received");
    warn!(latency_ms = 150, "Request latency above threshold");
    
    // Error logging with context
    error!(
        error_code = "E001",
        component = "database",
        retry_count = 3,
        "Database connection failed after retries"
    );
    
    info!("Structured logging demonstration complete");
}

/// Demonstrate distributed tracing
async fn demonstrate_distributed_tracing(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let _span = span!(Level::INFO, "distributed_tracing_demo").entered();
    
    info!("=== Distributed Tracing Demo ===");
    
    // Simulate a distributed operation with multiple spans
    let operation_span = span!(Level::INFO, "distributed_operation", operation_id = "op789");
    let _enter = operation_span.enter();
    
    // Span 1: Module compilation
    {
        let _compile_span = span!(Level::INFO, "module_compilation", module_name = "example").entered();
        info!("Starting module compilation");
        tokio::time::sleep(Duration::from_millis(50)).await;
        info!("Module compilation completed");
    }
    
    // Span 2: Module execution
    {
        let _exec_span = span!(Level::INFO, "module_execution", backend = "wasm").entered();
        info!("Starting module execution");
        
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Nested span for specific operation
        {
            let _nested_span = span!(Level::DEBUG, "function_call", function = "main").entered();
            debug!("Executing main function");
            tokio::time::sleep(Duration::from_millis(25)).await;
            debug!("Main function completed");
        }
        
        info!("Module execution completed");
    }
    
    // Span 3: State synchronization (simulated distributed operation)
    {
        let _sync_span = span!(Level::INFO, "state_synchronization", 
                              node_count = 3, 
                              consensus_round = 1).entered();
        info!("Starting distributed state synchronization");
        
        // Simulate consensus protocol
        for node_id in 1..=3 {
            let _node_span = span!(Level::DEBUG, "node_consensus", node_id = node_id).entered();
            debug!("Sending consensus message to node");
            tokio::time::sleep(Duration::from_millis(20)).await;
            debug!("Received consensus response from node");
        }
        
        info!("State synchronization completed");
    }
    
    info!("Distributed tracing demonstration complete");
    Ok(())
}

/// Demonstrate metrics collection
async fn demonstrate_metrics_collection(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let _span = span!(Level::INFO, "metrics_demo").entered();
    
    info!("=== Metrics Collection Demo ===");
    
    // In a real implementation, these would be actual metrics
    // For demo purposes, we'll log what metrics would be collected
    
    info!("Collecting runtime metrics...");
    
    // Execution metrics
    info!(
        metric_type = "counter",
        metric_name = "mir_executions_total",
        value = 42,
        labels = r#"{"backend": "wasm", "status": "success"}"#,
        "Execution counter metric"
    );
    
    info!(
        metric_type = "histogram",
        metric_name = "mir_execution_duration_seconds",
        value = 0.125,
        labels = r#"{"backend": "wasm"}"#,
        "Execution duration histogram"
    );
    
    // HMR metrics
    info!(
        metric_type = "counter",
        metric_name = "mir_hot_reloads_total",
        value = 5,
        labels = r#"{"status": "success", "compatibility": "backward_compatible"}"#,
        "Hot reload counter metric"
    );
    
    info!(
        metric_type = "gauge",
        metric_name = "mir_active_modules",
        value = 12,
        "Active modules gauge metric"
    );
    
    // Distributed metrics
    info!(
        metric_type = "histogram",
        metric_name = "mir_consensus_latency_seconds",
        value = 0.045,
        labels = r#"{"cluster_size": "3"}"#,
        "Consensus latency histogram"
    );
    
    info!(
        metric_type = "gauge",
        metric_name = "mir_cluster_health",
        value = 1.0,
        labels = r#"{"status": "healthy"}"#,
        "Cluster health gauge metric"
    );
    
    // Resource metrics
    info!(
        metric_type = "gauge",
        metric_name = "mir_memory_usage_bytes",
        value = 134217728, // 128MB
        "Memory usage gauge metric"
    );
    
    info!(
        metric_type = "counter",
        metric_name = "mir_gc_runs_total",
        value = 8,
        "Garbage collection counter metric"
    );
    
    info!("Metrics collection demonstration complete");
    Ok(())
}

/// Demonstrate error reporting and debugging
async fn demonstrate_error_reporting(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let _span = span!(Level::INFO, "error_reporting_demo").entered();
    
    info!("=== Error Reporting Demo ===");
    
    // Simulate various types of errors that the runtime might encounter
    
    // Compilation error
    error!(
        error_type = "compilation_error",
        module_name = "broken_module",
        line = 42,
        column = 15,
        error_code = "E0001",
        message = "Undefined variable 'foo'",
        "Module compilation failed"
    );
    
    // Runtime error
    error!(
        error_type = "runtime_error",
        function_name = "divide",
        error_code = "E0002",
        message = "Division by zero",
        stack_trace = "divide() -> main() -> execute_module()",
        "Runtime execution error"
    );
    
    // Hot-reload error
    error!(
        error_type = "hot_reload_error",
        old_module_hash = "abc123",
        new_module_hash = "def456",
        error_code = "E0003",
        message = "Incompatible schema change: removed required field 'id'",
        migration_available = false,
        "Hot reload failed due to breaking change"
    );
    
    // Distributed coordination error
    error!(
        error_type = "consensus_error",
        node_id = "node-2",
        round = 5,
        error_code = "E0004",
        message = "Consensus timeout: node failed to respond",
        cluster_health = "degraded",
        "Distributed consensus failed"
    );
    
    // Recovery action
    info!(
        recovery_action = "rollback",
        previous_version = "v1.2.3",
        rollback_success = true,
        "Automatic recovery completed"
    );
    
    info!("Error reporting demonstration complete");
    Ok(())
}

/// Demonstrate performance monitoring
async fn demonstrate_performance_monitoring(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let _span = span!(Level::INFO, "performance_monitoring_demo").entered();
    
    info!("=== Performance Monitoring Demo ===");
    
    // Simulate performance monitoring data
    
    // Execution performance
    info!(
        performance_metric = "execution_time",
        module_name = "data_processor",
        backend = "wasm",
        duration_ms = 125,
        instructions_executed = 50000,
        memory_peak_mb = 32,
        "Module execution performance"
    );
    
    // Hot-reload performance
    info!(
        performance_metric = "hot_reload_time",
        module_size_kb = 256,
        analysis_time_ms = 15,
        migration_time_ms = 8,
        total_time_ms = 45,
        downtime_ms = 2,
        "Hot reload performance"
    );
    
    // Distributed coordination performance
    info!(
        performance_metric = "consensus_performance",
        cluster_size = 5,
        proposal_size_bytes = 1024,
        round_trip_time_ms = 35,
        total_consensus_time_ms = 120,
        "Consensus protocol performance"
    );
    
    // Resource utilization
    info!(
        performance_metric = "resource_utilization",
        cpu_usage_percent = 15.5,
        memory_usage_mb = 128,
        network_io_kbps = 45,
        disk_io_kbps = 12,
        "System resource utilization"
    );
    
    // Bottleneck detection
    warn!(
        performance_issue = "bottleneck_detected",
        component = "type_registry",
        operation = "type_lookup",
        avg_latency_ms = 25,
        p99_latency_ms = 150,
        recommendation = "Consider adding type lookup cache",
        "Performance bottleneck detected"
    );
    
    info!("Performance monitoring demonstration complete");
    Ok(())
}

/// Show runtime health and statistics
async fn show_runtime_health(runtime: &MirRuntime) {
    let _span = span!(Level::INFO, "runtime_health_check").entered();
    
    info!("=== Runtime Health Check ===");
    
    let runtime_info = runtime.get_runtime_info().await;
    
    info!(
        runtime_state = ?runtime_info.state,
        runtime_version = %runtime_info.version,
        uptime_seconds = runtime_info.uptime.as_secs(),
        "Runtime status"
    );
    
    // Component health
    for (component, health) in &runtime_info.component_health {
        match health {
            mir_runtime_assembly::ComponentHealth::Healthy => {
                info!(component = %component, status = "healthy", "Component status");
            }
            mir_runtime_assembly::ComponentHealth::Degraded => {
                warn!(component = %component, status = "degraded", "Component status");
            }
            mir_runtime_assembly::ComponentHealth::Unhealthy => {
                error!(component = %component, status = "unhealthy", "Component status");
            }
            mir_runtime_assembly::ComponentHealth::Unknown => {
                warn!(component = %component, status = "unknown", "Component status");
            }
        }
    }
    
    // Backend information
    for (backend, info) in &runtime_info.backend_info {
        info!(
            backend = %backend,
            backend_info = ?info,
            "Backend status"
        );
    }
    
    info!("Runtime health check complete");
}