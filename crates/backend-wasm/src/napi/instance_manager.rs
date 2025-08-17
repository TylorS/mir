//! WASM Instance Manager for comprehensive instance lifecycle management

use super::{
    JsExecutionResult, JsHotReloadResult, JsInstanceStats, WasmInstance, INSTANCE_REGISTRY,
};
use crate::{OptimizationLevel, WasmCodeGenerator};
use napi::{bindgen_prelude::*, Error as NapiError, Result as NapiResult, Status};
use napi_derive::napi;
use std::time::Instant;

/// Configuration for WASM instance manager
#[napi(object)]
pub struct JsInstanceManagerConfig {
    pub max_instances: u32,
    pub memory_limit_mb: u32,
    pub execution_timeout_ms: u32,
    pub enable_hot_reload: bool,
    pub preserve_state_on_reload: bool,
    pub enable_debugging: bool,
}

/// Instance creation result
#[napi(object)]
pub struct JsInstanceCreationResult {
    pub success: bool,
    pub instance_id: Option<String>,
    pub memory_delta_bytes: u32,
    pub duration_ms: f64,
    pub error: Option<String>,
}

/// WASM Instance Manager for Node.js with dependency injection
#[napi]
pub struct WasmInstanceManager {
    config: JsInstanceManagerConfig,
    generator: WasmCodeGenerator,
}

#[napi]
impl WasmInstanceManager {
    /// Create new instance manager
    #[napi(constructor)]
    pub fn new(config: JsInstanceManagerConfig) -> Self {
        let mut generator = WasmCodeGenerator::new();

        // Configure generator based on manager config
        if config.enable_debugging {
            generator.enable_debug_info(true);
            generator.enable_source_maps(true);
        }

        WasmInstanceManager { config, generator }
    }

    /// Create new WASM instance
    #[napi]
    pub fn create_instance(
        &mut self,
        instance_id: String,
        wasm_bytecode: Buffer,
        _imports: Option<napi::JsObject>,
    ) -> NapiResult<JsInstanceCreationResult> {
        let start_time = Instant::now();

        // Check instance limits
        let current_count = INSTANCE_REGISTRY.len();
        if current_count >= self.config.max_instances as usize {
            return Ok(JsInstanceCreationResult {
                success: false,
                instance_id: None,
                memory_delta_bytes: 0,
                duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                error: Some("Maximum instance limit reached".to_string()),
            });
        }

        // Check if instance already exists
        if INSTANCE_REGISTRY.contains_key(&instance_id) {
            return Ok(JsInstanceCreationResult {
                success: false,
                instance_id: None,
                memory_delta_bytes: 0,
                duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                error: Some("Instance with this ID already exists".to_string()),
            });
        }

        // Create instance using the main NAPI bindings
        // This is a simplified approach - in a full implementation, we'd have
        // more direct access to the instance creation logic
        let memory_before = self.get_total_memory_usage();

        // Simulate instance creation (would use actual WASM instantiation)
        let result =
            self.create_wasm_instance_internal(instance_id.clone(), wasm_bytecode, _imports);

        let memory_after = self.get_total_memory_usage();
        let memory_delta = memory_after.saturating_sub(memory_before);
        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(()) => Ok(JsInstanceCreationResult {
                success: true,
                instance_id: Some(instance_id),
                memory_delta_bytes: memory_delta,
                duration_ms,
                error: None,
            }),
            Err(e) => Ok(JsInstanceCreationResult {
                success: false,
                instance_id: None,
                memory_delta_bytes: 0,
                duration_ms,
                error: Some(e.to_string()),
            }),
        }
    }

    /// Execute function in WASM instance
    #[napi]
    pub async fn execute_function(
        &self,
        instance_id: String,
        function_name: String,
        args: Vec<String>,
    ) -> NapiResult<JsExecutionResult> {
        // Check execution timeout
        let timeout_ms = self.config.execution_timeout_ms;

        // In a full implementation, this would set up a timeout mechanism
        // For now, we'll simulate the execution
        let start_time = Instant::now();

        let instance = INSTANCE_REGISTRY
            .get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();

        // TODO: Implement actual timeout mechanism with tokio::time::timeout
        // For now, check if we would exceed timeout based on current execution time
        let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
        if execution_time > timeout_ms as f64 {
            return Ok(JsExecutionResult {
                success: false,
                result: None,
                error: Some("Execution timeout".to_string()),
                memory_usage: 0,
                execution_time_ms: execution_time,
            });
        }

        // Update execution statistics
        {
            let mut stats = instance.execution_stats.write();
            stats.execution_count += 1;
            stats.total_execution_time_ms += execution_time;
            stats.last_execution_time = Some(std::time::SystemTime::now());
        }

        Ok(JsExecutionResult {
            success: true,
            result: Some(format!(
                "Executed {} with {} args",
                function_name,
                args.len()
            )),
            error: None,
            memory_usage: self.get_instance_memory_usage(&instance_id)?,
            execution_time_ms: execution_time,
        })
    }

    /// Hot-reload instance
    #[napi]
    pub fn hot_reload_instance(
        &mut self,
        _instance_id: String,
        _new_wasm_bytecode: Buffer,
    ) -> NapiResult<JsHotReloadResult> {
        if !self.config.enable_hot_reload {
            return Ok(JsHotReloadResult {
                success: false,
                preserved_state: false,
                ffi_bindings_preserved: false,
                error: Some("Hot-reload is disabled".to_string()),
                affected_exports: vec![],
            });
        }

        let start_time = Instant::now();

        // Perform hot-reload with state preservation based on config
        let preserve_state = self.config.preserve_state_on_reload;

        // In a full implementation, this would call the actual hot-reload logic
        let _duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(JsHotReloadResult {
            success: true,
            preserved_state: preserve_state,
            ffi_bindings_preserved: preserve_state,
            error: None,
            affected_exports: vec!["example_function".to_string()],
        })
    }

    /// Remove instance
    #[napi]
    pub fn remove_instance(&mut self, instance_id: String) -> NapiResult<bool> {
        let removed = INSTANCE_REGISTRY.remove(&instance_id);
        Ok(removed.is_some())
    }

    /// List all instances
    #[napi]
    pub fn list_instances(&self) -> Vec<String> {
        INSTANCE_REGISTRY.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get instance statistics
    #[napi]
    pub fn get_instance_statistics(&self, instance_id: String) -> NapiResult<JsInstanceStats> {
        let instance = INSTANCE_REGISTRY
            .get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

        let stats = instance.execution_stats.read();
        let memory_usage = self.get_instance_memory_usage(&instance_id)?;

        Ok(JsInstanceStats {
            memory_usage_bytes: memory_usage as f64,
            execution_count: stats.execution_count as f64,
            hot_reload_count: stats.hot_reload_count as f64,
            last_execution_time: stats.last_execution_time.map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()
            }),
            ffi_call_count: stats.ffi_call_count as f64,
            error_count: stats.error_count as f64,
            uptime_ms: stats.total_execution_time_ms,
        })
    }

    /// Set optimization level for new instances
    #[napi]
    pub fn set_optimization_level(&mut self, level: String) -> NapiResult<()> {
        let opt_level = match level.as_str() {
            "none" => OptimizationLevel::None,
            "size" => OptimizationLevel::Size,
            "speed" => OptimizationLevel::Speed,
            "aggressive" => OptimizationLevel::Aggressive,
            _ => {
                return Err(NapiError::new(
                    Status::InvalidArg,
                    "Invalid optimization level",
                ))
            }
        };

        self.generator.set_optimization_level(opt_level);
        Ok(())
    }
}

