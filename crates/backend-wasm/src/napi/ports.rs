//! Port definitions for hexagonal architecture compliance

use crate::{WasmModule, OptimizationLevel};
use mir_types::ContentHash;
use std::collections::HashMap;

/// Core domain port for WASM instance management
pub trait WasmInstancePort {
    type Error;
    type InstanceId: Clone + std::fmt::Display;
    type ExecutionResult;
    type HotReloadResult;
    type Stats;

    /// Create a new WASM instance
    fn create_instance(
        &mut self,
        id: Self::InstanceId,
        bytecode: &[u8],
        config: &InstanceConfig,
    ) -> Result<Self::ExecutionResult, Self::Error>;

    /// Execute function in instance
    fn execute_function(
        &self,
        id: &Self::InstanceId,
        function_name: &str,
        args: &[String],
    ) -> Result<Self::ExecutionResult, Self::Error>;

    /// Hot-reload instance with new bytecode
    fn hot_reload(
        &mut self,
        id: &Self::InstanceId,
        new_bytecode: &[u8],
        preserve_state: bool,
    ) -> Result<Self::HotReloadResult, Self::Error>;

    /// Get instance statistics
    fn get_stats(&self, id: &Self::InstanceId) -> Result<Self::Stats, Self::Error>;

    /// Remove instance and cleanup resources
    fn remove_instance(&mut self, id: &Self::InstanceId) -> Result<bool, Self::Error>;

    /// List all active instances
    fn list_instances(&self) -> Vec<Self::InstanceId>;
}

/// Configuration for WASM instance creation
#[derive(Debug, Clone)]
pub struct InstanceConfig {
    pub max_memory_mb: u32,
    pub execution_timeout_ms: u32,
    pub enable_debugging: bool,
    pub optimization_level: OptimizationLevel,
}

/// Memory management port
pub trait MemoryPort {
    type Error;
    type InstanceId;
    type AllocationId;

    fn allocate(&mut self, instance_id: &Self::InstanceId, size: u32) -> Result<Self::AllocationId, Self::Error>;
    fn deallocate(&mut self, instance_id: &Self::InstanceId, allocation_id: &Self::AllocationId) -> Result<(), Self::Error>;
    fn read(&self, instance_id: &Self::InstanceId, offset: u32, size: u32) -> Result<Vec<u8>, Self::Error>;
    fn write(&mut self, instance_id: &Self::InstanceId, offset: u32, data: &[u8]) -> Result<(), Self::Error>;
}

/// FFI bridge port for host function interaction
pub trait FFIPort {
    type Error;
    type InstanceId;
    type FunctionId;

    fn register_host_function<F>(&mut self, name: String, function: F) -> Result<Self::FunctionId, Self::Error>
    where
        F: Fn(&[String]) -> Result<String, Self::Error> + Send + Sync + 'static;

    fn call_host_function(
        &self,
        function_id: &Self::FunctionId,
        args: &[String],
    ) -> Result<String, Self::Error>;
}