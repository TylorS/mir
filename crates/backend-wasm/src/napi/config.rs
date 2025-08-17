//! Configuration builders for WASM instance management

use crate::OptimizationLevel;

/// Builder for WASM instance manager configuration
#[derive(Debug, Clone)]
pub struct InstanceManagerConfigBuilder {
    max_instances: u32,
    memory_limit_mb: u32,
    execution_timeout_ms: u32,
    enable_hot_reload: bool,
    preserve_state_on_reload: bool,
    enable_debugging: bool,
}

impl Default for InstanceManagerConfigBuilder {
    fn default() -> Self {
        Self {
            max_instances: 10,
            memory_limit_mb: 64,
            execution_timeout_ms: 5000,
            enable_hot_reload: true,
            preserve_state_on_reload: true,
            enable_debugging: false,
        }
    }
}

impl InstanceManagerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn max_instances(mut self, max: u32) -> Self {
        self.max_instances = max;
        self
    }
    
    pub fn memory_limit_mb(mut self, limit: u32) -> Self {
        self.memory_limit_mb = limit;
        self
    }
    
    pub fn execution_timeout_ms(mut self, timeout: u32) -> Self {
        self.execution_timeout_ms = timeout;
        self
    }
    
    pub fn enable_hot_reload(mut self, enable: bool) -> Self {
        self.enable_hot_reload = enable;
        self
    }
    
    pub fn preserve_state_on_reload(mut self, preserve: bool) -> Self {
        self.preserve_state_on_reload = preserve;
        self
    }
    
    pub fn enable_debugging(mut self, enable: bool) -> Self {
        self.enable_debugging = enable;
        self
    }
    
    pub fn build(self) -> InstanceManagerConfig {
        InstanceManagerConfig {
            max_instances: self.max_instances,
            memory_limit_mb: self.memory_limit_mb,
            execution_timeout_ms: self.execution_timeout_ms,
            enable_hot_reload: self.enable_hot_reload,
            preserve_state_on_reload: self.preserve_state_on_reload,
            enable_debugging: self.enable_debugging,
        }
    }
}

/// Configuration for WASM instance manager
#[derive(Debug, Clone)]
pub struct InstanceManagerConfig {
    pub max_instances: u32,
    pub memory_limit_mb: u32,
    pub execution_timeout_ms: u32,
    pub enable_hot_reload: bool,
    pub preserve_state_on_reload: bool,
    pub enable_debugging: bool,
}

impl InstanceManagerConfig {
    pub fn builder() -> InstanceManagerConfigBuilder {
        InstanceManagerConfigBuilder::new()
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.max_instances == 0 {
            return Err("max_instances must be greater than 0".to_string());
        }
        
        if self.memory_limit_mb == 0 {
            return Err("memory_limit_mb must be greater than 0".to_string());
        }
        
        if self.execution_timeout_ms == 0 {
            return Err("execution_timeout_ms must be greater than 0".to_string());
        }
        
        Ok(())
    }
}