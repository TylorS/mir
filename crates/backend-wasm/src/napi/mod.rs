//! NAPI bindings for WebAssembly backend
//!
//! This module provides Node.js NAPI bindings for the WASM backend, enabling:
//! - JavaScript interop for WASM modules
//! - WASM memory management integration
//! - FFI bridge preservation during hot-reloading
//! - Host environment state coordination

use crate::{WasmCodeGenerator, WasmModule};

use mir_types::ContentHash;
use napi::{
    bindgen_prelude::*, Error as NapiError, Result as NapiResult, Status,
};
use napi_derive::napi;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;

use wasmtime::{Engine, Func, Instance, Memory, Store, Val, ValType};

use dashmap::DashMap;

/// Global WASM engine instance for efficient module compilation
static WASM_ENGINE: Lazy<Engine> = Lazy::new(|| {
    let mut config = wasmtime::Config::new();
    config.wasm_multi_memory(true);
    config.async_support(true);
    Engine::new(&config).expect("Failed to create WASM engine")
});

/// WASM instance registry for managing active instances (using DashMap for better concurrency)
static INSTANCE_REGISTRY: Lazy<DashMap<String, WasmInstance>> = Lazy::new(|| DashMap::new());

/// WASM instance wrapper with memory management
#[derive(Clone)]
pub struct WasmInstance {
    pub instance: Arc<Mutex<Instance>>,
    pub store: Arc<Mutex<Store<()>>>,
    pub memory: Arc<Mutex<Option<Memory>>>,
    pub module_hash: ContentHash,
    pub exports: HashMap<String, String>,
    pub ffi_bindings: Arc<RwLock<HashMap<String, FFIBinding>>>,
    pub memory_allocations: Arc<RwLock<HashMap<String, MemoryAllocation>>>,
    pub execution_stats: Arc<RwLock<ExecutionStats>>,
}

/// FFI binding information
#[derive(Debug, Clone)]
pub struct FFIBinding {
    pub name: String,
    pub binding_type: String,
    pub is_async: bool,
    pub param_types: Vec<String>,
    pub result_types: Vec<String>,
    pub preserve_state: bool,
    pub host_function: Option<String>, // Serialized function reference
}

/// Memory allocation tracking
#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    pub id: String,
    pub offset: u32,
    pub size: u32,
    pub allocation_type: String,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
}

/// Execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub execution_count: u64,
    pub hot_reload_count: u64,
    pub total_execution_time_ms: f64,
    pub memory_usage_bytes: u64,
    pub last_execution_time: Option<SystemTime>,
    pub ffi_call_count: u64,
    pub error_count: u64,
}

/// WASM-specific error types
#[derive(Debug, Error)]
pub enum WasmError {
    #[error("Instance not found: {id}")]
    InstanceNotFound { id: String },
    #[error("Compilation failed: {reason}")]
    CompilationFailed { reason: String },
    #[error("Execution timeout after {timeout_ms}ms")]
    ExecutionTimeout { timeout_ms: u32 },
    #[error("Memory allocation failed: {reason}")]
    MemoryAllocationFailed { reason: String },
    #[error("FFI call failed: {reason}")]
    FFICallFailed { reason: String },
}

/// Port trait for WASM instance operations
pub trait WasmInstancePort {
    fn create_instance(&self, config: InstanceConfig) -> Result<InstanceId, WasmError>;
    fn execute_function(
        &self,
        id: &InstanceId,
        name: &str,
        args: &[WasmValue],
    ) -> Result<WasmValue, WasmError>;
    fn hot_reload(&self, id: &InstanceId, bytecode: &[u8]) -> Result<(), WasmError>;
    fn allocate_memory(&self, id: &InstanceId, size: u32) -> Result<MemoryAllocation, WasmError>;
}

/// Domain types
pub type InstanceId = String;

#[derive(Debug, Clone)]
pub struct InstanceConfig {
    pub max_memory_pages: u32,
    pub enable_debugging: bool,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// Main NAPI bindings structure with dependency injection
#[napi]
pub struct WasmNapiBindings {
    generator: WasmCodeGenerator,
}

/// JavaScript-compatible WASM module representation
#[napi(object)]
pub struct JsWasmModule {
    pub bytecode: Buffer,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub functions: Vec<String>,
    pub has_memory: bool,
}

/// WASM execution result for JavaScript
#[napi(object)]
pub struct JsExecutionResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub memory_usage: u32,
    pub execution_time_ms: f64,
}

