# MIR Runtime API Reference

This document provides comprehensive API documentation for the MIR Runtime Assembly.

## Table of Contents

1. [Core Types](#core-types)
2. [MirRuntime](#mirruntime)
3. [RuntimeConfig](#runtimeconfig)
4. [Backend Management](#backend-management)
5. [Lifecycle Management](#lifecycle-management)
6. [Error Types](#error-types)
7. [Observability](#observability)
8. [Examples](#examples)

## Core Types

### ContentHash

Represents a content-addressable hash for modules and data.

```rust
pub struct ContentHash([u8; 32]);

impl ContentHash {
    pub fn new(data: &[u8]) -> Self;
    pub fn from_hex(hex: &str) -> Result<Self, ParseError>;
    pub fn to_hex(&self) -> String;
    pub fn as_bytes(&self) -> &[u8; 32];
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
```

### RuntimeState

Represents the current state of the runtime.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Initializing,
    Starting,
    Running,
    ShuttingDown,
    Stopped,
    Error,
}
```

### Environment

Configuration for different deployment environments.

```rust
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
```

## MirRuntime

The main runtime class that coordinates all components.

### Constructor

```rust
impl MirRuntime {
    /// Create a new MIR runtime with the given configuration
    pub async fn new(config: RuntimeConfig) -> RuntimeResult<Self>;
}
```

### Lifecycle Methods

```rust
impl MirRuntime {
    /// Start the runtime and all its components
    pub async fn start(&mut self) -> RuntimeResult<()>;
    
    /// Stop the runtime gracefully
    pub async fn shutdown(&mut self) -> RuntimeResult<()>;
    
    /// Get the current runtime state
    pub async fn state(&self) -> RuntimeState;
}
```

### Execution Methods

```rust
impl MirRuntime {
    /// Execute a module by content hash
    pub async fn execute_module(&self, module_hash: ContentHash) -> RuntimeResult<serde_json::Value>;
    
    /// Hot-reload a module
    pub async fn hot_reload_module(&self, old_hash: ContentHash, new_module_data: &[u8]) -> RuntimeResult<()>;
}
```

### Information Methods

```rust
impl MirRuntime {
    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig;
    
    /// Get access to core components (for advanced usage)
    pub fn components(&self) -> &RuntimeComponents;
    
    /// Get runtime statistics and health information
    pub async fn get_runtime_info(&self) -> RuntimeInfo;
}
```

### RuntimeInfo

```rust
#[derive(Debug, Clone)]
pub struct RuntimeInfo {
    pub state: RuntimeState,
    pub version: String,
    pub uptime: std::time::Duration,
    pub backend_info: HashMap<String, serde_json::Value>,
    pub component_health: HashMap<String, ComponentHealth>,
}
```

### ComponentHealth

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

## RuntimeConfig

Configuration for the runtime.

### Constructor

```rust
impl RuntimeConfig {
    /// Create a new runtime configuration with defaults
    pub fn new() -> Self;
    
    /// Create a configuration builder
    pub fn builder() -> RuntimeConfigBuilder;
    
    /// Load configuration from JSON file
    pub fn from_file(path: &str) -> Result<Self, ConfigError>;
    
    /// Save configuration to JSON file
    pub fn to_file(&self, path: &str) -> Result<(), ConfigError>;
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError>;
}
```

### RuntimeConfigBuilder

```rust
impl RuntimeConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self;
    
    /// Set the environment
    pub fn with_environment(self, environment: Environment) -> Self;
    
    /// Enable a backend
    pub fn with_backend(self, backend_type: &str) -> Self;
    
    /// Set backend configuration
    pub fn with_backend_config(self, name: String, config: BackendConfig) -> Self;
    
    /// Set HMR configuration
    pub fn with_hmr_config(self, hmr_config: HMRConfig) -> Self;
    
    /// Set telemetry configuration
    pub fn with_telemetry_config(self, telemetry_config: InstrumentationConfig) -> Self;
    
    /// Set development tools configuration
    pub fn with_dev_tools_config(self, dev_tools_config: DevToolsConfig) -> Self;
    
    /// Set consensus configuration
    pub fn with_consensus_config(self, consensus_config: ConsensusConfig) -> Self;
    
    /// Set runtime settings
    pub fn with_runtime_settings(self, runtime_settings: RuntimeSettings) -> Self;
    
    /// Set maximum concurrent executions
    pub fn with_max_concurrent_executions(self, max: usize) -> Self;
    
    /// Set execution timeout
    pub fn with_execution_timeout(self, timeout: Duration) -> Self;
    
    /// Enable or disable sandbox
    pub fn with_sandbox(self, enabled: bool) -> Self;
    
    /// Build the configuration
    pub fn build(self) -> Result<RuntimeConfig, ConfigError>;
}
```

### BackendConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub backend_type: String,
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub optimization_level: OptimizationLevel,
    pub include_debug_info: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Size,
    Speed,
}
```

### RuntimeSettings

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSettings {
    pub max_concurrent_executions: usize,
    pub execution_timeout: Duration,
    pub memory_limits: MemoryLimits,
    pub gc_settings: GCSettings,
    pub thread_pool: ThreadPoolConfig,
    pub security: SecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    pub max_heap_size: usize,
    pub max_stack_size: usize,
    pub pressure_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCSettings {
    pub strategy: GCStrategy,
    pub trigger_threshold: f64,
    pub max_pause_time: Duration,
    pub concurrent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GCStrategy {
    MarkAndSweep,
    Generational,
    Incremental,
    Concurrent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub enable_sandbox: bool,
    pub allowed_syscalls: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub capability_based: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_file_descriptors: usize,
    pub max_network_connections: usize,
    pub max_cpu_time: Duration,
    pub max_memory_per_execution: usize,
}
```

## Backend Management

### BackendManager

```rust
impl BackendManager {
    /// Create a new backend manager
    pub fn new() -> Self;
    
    /// Start the backend manager with the given configurations
    pub async fn start(&mut self, configs: &HashMap<String, BackendConfig>) -> Result<(), BackendError>;
    
    /// Compile a module using the specified backend
    pub async fn compile_module(&self, backend_type: BackendType, module: &Module) -> Result<CompiledModule, BackendError>;
    
    /// Execute a compiled module using the specified backend
    pub async fn execute_module(&self, backend_type: BackendType, compiled: &CompiledModule) -> Result<ExecutionResult, BackendError>;
    
    /// Hot-reload a module using the specified backend
    pub async fn hot_reload_module(&self, backend_type: BackendType, old_hash: ContentHash, new_module: &CompiledModule) -> Result<ReloadResult, BackendError>;
    
    /// Get the default backend type
    pub async fn default_backend(&self) -> Option<BackendType>;
    
    /// Get information about all backends
    pub async fn get_info(&self) -> HashMap<String, serde_json::Value>;
    
    /// Get available backend types
    pub async fn available_backends(&self) -> Vec<BackendType>;
    
    /// Shutdown all backends
    pub async fn shutdown(&mut self) -> Result<(), BackendError>;
}
```

### BackendType

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendType {
    Wasm,
    JavaScript,
    Wasi,
}

impl std::fmt::Display for BackendType;
impl std::str::FromStr for BackendType;
```

### CompiledModule

```rust
#[derive(Debug, Clone)]
pub struct CompiledModule {
    pub module_hash: ContentHash,
    pub compiled_data: Vec<u8>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub source_map: Option<Vec<u8>>,
}
```

### ExecutionResult

```rust
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub return_value: serde_json::Value,
    pub stats: ExecutionStats,
    pub output: Vec<u8>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub execution_time: std::time::Duration,
    pub memory_usage: usize,
    pub instructions_executed: u64,
    pub function_calls: u64,
}
```

### ReloadResult

```rust
#[derive(Debug, Clone)]
pub struct ReloadResult {
    pub success: bool,
    pub state_preserved: bool,
    pub messages: Vec<String>,
    pub rollback_info: Option<RollbackInfo>,
}

#[derive(Debug, Clone)]
pub struct RollbackInfo {
    pub previous_hash: ContentHash,
    pub rollback_success: bool,
    pub rollback_error: Option<String>,
}
```

## Lifecycle Management

### LifecycleManager

```rust
impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self;
    
    /// Start the lifecycle manager
    pub async fn start(&mut self) -> Result<(), LifecycleError>;
    
    /// Stop the lifecycle manager
    pub async fn stop(&mut self) -> Result<(), LifecycleError>;
    
    /// Get the current runtime state
    pub async fn current_state(&self) -> RuntimeState;
    
    /// Transition to a new state
    pub async fn transition_state(&self, new_state: RuntimeState) -> Result<(), LifecycleError>;
    
    /// Emit a lifecycle event
    pub async fn emit_event(&self, event: LifecycleEvent);
    
    /// Subscribe to lifecycle events
    pub fn subscribe(&self) -> broadcast::Receiver<LifecycleEvent>;
    
    /// Register an event handler
    pub async fn register_handler(&self, handler: Box<dyn LifecycleEventHandler>);
    
    /// Get runtime uptime
    pub fn uptime(&self) -> Duration;
}
```

### LifecycleEvent

```rust
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    RuntimeStarted,
    RuntimeShuttingDown,
    RuntimeStopped,
    ComponentStarted { component: String },
    ComponentStopped { component: String },
    ComponentError { component: String, error: String },
    StateTransition { from: RuntimeState, to: RuntimeState },
    Custom { event_type: String, data: serde_json::Value },
}
```

### LifecycleEventHandler

```rust
pub trait LifecycleEventHandler: Send + Sync {
    /// Handle a lifecycle event
    fn handle_event(&self, event: &LifecycleEvent);
    
    /// Get the handler name
    fn name(&self) -> &str;
}
```

## Error Types

### RuntimeError

```rust
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Component error: {0}")]
    ComponentError(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
    
    #[error("Lifecycle error: {0}")]
    LifecycleError(String),
    
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Runtime is not in the expected state: expected {expected}, found {actual}")]
    InvalidState { expected: String, actual: String },
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
```

### ConfigError

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    
    #[error("Backend configuration error: {0}")]
    BackendError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

### BackendError

```rust
#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Backend not found: {backend_type:?}")]
    BackendNotFound { backend_type: BackendType },
    
    #[error("Backend not initialized: {backend_type:?}")]
    BackendNotInitialized { backend_type: BackendType },
    
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Hot-reload failed: {0}")]
    HotReloadFailed(String),
    
    #[error("Backend initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Backend configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Backend is busy")]
    BackendBusy,
    
    #[error("Backend error: {0}")]
    BackendError(String),
}
```

### LifecycleError

```rust
#[derive(Error, Debug)]
pub enum LifecycleError {
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: RuntimeState, to: RuntimeState },
    
    #[error("Event handling error: {0}")]
    EventHandlingError(String),
    
    #[error("Lifecycle manager not started")]
    NotStarted,
    
    #[error("Lifecycle manager already started")]
    AlreadyStarted,
}
```

## Observability

### InstrumentationConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationConfig {
    pub auto_instrument: bool,
    pub sampling_rate: f64,
    pub export_endpoint: String,
    pub service_name: String,
    pub service_version: String,
    pub enable_metrics: bool,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for InstrumentationConfig;
```

### DevToolsConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsConfig {
    pub file_watcher_enabled: bool,
    pub watch_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub debounce_ms: u64,
    pub build_tool_integration: BuildToolIntegration,
    pub version_control_integration: VersionControlIntegration,
    pub ci_cd_integration: CICDIntegration,
}

impl Default for DevToolsConfig;
```

## Examples

### Basic Runtime Setup

```rust
use mir_runtime_assembly::{MirRuntime, RuntimeConfig, Environment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuntimeConfig::builder()
        .with_environment(Environment::Development {
            auto_reload: true,
            file_watcher_enabled: true,
            aggressive_optimization: false,
        })
        .with_backend("wasm")
        .build()?;

    let mut runtime = MirRuntime::new(config).await?;
    runtime.start().await?;

    // Use the runtime...

    runtime.shutdown().await?;
    Ok(())
}
```

### Production Configuration

```rust
use mir_runtime_assembly::{
    RuntimeConfig, Environment, HMRConfig, UpdatePolicy, ValidationLevel,
    RuntimeSettings, MemoryLimits, SecuritySettings, ResourceLimits
};
use std::time::Duration;

let config = RuntimeConfig::builder()
    .with_environment(Environment::Production {
        require_explicit_deployment: true,
        staged_rollout: true,
        canary_percentage: 10.0,
    })
    .with_hmr_config(HMRConfig {
        update_policy: UpdatePolicy::Manual,
        validation_level: ValidationLevel::Extensive,
        ..Default::default()
    })
    .with_runtime_settings(RuntimeSettings {
        max_concurrent_executions: 16,
        execution_timeout: Duration::from_secs(120),
        memory_limits: MemoryLimits {
            max_heap_size: 2 * 1024 * 1024 * 1024, // 2GB
            max_stack_size: 16 * 1024 * 1024,      // 16MB
            pressure_threshold: 0.85,
        },
        security: SecuritySettings {
            enable_sandbox: true,
            resource_limits: ResourceLimits {
                max_file_descriptors: 4096,
                max_network_connections: 1000,
                max_cpu_time: Duration::from_secs(300),
                max_memory_per_execution: 512 * 1024 * 1024, // 512MB
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .build()?;
```

### Hot-Reload Example

```rust
use mir_runtime_assembly::{MirRuntime, ContentHash};

async fn hot_reload_example(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    // Get the current module hash
    let old_hash = ContentHash::from_hex("abc123...")?;
    
    // Load new module data
    let new_module_data = std::fs::read("updated_module.mir")?;
    
    // Perform hot-reload
    match runtime.hot_reload_module(old_hash, &new_module_data).await {
        Ok(()) => println!("Hot-reload successful"),
        Err(e) => println!("Hot-reload failed: {}", e),
    }
    
    Ok(())
}
```

### Event Handling Example

```rust
use mir_runtime_assembly::{LifecycleEvent, LifecycleEventHandler};

struct CustomEventHandler;

impl LifecycleEventHandler for CustomEventHandler {
    fn handle_event(&self, event: &LifecycleEvent) {
        match event {
            LifecycleEvent::RuntimeStarted => {
                println!("Runtime has started!");
            }
            LifecycleEvent::ComponentError { component, error } => {
                eprintln!("Component {} error: {}", component, error);
            }
            _ => {}
        }
    }
    
    fn name(&self) -> &str {
        "CustomEventHandler"
    }
}

// Register the handler
runtime.lifecycle_manager()
    .register_handler(Box::new(CustomEventHandler))
    .await;
```

### Error Handling Example

```rust
use mir_runtime_assembly::{RuntimeError, RuntimeResult};

async fn handle_runtime_errors(runtime: &MirRuntime) -> RuntimeResult<()> {
    match runtime.execute_module(module_hash).await {
        Ok(result) => {
            println!("Execution successful: {:?}", result);
            Ok(())
        }
        Err(RuntimeError::ComponentError(msg)) => {
            eprintln!("Component error: {}", msg);
            // Handle component error
            Err(RuntimeError::ComponentError(msg))
        }
        Err(RuntimeError::BackendError(msg)) => {
            eprintln!("Backend error: {}", msg);
            // Try with different backend
            Err(RuntimeError::BackendError(msg))
        }
        Err(RuntimeError::InvalidState { expected, actual }) => {
            eprintln!("Invalid state: expected {}, found {}", expected, actual);
            // Wait for correct state or restart runtime
            Err(RuntimeError::InvalidState { expected, actual })
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            Err(e)
        }
    }
}
```

This API reference provides comprehensive documentation for all public APIs in the MIR Runtime Assembly. Use this reference to understand the available functionality and how to integrate the runtime into your applications.