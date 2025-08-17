//! FFI Bridge for host environment interaction

use super::{JsFFICallResult, JsFFIStats, FFIBinding, INSTANCE_REGISTRY};
use napi::{Result as NapiResult, Error as NapiError, Status};
use napi_derive::napi;
use std::collections::HashMap;
use std::time::{Instant, SystemTime};

/// FFI state preservation result
#[napi(object)]
pub struct JsFFIStatePreservation {
    pub success: bool,
    pub preserved_bindings: Vec<String>,
    pub preserved_state_size: u32,
    pub error: Option<String>,
}

/// Host function configuration
#[derive(Clone)]
#[napi(object)]
pub struct JsHostFunctionConfig {
    pub name: String,
    pub binding_type: String,
    pub is_async: bool,
    pub param_types: Vec<String>,
    pub result_types: Vec<String>,
    pub preserve_state: bool,
}

/// FFI Bridge for Node.js
#[napi]
pub struct FFIBridge {
    host_functions: HashMap<String, JsHostFunctionConfig>,
    call_statistics: HashMap<String, FFICallStats>,
}

/// Internal FFI call statistics
#[derive(Debug, Clone, Default)]
struct FFICallStats {
    total_calls: u64,
    successful_calls: u64,
    failed_calls: u64,
    total_execution_time_ms: f64,
    last_call_time: Option<SystemTime>,
}

#[napi]
impl FFIBridge {
    /// Create new FFI bridge
    #[napi(constructor)]
    pub fn new() -> Self {
        FFIBridge {
            host_functions: HashMap::new(),
            call_statistics: HashMap::new(),
        }
    }

    /// Register host function
    #[napi]
    pub fn register_host_function(
        &mut self,
        config: JsHostFunctionConfig,
        _host_function: JsFunction,
    ) -> NapiResult<bool> {
        // In a full implementation, this would store the actual JavaScript function
        // For now, we'll just store the configuration
        let function_name = config.name.clone();
        
        self.host_functions.insert(function_name.clone(), config);
        self.call_statistics.insert(function_name, FFICallStats::default());
        
        Ok(true)
    }