/// Hot-reload result information
#[napi(object)]
pub struct JsHotReloadResult {
    pub success: bool,
    pub preserved_state: bool,
    pub ffi_bindings_preserved: bool,
    pub error: Option<String>,
    pub affected_exports: Vec<String>,
}

/// Memory management statistics
#[napi(object)]
pub struct JsMemoryStats {
    pub total_pages: u32,
    pub used_pages: u32,
    pub max_pages: Option<u32>,
    pub growth_allowed: bool,
    pub bytes_per_page: u32,
    pub active_allocations: u32,
    pub fragmentation_ratio: f64,
    pub total_allocated_bytes: f64, // Use f64 for NAPI compatibility
    pub peak_memory_usage: f64,     // Use f64 for NAPI compatibility
}

/// Memory allocation result
#[napi(object)]
pub struct JsMemoryAllocation {
    pub success: bool,
    pub allocation_id: Option<String>,
    pub offset: Option<u32>,
    pub size: Option<u32>,
    pub error: Option<String>,
}

/// FFI binding configuration
#[napi(object)]
pub struct JsFFIBinding {
    pub name: String,
    pub binding_type: String,
    pub is_async: bool,
    pub param_types: Vec<String>,
    pub result_types: Vec<String>,
    pub preserve_state: bool,
}

/// FFI call result
#[napi(object)]
pub struct JsFFICallResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: f64,
    pub state_preserved: bool,
}

/// Value conversion request
#[napi(object)]
pub struct JsValueConversion {
    pub source_value: String,
    pub source_type: String,
    pub target_type: String,
    pub conversion_options: Option<JsConversionOptions>,
}

/// Conversion options
#[napi(object)]
pub struct JsConversionOptions {
    pub strict_typing: bool,
    pub allow_lossy_conversion: bool,
    pub use_cache: bool,
    pub validate_ranges: bool,
    pub preserve_precision: bool,
}

/// Batch conversion result
#[napi(object)]
pub struct JsBatchConversionResult {
    pub success_rate: f64,
    pub total_time_ms: f64,
    pub converted_values: Vec<String>,
    pub errors: Vec<String>,
}

/// Instance statistics
#[napi(object)]
pub struct JsInstanceStats {
    pub memory_usage_bytes: f64, // Use f64 for NAPI compatibility
    pub execution_count: f64,    // Use f64 for NAPI compatibility
    pub hot_reload_count: f64,   // Use f64 for NAPI compatibility
    pub last_execution_time: Option<String>,
    pub ffi_call_count: f64, // Use f64 for NAPI compatibility
    pub error_count: f64,    // Use f64 for NAPI compatibility
    pub uptime_ms: f64,
}

/// FFI statistics
#[napi(object)]
pub struct JsFFIStats {
    pub total_host_bindings: u32,
    pub total_wasm_exports: u32,
    pub total_calls: f64,      // Use f64 for NAPI compatibility
    pub successful_calls: f64, // Use f64 for NAPI compatibility
    pub failed_calls: f64,     // Use f64 for NAPI compatibility
    pub average_call_time_ms: f64,
}

/// Conversion statistics
#[napi(object)]
pub struct JsConversionStats {
    pub total_conversions: f64,      // Use f64 for NAPI compatibility
    pub cache_hits: f64,             // Use f64 for NAPI compatibility
    pub cache_misses: f64,           // Use f64 for NAPI compatibility
    pub successful_conversions: f64, // Use f64 for NAPI compatibility
    pub failed_conversions: f64,     // Use f64 for NAPI compatibility
    pub average_conversion_time_ms: f64,
}

#[napi]
impl WasmNapiBindings {
    /// Create new NAPI bindings instance
    #[napi(constructor)]
    pub fn new() -> Self {
        WasmNapiBindings {
            generator: WasmCodeGenerator::new(),
        }
    }
}

/// Compile MIR module to WASM with JavaScript-compatible output
#[napi]
pub fn compile_module(&mut self, module_json: String) -> NapiResult<JsWasmModule> {
    // Parse MIR module from JSON
    let module: Module = serde_json::from_str(&module_json)
        .map_err(|e| NapiError::new(Status::InvalidArg, format!("Invalid module JSON: {}", e)))?;

    // Generate WASM module
    let wasm_module = self.generator.generate(&module).map_err(|e| {
        NapiError::new(
            Status::GenericFailure,
            format!("WASM generation failed: {:?}", e),
        )
    })?;

    // Convert to JavaScript-compatible format
    Ok(self.convert_to_js_module(wasm_module)?)
}