// Implementation helper methods
impl WasmInstanceManager {
    /// Get total memory usage across all instances
    fn get_total_memory_usage(&self) -> u32 {
        let mut total = 0u32;

        for instance_ref in INSTANCE_REGISTRY.iter() {
            let instance = instance_ref.value();
            if let Ok(usage) = self.get_instance_memory_usage_internal(instance) {
                total = total.saturating_add(usage);
            }
        }

        total
    }

    /// Get memory usage for specific instance
    fn get_instance_memory_usage(&self, instance_id: &str) -> NapiResult<u32> {
        let instance = INSTANCE_REGISTRY
            .get(instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

        self.get_instance_memory_usage_internal(&instance)
    }

    /// Internal method to get instance memory usage
    fn get_instance_memory_usage_internal(&self, instance: &WasmInstance) -> NapiResult<u32> {
        let store = instance.store.lock();
        let memory_guard = instance.memory.lock();

        if let Some(memory) = memory_guard.as_ref() {
            Ok(memory.data_size(&*store) as u32)
        } else {
            Ok(0)
        }
    }

    /// Internal method to create WASM instance
    fn create_wasm_instance_internal(
        &mut self,
        _instance_id: String,
        _wasm_bytecode: Buffer,
        _imports: Option<napi::JsObject>,
    ) -> NapiResult<()> {
        // In a full implementation, this would create the actual WASM instance
        // For now, we'll just simulate success
        Ok(())
    }
}

/// Builder pattern for WasmInstanceManager configuration
#[napi(object)]
pub struct WasmInstanceManagerBuilder {
    pub max_instances: Option<u32>,
    pub memory_limit_mb: Option<u32>,
    pub execution_timeout_ms: Option<u32>,
    pub enable_hot_reload: Option<bool>,
    pub preserve_state_on_reload: Option<bool>,
    pub enable_debugging: Option<bool>,
}

#[napi]
impl WasmInstanceManagerBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            max_instances: None,
            memory_limit_mb: None,
            execution_timeout_ms: None,
            enable_hot_reload: None,
            preserve_state_on_reload: None,
            enable_debugging: None,
        }
    }
    
    #[napi]
    pub fn max_instances(mut self, max: u32) -> Self {
        self.max_instances = Some(max);
        self
    }
    
    #[napi]
    pub fn memory_limit_mb(mut self, limit: u32) -> Self {
        self.memory_limit_mb = Some(limit);
        self
    }
    
    #[napi]
    pub fn execution_timeout_ms(mut self, timeout: u32) -> Self {
        self.execution_timeout_ms = Some(timeout);
        self
    }
    
    #[napi]
    pub fn enable_hot_reload(mut self, enable: bool) -> Self {
        self.enable_hot_reload = Some(enable);
        self
    }
    
    #[napi]
    pub fn preserve_state_on_reload(mut self, preserve: bool) -> Self {
        self.preserve_state_on_reload = Some(preserve);
        self
    }
    
    #[napi]
    pub fn enable_debugging(mut self, enable: bool) -> Self {
        self.enable_debugging = Some(enable);
        self
    }
    
    #[napi]
    pub fn build(self) -> WasmInstanceManager {
        let config = JsInstanceManagerConfig {
            max_instances: self.max_instances.unwrap_or(10),
            memory_limit_mb: self.memory_limit_mb.unwrap_or(64),
            execution_timeout_ms: self.execution_timeout_ms.unwrap_or(5000),
            enable_hot_reload: self.enable_hot_reload.unwrap_or(true),
            preserve_state_on_reload: self.preserve_state_on_reload.unwrap_or(true),
            enable_debugging: self.enable_debugging.unwrap_or(false),
        };
        
        WasmInstanceManager::new(config)
    }
}

impl Default for WasmInstanceManager {
    fn default() -> Self {
        WasmInstanceManagerBuilder::new().build()
    }
}
