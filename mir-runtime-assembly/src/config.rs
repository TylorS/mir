//! Runtime configuration management
//!
//! This module provides comprehensive configuration for the MIR runtime,
//! including environment-specific settings, backend configurations, and
//! component-specific options.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export core config types
pub use mir_runtime::{
    HMRConfig, Environment, UpdatePolicy, ValidationLevel, RollbackStrategy,
    StatePreservationConfig, EnvironmentAdapter, ConsensusConfig
};
pub use mir_runtime::telemetry::InstrumentationConfig;
pub use mir_runtime::dev_tools::DevToolsConfig;

/// Main runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// HMR-specific configuration
    pub hmr_config: HMRConfig,
    
    /// Backend configurations
    pub backend_configs: HashMap<String, BackendConfig>,
    
    /// Telemetry and observability configuration
    pub telemetry_config: InstrumentationConfig,
    
    /// Development tools integration configuration
    pub dev_tools_config: DevToolsConfig,
    
    /// Consensus protocol configuration for distributed coordination
    pub consensus_config: ConsensusConfig,
    
    /// Runtime-specific settings
    pub runtime_settings: RuntimeSettings,
}

/// Backend-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend type (wasm, js, wasi)
    pub backend_type: String,
    
    /// Whether this backend is enabled
    pub enabled: bool,
    
    /// Backend-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    
    /// Optimization level
    pub optimization_level: OptimizationLevel,
    
    /// Debug information inclusion
    pub include_debug_info: bool,
}

/// Optimization levels for backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Size,
    Speed,
}

/// Runtime-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSettings {
    /// Maximum number of concurrent executions
    pub max_concurrent_executions: usize,
    
    /// Execution timeout
    pub execution_timeout: Duration,
    
    /// Memory limits
    pub memory_limits: MemoryLimits,
    
    /// Garbage collection settings
    pub gc_settings: GCSettings,
    
    /// Thread pool configuration
    pub thread_pool: ThreadPoolConfig,
    
    /// Security settings
    pub security: SecuritySettings,
}

/// Memory limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum heap size in bytes
    pub max_heap_size: usize,
    
    /// Maximum stack size in bytes
    pub max_stack_size: usize,
    
    /// Memory pressure threshold (0.0 to 1.0)
    pub pressure_threshold: f64,
}

/// Garbage collection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCSettings {
    /// GC strategy
    pub strategy: GCStrategy,
    
    /// GC trigger threshold
    pub trigger_threshold: f64,
    
    /// Maximum GC pause time
    pub max_pause_time: Duration,
    
    /// Enable concurrent GC
    pub concurrent: bool,
}

/// Garbage collection strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GCStrategy {
    MarkAndSweep,
    Generational,
    Incremental,
    Concurrent,
}

/// Thread pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPoolConfig {
    /// Core thread count
    pub core_threads: usize,
    
    /// Maximum thread count
    pub max_threads: usize,
    
    /// Thread keep-alive time
    pub keep_alive: Duration,
    
    /// Queue capacity
    pub queue_capacity: usize,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Enable sandboxing
    pub enable_sandbox: bool,
    
    /// Allowed system calls (for WASI)
    pub allowed_syscalls: Vec<String>,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
    
    /// Enable capability-based security
    pub capability_based: bool,
}

/// Resource limits for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum file descriptors
    pub max_file_descriptors: usize,
    
    /// Maximum network connections
    pub max_network_connections: usize,
    
    /// Maximum CPU time per execution
    pub max_cpu_time: Duration,
    
    /// Maximum memory per execution
    pub max_memory_per_execution: usize,
}

/// Runtime configuration builder
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

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

impl RuntimeConfig {
    /// Create a new runtime configuration with defaults
    pub fn new() -> Self {
        Self {
            hmr_config: HMRConfig::default(),
            backend_configs: Self::default_backend_configs(),
            telemetry_config: InstrumentationConfig::default(),
            dev_tools_config: DevToolsConfig::default(),
            consensus_config: ConsensusConfig::default(),
            runtime_settings: RuntimeSettings::default(),
        }
    }
    