/// Load and instantiate WASM module with FFI bindings
#[napi]
pub fn instantiate_module(
    &mut self,
    instance_id: String,
    wasm_bytecode: Buffer,
    imports: Option<JsObject>,
) -> NapiResult<JsExecutionResult> {
    let start_time = Instant::now();

    // Compile WASM module
    let module = WasmtimeModule::from_binary(&WASM_ENGINE, &wasm_bytecode).map_err(|e| {
        NapiError::new(
            Status::GenericFailure,
            format!("WASM compilation failed: {}", e),
        )
    })?;

    // Create store
    let mut store = Store::new(&WASM_ENGINE, ());

    // Process imports if provided
    let import_object = if let Some(_imports_obj) = imports {
        // In a full implementation, this would process the imports object
        // and create the appropriate WASM imports
        Vec::new()
    } else {
        Vec::new()
    };

    // Instantiate module
    let instance = Instance::new(&mut store, &module, &import_object).map_err(|e| {
        NapiError::new(
            Status::GenericFailure,
            format!("WASM instantiation failed: {}", e),
        )
    })?;

    // Extract memory if present
    let memory = instance.get_memory(&mut store, "memory");

    // Extract exports information
    let exports = self.extract_exports(&instance, &mut store)?;

    // Create WASM instance wrapper with enhanced tracking
    let wasm_instance = WasmInstance {
        instance: Arc::new(Mutex::new(instance)),
        store: Arc::new(Mutex::new(store)),
        memory: Arc::new(Mutex::new(memory)),
        module_hash: ContentHash::new(wasm_bytecode.as_ref()),
        exports,
        ffi_bindings: Arc::new(RwLock::new(HashMap::new())),
        memory_allocations: Arc::new(RwLock::new(HashMap::new())),
        execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
    };

    // Register instance
    INSTANCE_REGISTRY.insert(instance_id, wasm_instance.clone());

    let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
    let memory_usage = self.get_memory_usage(&wasm_instance)?;

    Ok(JsExecutionResult {
        success: true,
        result: Some("instantiated".to_string()),
        error: None,
        memory_usage,
        execution_time_ms: execution_time,
    })
}

/// Call WASM function with JavaScript interop
#[napi]
pub async fn call_function(
    &self,
    instance_id: String,
    function_name: String,
    args: Vec<String>,
) -> NapiResult<JsExecutionResult> {
    let start_time = Instant::now();

    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();

    let mut store = wasm_instance.store.lock();
    let instance = wasm_instance.instance.lock();

    // Get function export
    let func = instance
        .get_func(&mut *store, &function_name)
        .ok_or_else(|| {
            NapiError::new(
                Status::InvalidArg,
                format!("Function '{}' not found", function_name),
            )
        })?;

    // Convert JavaScript arguments to WASM values
    let wasm_args = self.convert_js_args_to_wasm(&args, &func, &*store)?;

    // Call function
    let result_types = func.ty(&*store).results().collect::<Vec<_>>();
    let mut results = vec![Val::I32(0); result_types.len()];

    let call_result = func.call(&mut *store, &wasm_args, &mut results);

    // Update execution statistics
    {
        let mut stats = wasm_instance.execution_stats.write();
        stats.execution_count += 1;
        stats.last_execution_time = Some(SystemTime::now());
        let exec_time = start_time.elapsed().as_secs_f64() * 1000.0;
        stats.total_execution_time_ms += exec_time;

        if call_result.is_err() {
            stats.error_count += 1;
        }
    }

    match call_result {
        Ok(()) => {
            let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
            let memory_usage = self.get_memory_usage(&wasm_instance)?;

            // Convert results back to JavaScript-compatible format
            let result_str = self.convert_wasm_results_to_js(&results)?;

            Ok(JsExecutionResult {
                success: true,
                result: Some(result_str),
                error: None,
                memory_usage,
                execution_time_ms: execution_time,
            })
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
            let memory_usage = self.get_memory_usage(&wasm_instance)?;

            Ok(JsExecutionResult {
                success: false,
                result: None,
                error: Some(format!("Function call failed: {}", e)),
                memory_usage,
                execution_time_ms: execution_time,
            })
        }
    }
}