    /// Call host function from WASM
    #[napi]
    pub fn call_host_function(
        &mut self,
        function_name: String,
        args: Vec<String>,
        instance_id: String,
    ) -> NapiResult<JsFFICallResult> {
        let start_time = Instant::now();

        // Check if function exists
        let config = self.host_functions.get(&function_name)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, format!("Host function '{}' not found", function_name)))?
            .clone();

        // Check if instance exists
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        // Update instance FFI statistics
        {
            let mut stats = wasm_instance.execution_stats.write();
            stats.ffi_call_count += 1;
        }

        // Validate arguments
        if args.len() != config.param_types.len() {
            let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
            
            // Update call statistics
            if let Some(call_stats) = self.call_statistics.get_mut(&function_name) {
                call_stats.total_calls += 1;
                call_stats.failed_calls += 1;
                call_stats.total_execution_time_ms += execution_time;
                call_stats.last_call_time = Some(SystemTime::now());
            }

            return Ok(JsFFICallResult {
                success: false,
                result: None,
                error: Some(format!("Argument count mismatch: expected {}, got {}", config.param_types.len(), args.len())),
                execution_time_ms: execution_time,
                state_preserved: config.preserve_state,
            });
        }

        // Simulate function call (in a full implementation, this would call the actual JavaScript function)
        let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Update call statistics
        if let Some(call_stats) = self.call_statistics.get_mut(&function_name) {
            call_stats.total_calls += 1;
            call_stats.successful_calls += 1;
            call_stats.total_execution_time_ms += execution_time;
            call_stats.last_call_time = Some(SystemTime::now());
        }

        let result = if config.is_async {
            format!("Async call to {} completed with {} args", function_name, args.len())
        } else {
            format!("Sync call to {} completed with {} args", function_name, args.len())
        };

        Ok(JsFFICallResult {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms: execution_time,
            state_preserved: config.preserve_state,
        })
    }

    /// Register WASM export for host access
    #[napi]
    pub fn register_wasm_export(
        &mut self,
        instance_id: String,
        export_name: String,
        export_type: String,
    ) -> NapiResult<bool> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        // Check if export exists in the instance
        if wasm_instance.exports.contains_key(&export_name) {
            // In a full implementation, this would register the export for host access
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Call WASM export from host
    #[napi]
    pub fn call_wasm_export(
        &self,
        instance_id: String,
        export_name: String,
        args: Vec<String>,
    ) -> NapiResult<JsFFICallResult> {
        let start_time = Instant::now();

        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        // Check if export exists
        if !wasm_instance.exports.contains_key(&export_name) {
            let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
            return Ok(JsFFICallResult {
                success: false,
                result: None,
                error: Some(format!("Export '{}' not found", export_name)),
                execution_time_ms: execution_time,
                state_preserved: false,
            });
        }

        // Simulate WASM export call
        let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

        // Update instance statistics
        {
            let mut stats = wasm_instance.execution_stats.write();
            stats.execution_count += 1;
            stats.total_execution_time_ms += execution_time;
            stats.last_execution_time = Some(SystemTime::now());
        }

        Ok(JsFFICallResult {
            success: true,
            result: Some(format!("Called WASM export {} with {} args", export_name, args.len())),
            error: None,
            execution_time_ms: execution_time,
            state_preserved: false,
        })
    }

    /// Preserve FFI state during hot-reload
    #[napi]
    pub fn preserve_ffi_state(&self, instance_id: String) -> NapiResult<JsFFIStatePreservation> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

        let ffi_bindings = wasm_instance.ffi_bindings.read();
        let preserved_bindings: Vec<String> = ffi_bindings.keys().cloned().collect();
        let preserved_state_size = preserved_bindings.len() as u32 * 64; // Estimate

        Ok(JsFFIStatePreservation {
            success: true,
            preserved_bindings,
            preserved_state_size,
            error: None,
        })
    }

    /// Restore FFI state after hot-reload
    #[napi]
    pub fn restore_ffi_state(
        &mut self,
        instance_id: String,
        preserved_bindings: Vec<String>,
    ) -> NapiResult<bool> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        // Restore FFI bindings (simplified)
        let mut ffi_bindings = wasm_instance.ffi_bindings.write();
        for binding_name in preserved_bindings {
            if let Some(host_config) = self.host_functions.get(&binding_name) {
                let ffi_binding = FFIBinding {
                    name: binding_name.clone(),
                    binding_type: host_config.binding_type.clone(),
                    is_async: host_config.is_async,
                    param_types: host_config.param_types.clone(),
                    result_types: host_config.result_types.clone(),
                    preserve_state: host_config.preserve_state,
                    host_function: None,
                };
                ffi_bindings.insert(binding_name, ffi_binding);
            }
        }

        Ok(true)
    }

    /// Get FFI statistics
    #[napi]
    pub fn get_ffi_stats(&self) -> NapiResult<JsFFIStats> {
        let instances = INSTANCE_REGISTRY.read();
        
        let mut total_host_bindings = self.host_functions.len() as u32;
        let mut total_wasm_exports = 0u32;
        let mut total_calls = 0u64;
        let mut successful_calls = 0u64;
        let mut total_execution_time = 0f64;

        // Aggregate statistics from all instances
        for instance in instances.values() {
            let ffi_bindings = instance.ffi_bindings.read();
            let stats = instance.execution_stats.read();
            
            total_wasm_exports += instance.exports.len() as u32;
            total_calls += stats.ffi_call_count;
            // Assume successful calls = total calls - errors for simplicity
            successful_calls += stats.ffi_call_count.saturating_sub(stats.error_count);
            total_execution_time += stats.total_execution_time_ms;
        }

        // Add statistics from host function calls
        for call_stats in self.call_statistics.values() {
            total_calls += call_stats.total_calls;
            successful_calls += call_stats.successful_calls;
            total_execution_time += call_stats.total_execution_time_ms;
        }

        let failed_calls = total_calls - successful_calls;
        let average_call_time_ms = if total_calls > 0 {
            total_execution_time / total_calls as f64
        } else {
            0.0
        };

        Ok(JsFFIStats {
            total_host_bindings,
            total_wasm_exports,
            total_calls: total_calls as f64,
            successful_calls: successful_calls as f64,
            failed_calls: failed_calls as f64,
            average_call_time_ms,
        })
    }

    /// List registered host functions
    #[napi]
    pub fn list_host_functions(&self) -> Vec<String> {
        self.host_functions.keys().cloned().collect()
    }

    /// Get host function configuration
    #[napi]
    pub fn get_host_function_config(&self, function_name: String) -> NapiResult<JsHostFunctionConfig> {
        self.host_functions.get(&function_name)
            .cloned()
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Host function not found"))
    }

    /// Remove host function
    #[napi]
    pub fn remove_host_function(&mut self, function_name: String) -> bool {
        let removed_config = self.host_functions.remove(&function_name);
        let removed_stats = self.call_statistics.remove(&function_name);
        removed_config.is_some() && removed_stats.is_some()
    }

    /// Get call statistics for specific function
    #[napi]
    pub fn get_function_call_stats(&self, function_name: String) -> NapiResult<JsFFICallResult> {
        let call_stats = self.call_statistics.get(&function_name)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Function not found"))?;

        let average_time = if call_stats.total_calls > 0 {
            call_stats.total_execution_time_ms / call_stats.total_calls as f64
        } else {
            0.0
        };

        Ok(JsFFICallResult {
            success: true,
            result: Some(format!(
                "Total: {}, Success: {}, Failed: {}, Avg Time: {:.2}ms",
                call_stats.total_calls,
                call_stats.successful_calls,
                call_stats.failed_calls,
                average_time
            )),
            error: None,
            execution_time_ms: average_time,
            state_preserved: false,
        })
    }

    /// Clear all call statistics
    #[napi]
    pub fn clear_call_statistics(&mut self) {
        for call_stats in self.call_statistics.values_mut() {
            *call_stats = FFICallStats::default();
        }
    }

    /// Check if host function is registered
    #[napi]
    pub fn has_host_function(&self, function_name: String) -> bool {
        self.host_functions.contains_key(&function_name)
    }

    /// Get total number of registered functions
    #[napi]
    pub fn get_function_count(&self) -> u32 {
        self.host_functions.len() as u32
    }
}

impl Default for FFIBridge {
    fn default() -> Self {
        Self::new()
    }
}