    /// Create a configuration builder
    pub fn builder() -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::new()
    }
    
    /// Load configuration from JSON file
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidConfig(format!("Failed to read config file: {}", e)))?;
        
        let config: Self = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to JSON file
    pub fn to_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
            .map_err(|e| ConfigError::InvalidConfig(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate runtime settings
        if self.runtime_settings.max_concurrent_executions == 0 {
            return Err(ConfigError::InvalidConfig(
                "max_concurrent_executions must be greater than 0".to_string()
            ));
        }
        
        if self.runtime_settings.memory_limits.pressure_threshold < 0.0 
            || self.runtime_settings.memory_limits.pressure_threshold > 1.0 {
            return Err(ConfigError::InvalidConfig(
                "pressure_threshold must be between 0.0 and 1.0".to_string()
            ));
        }
        
        // Validate backend configurations
        for (name, backend_config) in &self.backend_configs {
            if backend_config.enabled && backend_config.backend_type.is_empty() {
                return Err(ConfigError::BackendError(
                    format!("Backend '{}' is enabled but has no type specified", name)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get default backend configurations
    fn default_backend_configs() -> HashMap<String, BackendConfig> {
        let mut configs = HashMap::new();
        
        // WASM backend
        configs.insert("wasm".to_string(), BackendConfig {
            backend_type: "wasm".to_string(),
            enabled: true,
            settings: HashMap::new(),
            optimization_level: OptimizationLevel::Basic,
            include_debug_info: true,
        });
        
        // JavaScript backend
        configs.insert("js".to_string(), BackendConfig {
            backend_type: "js".to_string(),
            enabled: true,
            settings: HashMap::new(),
            optimization_level: OptimizationLevel::Basic,
            include_debug_info: true,
        });
        
        // WASI backend
        configs.insert("wasi".to_string(), BackendConfig {
            backend_type: "wasi".to_string(),
            enabled: false, // Disabled by default
            settings: HashMap::new(),
            optimization_level: OptimizationLevel::Basic,
            include_debug_info: true,
        });
        
        configs
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSettings {
    /// Create default runtime settings
    pub fn new() -> Self {
        Self {
            max_concurrent_executions: num_cpus::get() * 2,
            execution_timeout: Duration::from_secs(30),
            memory_limits: MemoryLimits::default(),
            gc_settings: GCSettings::default(),
            thread_pool: ThreadPoolConfig::default(),
            security: SecuritySettings::default(),
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_heap_size: 1024 * 1024 * 1024, // 1GB
            max_stack_size: 8 * 1024 * 1024,   // 8MB
            pressure_threshold: 0.8,
        }
    }
}

impl Default for GCSettings {
    fn default() -> Self {
        Self {
            strategy: GCStrategy::Generational,
            trigger_threshold: 0.7,
            max_pause_time: Duration::from_millis(10),
            concurrent: true,
        }
    }
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            core_threads: num_cpus::get(),
            max_threads: num_cpus::get() * 4,
            keep_alive: Duration::from_secs(60),
            queue_capacity: 1000,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            allowed_syscalls: vec![
                "read".to_string(),
                "write".to_string(),
                "open".to_string(),
                "close".to_string(),
            ],
            resource_limits: ResourceLimits::default(),
            capability_based: true,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_file_descriptors: 1024,
            max_network_connections: 100,
            max_cpu_time: Duration::from_secs(10),
            max_memory_per_execution: 256 * 1024 * 1024, // 256MB
        }
    }
}

impl RuntimeConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::new(),
        }
    }
    
    /// Set the environment
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.config.hmr_config.environment = environment;
        self
    }
    
    /// Enable a backend
    pub fn with_backend(mut self, backend_type: &str) -> Self {
        if let Some(backend_config) = self.config.backend_configs.get_mut(backend_type) {
            backend_config.enabled = true;
        } else {
            self.config.backend_configs.insert(backend_type.to_string(), BackendConfig {
                backend_type: backend_type.to_string(),
                enabled: true,
                settings: HashMap::new(),
                optimization_level: OptimizationLevel::Basic,
                include_debug_info: true,
            });
        }
        self
    }
    
    /// Set backend configuration
    pub fn with_backend_config(mut self, name: String, config: BackendConfig) -> Self {
        self.config.backend_configs.insert(name, config);
        self
    }
    
    /// Set HMR configuration
    pub fn with_hmr_config(mut self, hmr_config: HMRConfig) -> Self {
        self.config.hmr_config = hmr_config;
        self
    }
    
    /// Set telemetry configuration
    pub fn with_telemetry_config(mut self, telemetry_config: InstrumentationConfig) -> Self {
        self.config.telemetry_config = telemetry_config;
        self
    }
    
    /// Set development tools configuration
    pub fn with_dev_tools_config(mut self, dev_tools_config: DevToolsConfig) -> Self {
        self.config.dev_tools_config = dev_tools_config;
        self
    }
    
    /// Set consensus configuration
    pub fn with_consensus_config(mut self, consensus_config: ConsensusConfig) -> Self {
        self.config.consensus_config = consensus_config;
        self
    }
    
    /// Set runtime settings
    pub fn with_runtime_settings(mut self, runtime_settings: RuntimeSettings) -> Self {
        self.config.runtime_settings = runtime_settings;
        self
    }
    
    /// Set maximum concurrent executions
    pub fn with_max_concurrent_executions(mut self, max: usize) -> Self {
        self.config.runtime_settings.max_concurrent_executions = max;
        self
    }
    
    /// Set execution timeout
    pub fn with_execution_timeout(mut self, timeout: Duration) -> Self {
        self.config.runtime_settings.execution_timeout = timeout;
        self
    }
    
    /// Enable or disable sandbox
    pub fn with_sandbox(mut self, enabled: bool) -> Self {
        self.config.runtime_settings.security.enable_sandbox = enabled;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> Result<RuntimeConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}