/// Allocate memory in WASM instance
#[napi]
pub fn allocate_memory(
    &self,
    instance_id: String,
    size: u32,
    allocation_type: String,
) -> NapiResult<JsMemoryAllocation> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();

    let mut store = wasm_instance.store.lock();
    let memory_guard = wasm_instance.memory.lock();

    if let Some(memory) = memory_guard.as_ref() {
        let current_size = memory.data_size(&*store);
        let pages_needed = ((size + 65535) / 65536) as u64; // Round up to page size

        // Try to grow memory if needed
        let current_pages = memory.size(&*store);
        if current_size < size as usize {
            match memory.grow(&mut *store, pages_needed) {
                Ok(_) => {
                    // Memory grown successfully
                }
                Err(e) => {
                    return Ok(JsMemoryAllocation {
                        success: false,
                        allocation_id: None,
                        offset: None,
                        size: None,
                        error: Some(format!("Failed to grow memory: {}", e)),
                    });
                }
            }
        }

        // Find a suitable offset (simplified allocation)
        let offset = current_size as u32;
        let allocation_id = Uuid::new_v4().to_string();

        // Track allocation
        let allocation = MemoryAllocation {
            id: allocation_id.clone(),
            offset,
            size,
            allocation_type,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        };

        wasm_instance
            .memory_allocations
            .write()
            .insert(allocation_id.clone(), allocation);

        Ok(JsMemoryAllocation {
            success: true,
            allocation_id: Some(allocation_id),
            offset: Some(offset),
            size: Some(size),
            error: None,
        })
    } else {
        Ok(JsMemoryAllocation {
            success: false,
            allocation_id: None,
            offset: None,
            size: None,
            error: Some("No memory available in instance".to_string()),
        })
    }
}

/// Write data to WASM memory
#[napi]
pub fn write_memory(&self, instance_id: String, offset: u32, data: Buffer) -> NapiResult<bool> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();

    let mut store = wasm_instance.store.lock();
    let memory_guard = wasm_instance.memory.lock();

    if let Some(memory) = memory_guard.as_ref() {
        let memory_data = memory.data_mut(&mut *store);
        let data_slice = data.as_ref();

        if offset as usize + data_slice.len() <= memory_data.len() {
            memory_data[offset as usize..offset as usize + data_slice.len()]
                .copy_from_slice(data_slice);
            Ok(true)
        } else {
            Err(NapiError::new(
                Status::InvalidArg,
                "Memory write out of bounds",
            ))
        }
    } else {
        Err(NapiError::new(
            Status::InvalidArg,
            "No memory available in instance",
        ))
    }
}

/// Read data from WASM memory
#[napi]
pub fn read_memory(&self, instance_id: String, offset: u32, size: u32) -> NapiResult<Buffer> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();

    let store = wasm_instance.store.lock();
    let memory_guard = wasm_instance.memory.lock();

    if let Some(memory) = memory_guard.as_ref() {
        let memory_data = memory.data(&*store);

        if offset as usize + size as usize <= memory_data.len() {
            let data = memory_data[offset as usize..offset as usize + size as usize].to_vec();
            Ok(data.into())
        } else {
            Err(NapiError::new(
                Status::InvalidArg,
                "Memory read out of bounds",
            ))
        }
    } else {
        Err(NapiError::new(
            Status::InvalidArg,
            "No memory available in instance",
        ))
    }
}

/// Free allocated memory
#[napi]
pub fn free_memory(&self, instance_id: String, allocation_id: String) -> NapiResult<bool> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();

    let removed = wasm_instance
        .memory_allocations
        .write()
        .remove(&allocation_id);
    Ok(removed.is_some())
}

/// Hot-reload WASM module while preserving FFI bindings and state
#[napi]
pub fn hot_reload_module(
    &mut self,
    instance_id: String,
    new_wasm_bytecode: Buffer,
    preserve_state: Option<bool>,
) -> NapiResult<JsHotReloadResult> {
    let preserve_state = preserve_state.unwrap_or(true);

    let old_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();

    // Preserve FFI bindings and memory allocations if requested
    let preserved_ffi_bindings = if preserve_state {
        old_instance.ffi_bindings.read().clone()
    } else {
        HashMap::new()
    };

    let preserved_allocations = if preserve_state {
        old_instance.memory_allocations.read().clone()
    } else {
        HashMap::new()
    };

    let mut preserved_stats = old_instance.execution_stats.read().clone();

    // Create new instance
    let new_module =
        WasmtimeModule::from_binary(&WASM_ENGINE, &new_wasm_bytecode).map_err(|e| {
            NapiError::new(
                Status::GenericFailure,
                format!("New WASM compilation failed: {}", e),
            )
        })?;

    let mut new_store = Store::new(&WASM_ENGINE, ());
    let new_instance = Instance::new(&mut new_store, &new_module, &[]).map_err(|e| {
        NapiError::new(
            Status::GenericFailure,
            format!("New WASM instantiation failed: {}", e),
        )
    })?;

    // Extract new exports
    let new_exports = self.extract_exports(&new_instance, &mut new_store)?;
    let affected_exports: Vec<String> = new_exports.keys().cloned().collect();

    // Get memory before moving store
    let memory = new_instance.get_memory(&mut new_store, "memory");

    // Update hot-reload statistics
    preserved_stats.hot_reload_count += 1;

    // Create new WASM instance wrapper with preserved state
    let new_wasm_instance = WasmInstance {
        instance: Arc::new(Mutex::new(new_instance)),
        store: Arc::new(Mutex::new(new_store)),
        memory: Arc::new(Mutex::new(memory)),
        module_hash: ContentHash::new(new_wasm_bytecode.as_ref()),
        exports: new_exports,
        ffi_bindings: Arc::new(RwLock::new(preserved_ffi_bindings)),
        memory_allocations: Arc::new(RwLock::new(preserved_allocations)),
        execution_stats: Arc::new(RwLock::new(preserved_stats)),
    };

    // Replace old instance
    INSTANCE_REGISTRY.insert(instance_id, new_wasm_instance);

    Ok(JsHotReloadResult {
        success: true,
        preserved_state: preserve_state,
        ffi_bindings_preserved: preserve_state,
        error: None,
        affected_exports,
    })
}

/// Get memory statistics for WASM instance
#[napi]
pub fn get_memory_stats(&self, instance_id: String) -> NapiResult<JsMemoryStats> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

    let store = wasm_instance.store.lock();
    let memory_guard = wasm_instance.memory.lock();
    let allocations = wasm_instance.memory_allocations.read();
    let stats = wasm_instance.execution_stats.read();

    if let Some(memory) = memory_guard.as_ref() {
        let size = memory.size(&*store);
        let data_size = memory.data_size(&*store);
        let max_size = memory.ty(&*store).maximum();

        let total_allocated: u64 = allocations.values().map(|a| a.size as u64).sum();
        let active_allocations = allocations.len() as u32;

        // Calculate fragmentation ratio (simplified)
        let fragmentation_ratio = if data_size > 0 {
            1.0 - (total_allocated as f64 / data_size as f64)
        } else {
            0.0
        };

        Ok(JsMemoryStats {
            total_pages: size as u32,
            used_pages: (data_size / 65536) as u32, // 64KB per page
            max_pages: max_size.map(|m| m as u32),
            growth_allowed: max_size.is_none() || max_size.unwrap() > size,
            bytes_per_page: 65536,
            active_allocations,
            fragmentation_ratio,
            total_allocated_bytes: total_allocated as f64,
            peak_memory_usage: stats.memory_usage_bytes as f64,
        })
    } else {
        Ok(JsMemoryStats {
            total_pages: 0,
            used_pages: 0,
            max_pages: None,
            growth_allowed: false,
            bytes_per_page: 65536,
            active_allocations: 0,
            fragmentation_ratio: 0.0,
            total_allocated_bytes: 0.0,
            peak_memory_usage: 0.0,
        })
    }
}

/// Get instance statistics
#[napi]
pub fn get_instance_stats(&self, instance_id: String) -> NapiResult<JsInstanceStats> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

    let stats = wasm_instance.execution_stats.read();
    let memory_usage = self.get_memory_usage(wasm_instance)?;

    let last_execution_time = stats
        .last_execution_time
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string());

    // Calculate uptime (simplified - would track creation time in real implementation)
    let uptime_ms = stats.total_execution_time_ms;

    Ok(JsInstanceStats {
        memory_usage_bytes: memory_usage as f64,
        execution_count: stats.execution_count as f64,
        hot_reload_count: stats.hot_reload_count as f64,
        last_execution_time,
        ffi_call_count: stats.ffi_call_count as f64,
        error_count: stats.error_count as f64,
        uptime_ms,
    })
}

/// Get FFI statistics
#[napi]
pub fn get_ffi_stats(&self) -> NapiResult<JsFFIStats> {
    let mut total_host_bindings = 0u32;
    let mut total_wasm_exports = 0u32;
    let mut total_calls = 0u64;
    let mut successful_calls = 0u64;
    let mut total_execution_time = 0f64;

    for instance in INSTANCE_REGISTRY.iter() {
        let instance = instance.value();
        let ffi_bindings = instance.ffi_bindings.read();
        let stats = instance.execution_stats.read();

        total_host_bindings += ffi_bindings.len() as u32;
        total_wasm_exports += instance.exports.len() as u32;
        total_calls += stats.ffi_call_count;
        // Assume successful calls = total calls - errors for simplicity
        successful_calls += stats.ffi_call_count.saturating_sub(stats.error_count);
        total_execution_time += stats.total_execution_time_ms;
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

/// Convert JavaScript values to WASM values
#[napi]
pub fn convert_js_to_wasm(&self, conversion: JsValueConversion) -> NapiResult<String> {
    // Simplified conversion - in a full implementation, this would handle
    // comprehensive type conversion between JavaScript and WASM
    let result = match conversion.target_type.as_str() {
        "i32" => {
            let value: i32 = conversion
                .source_value
                .parse()
                .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to i32"))?;
            value.to_string()
        }
        "i64" => {
            let value: i64 = conversion
                .source_value
                .parse()
                .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to i64"))?;
            value.to_string()
        }
        "f32" => {
            let value: f32 = conversion
                .source_value
                .parse()
                .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to f32"))?;
            value.to_string()
        }
        "f64" => {
            let value: f64 = conversion
                .source_value
                .parse()
                .map_err(|_| NapiError::new(Status::InvalidArg, "Cannot convert to f64"))?;
            value.to_string()
        }
        "string" => conversion.source_value,
        _ => {
            return Err(NapiError::new(
                Status::InvalidArg,
                "Unsupported target type",
            ))
        }
    };

    Ok(result)
}

/// Batch convert values
#[napi]
pub fn batch_convert_values(
    &self,
    conversions: Vec<JsValueConversion>,
) -> NapiResult<JsBatchConversionResult> {
    let start_time = Instant::now();
    let mut converted_values = Vec::new();
    let mut errors = Vec::new();
    let mut successful_conversions = 0;

    for conversion in conversions {
        match self.convert_js_to_wasm(conversion) {
            Ok(result) => {
                converted_values.push(result);
                successful_conversions += 1;
            }
            Err(e) => {
                converted_values.push(String::new());
                errors.push(e.to_string());
            }
        }
    }

    let total_conversions = converted_values.len();
    let success_rate = if total_conversions > 0 {
        successful_conversions as f64 / total_conversions as f64
    } else {
        0.0
    };

    let total_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    Ok(JsBatchConversionResult {
        success_rate,
        total_time_ms,
        converted_values,
        errors,
    })
}

/// Compact memory for instance
#[napi]
pub fn compact_memory(&self, instance_id: String) -> NapiResult<bool> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

    // In a full implementation, this would perform memory compaction
    // For now, just clean up expired allocations
    let mut allocations = wasm_instance.memory_allocations.write();
    let now = SystemTime::now();
    let initial_count = allocations.len();

    allocations.retain(|_, allocation| {
        // Keep allocations that were accessed recently (within 1 hour)
        now.duration_since(allocation.last_accessed)
            .map(|d| d.as_secs() < 3600)
            .unwrap_or(true)
    });

    let cleaned_count = initial_count - allocations.len();
    Ok(cleaned_count > 0)
}

/// Garbage collect unused allocations
#[napi]
pub fn garbage_collect(&self, instance_id: String) -> NapiResult<u32> {
    let wasm_instance = INSTANCE_REGISTRY
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

    let mut allocations = wasm_instance.memory_allocations.write();
    let initial_count = allocations.len();

    // Simple garbage collection - remove old allocations
    let now = SystemTime::now();
    allocations.retain(|_, allocation| {
        now.duration_since(allocation.last_accessed)
            .map(|d| d.as_secs() < 3600) // Keep allocations accessed within 1 hour
            .unwrap_or(true)
    });

    Ok((initial_count - allocations.len()) as u32)
}

/// Register FFI binding for host environment interaction
#[napi]
pub fn register_ffi_binding(
    &mut self,
    instance_id: String,
    binding_config: JsFFIBinding,
) -> NapiResult<bool> {
    let instances = INSTANCE_REGISTRY.read();
    let wasm_instance = instances
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();
    drop(instances);

    let ffi_binding = FFIBinding {
        name: binding_config.name.clone(),
        binding_type: binding_config.binding_type,
        is_async: binding_config.is_async,
        param_types: binding_config.param_types,
        result_types: binding_config.result_types,
        preserve_state: binding_config.preserve_state,
        host_function: None, // Would store serialized function reference
    };

    wasm_instance
        .ffi_bindings
        .write()
        .insert(binding_config.name, ffi_binding);
    Ok(true)
}

/// Call FFI function
#[napi]
pub async fn call_ffi_function(
    &self,
    instance_id: String,
    function_name: String,
    args: Vec<String>,
) -> NapiResult<JsFFICallResult> {
    let start_time = Instant::now();

    let instances = INSTANCE_REGISTRY.read();
    let wasm_instance = instances
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
        .clone();
    drop(instances);

    let ffi_bindings = wasm_instance.ffi_bindings.read();
    let binding = ffi_bindings
        .get(&function_name)
        .ok_or_else(|| {
            NapiError::new(
                Status::InvalidArg,
                format!("FFI binding '{}' not found", function_name),
            )
        })?
        .clone();
    drop(ffi_bindings);

    // Update FFI call statistics
    {
        let mut stats = wasm_instance.execution_stats.write();
        stats.ffi_call_count += 1;
    }

    // Simulate FFI call (in a full implementation, this would call the actual host function)
    let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

    Ok(JsFFICallResult {
        success: true,
        result: Some(format!(
            "FFI call to {} with {} args",
            function_name,
            args.len()
        )),
        error: None,
        execution_time_ms: execution_time,
        state_preserved: binding.preserve_state,
    })
}

/// Get FFI bindings for instance
#[napi]
pub fn get_ffi_bindings(&self, instance_id: String) -> NapiResult<Vec<JsFFIBinding>> {
    let instances = INSTANCE_REGISTRY.read();
    let wasm_instance = instances
        .get(&instance_id)
        .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

    let bindings = wasm_instance.ffi_bindings.read();
    let js_bindings = bindings
        .values()
        .map(|binding| JsFFIBinding {
            name: binding.name.clone(),
            binding_type: binding.binding_type.clone(),
            is_async: binding.is_async,
            param_types: binding.param_types.clone(),
            result_types: binding.result_types.clone(),
            preserve_state: binding.preserve_state,
        })
        .collect();

    Ok(js_bindings)
}

/// Dispose WASM instance and clean up resources
#[napi]
pub fn dispose_instance(&mut self, instance_id: String) -> NapiResult<bool> {
    let mut instances = INSTANCE_REGISTRY.write();
    Ok(instances.remove(&instance_id).is_some())
}

/// Get list of active WASM instances
#[napi]
pub fn list_instances(&self) -> Vec<String> {
    INSTANCE_REGISTRY.read().keys().cloned().collect()
}

/// Set optimization level for WASM generation
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

// Implementation helper methods

/// Convert WASM module to JavaScript-compatible format
fn convert_to_js_module(&self, wasm_module: WasmModule) -> NapiResult<JsWasmModule> {
    let exports = wasm_module
        .exports
        .into_iter()
        .map(|exp| exp.name)
        .collect();
    let imports = wasm_module
        .imports
        .into_iter()
        .map(|imp| format!("{}::{}", imp.module, imp.name))
        .collect();
    let functions = wasm_module
        .functions
        .into_iter()
        .filter_map(|func| func.name)
        .collect();
    let has_memory = wasm_module.memory.is_some();

    Ok(JsWasmModule {
        bytecode: wasm_module.bytecode.into(),
        exports,
        imports,
        functions,
        has_memory,
    })
}

/// Extract export information from WASM instance
fn extract_exports(
    &self,
    instance: &Instance,
    store: &mut Store<()>,
) -> NapiResult<HashMap<String, String>> {
    let mut exports = HashMap::new();

    for export in instance.exports(store) {
        let name = export.name().to_string();
        let export_type = match export.into_extern() {
            wasmtime::Extern::Func(_) => "function",
            wasmtime::Extern::Memory(_) => "memory",
            wasmtime::Extern::Global(_) => "global",
            wasmtime::Extern::Table(_) => "table",
            wasmtime::Extern::SharedMemory(_) => "shared_memory",
        };

        exports.insert(name, export_type.to_string());
    }

    Ok(exports)
}

/// Get memory usage for WASM instance
fn get_memory_usage(&self, instance: &WasmInstance) -> NapiResult<u32> {
    let store = instance.store.lock();
    let memory_guard = instance.memory.lock();

    if let Some(memory) = memory_guard.as_ref() {
        Ok(memory.data_size(&*store) as u32)
    } else {
        Ok(0)
    }
}

/// Convert JavaScript arguments to WASM values
fn convert_js_args_to_wasm(
    &self,
    args: &[String],
    func: &Func,
    store: &Store<()>,
) -> NapiResult<Vec<Val>> {
    let param_types = func.ty(store).params().collect::<Vec<_>>();
    let mut wasm_args = Vec::new();

    for (i, arg) in args.iter().enumerate() {
        if i >= param_types.len() {
            break; // Ignore extra arguments
        }

        let wasm_val = match param_types[i] {
            ValType::I32 => {
                let value: i32 = arg.parse().map_err(|_| {
                    NapiError::new(
                        Status::InvalidArg,
                        format!("Cannot convert '{}' to i32", arg),
                    )
                })?;
                Val::I32(value)
            }
            ValType::I64 => {
                let value: i64 = arg.parse().map_err(|_| {
                    NapiError::new(
                        Status::InvalidArg,
                        format!("Cannot convert '{}' to i64", arg),
                    )
                })?;
                Val::I64(value)
            }
            ValType::F32 => {
                let value: f32 = arg.parse().map_err(|_| {
                    NapiError::new(
                        Status::InvalidArg,
                        format!("Cannot convert '{}' to f32", arg),
                    )
                })?;
                Val::F32(value.to_bits())
            }
            ValType::F64 => {
                let value: f64 = arg.parse().map_err(|_| {
                    NapiError::new(
                        Status::InvalidArg,
                        format!("Cannot convert '{}' to f64", arg),
                    )
                })?;
                Val::F64(value.to_bits())
            }
            _ => {
                return Err(NapiError::new(
                    Status::InvalidArg,
                    format!("Unsupported parameter type: {:?}", param_types[i]),
                ));
            }
        };

        wasm_args.push(wasm_val);
    }

    // Pad with default values if not enough arguments
    while wasm_args.len() < param_types.len() {
        let default_val = match param_types[wasm_args.len()] {
            ValType::I32 => Val::I32(0),
            ValType::I64 => Val::I64(0),
            ValType::F32 => Val::F32(0),
            ValType::F64 => Val::F64(0),
            _ => {
                return Err(NapiError::new(
                    Status::InvalidArg,
                    "Cannot create default value for unsupported type",
                ))
            }
        };
        wasm_args.push(default_val);
    }

    Ok(wasm_args)
}

/// Convert WASM results to JavaScript-compatible format
fn convert_wasm_results_to_js(&self, results: &[Val]) -> NapiResult<String> {
    if results.is_empty() {
        return Ok("undefined".to_string());
    }

    if results.len() == 1 {
        let result_str = match &results[0] {
            Val::I32(v) => v.to_string(),
            Val::I64(v) => v.to_string(),
            Val::F32(v) => f32::from_bits(*v).to_string(),
            Val::F64(v) => f64::from_bits(*v).to_string(),
            _ => "unsupported".to_string(),
        };
        return Ok(result_str);
    }

    // Multiple results - return as JSON array
    let result_strings: Vec<String> = results
        .iter()
        .map(|val| match val {
            Val::I32(v) => v.to_string(),
            Val::I64(v) => v.to_string(),
            Val::F32(v) => f32::from_bits(*v).to_string(),
            Val::F64(v) => f64::from_bits(*v).to_string(),
            _ => "null".to_string(),
        })
        .collect();

    Ok(format!("[{}]", result_strings.join(",")))
}

impl Default for WasmNapiBindings {
    fn default() -> Self {
        Self::new()
    }
}

// Sub-modules for comprehensive NAPI functionality
pub mod ffi_bridge;
pub mod instance_manager;
pub mod memory_manager;
pub mod value_converter;

// Re-export for convenience
pub use ffi_bridge::{FFIBridge, JsFFIStatePreservation, JsHostFunctionConfig};
pub use instance_manager::{
    JsInstanceCreationResult, JsInstanceManagerConfig, WasmInstanceManager,
};
pub use memory_manager::{
    JsMemoryCompactionResult, JsMemoryView, JsMemoryViewConfig, WasmMemoryManager,
};
pub use value_converter::{JsTypeMapping, ValueConverter};
