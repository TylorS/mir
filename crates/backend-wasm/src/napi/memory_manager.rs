//! WASM Memory Manager for advanced memory operations

use super::{JsMemoryStats, JsMemoryAllocation, INSTANCE_REGISTRY};
use napi::{bindgen_prelude::*, Result as NapiResult, Error as NapiError, Status};
use napi_derive::napi;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Memory view configuration
#[napi(object)]
pub struct JsMemoryViewConfig {
    pub offset: u32,
    pub size: u32,
    pub view_type: String, // "uint8", "uint16", "uint32", "int8", "int16", "int32", "float32", "float64"
}

/// Memory view result
#[napi(object)]
pub struct JsMemoryView {
    pub success: bool,
    pub view_id: Option<String>,
    pub offset: u32,
    pub size: u32,
    pub view_type: String,
    pub error: Option<String>,
}

/// Memory compaction result
#[napi(object)]
pub struct JsMemoryCompactionResult {
    pub success: bool,
    pub bytes_freed: u32,
    pub fragmentation_before: f64,
    pub fragmentation_after: f64,
    pub duration_ms: f64,
}

/// WASM Memory Manager for Node.js
#[napi]
pub struct WasmMemoryManager {
    memory_views: HashMap<String, JsMemoryViewConfig>,
}

#[napi]
impl WasmMemoryManager {
    /// Create new memory manager
    #[napi(constructor)]
    pub fn new() -> Self {
        WasmMemoryManager {
            memory_views: HashMap::new(),
        }
    }

    /// Allocate memory in WASM instance
    #[napi]
    pub fn allocate_memory(
        &mut self,
        instance_id: String,
        size: u32,
        allocation_type: String,
    ) -> NapiResult<JsMemoryAllocation> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        let mut store = wasm_instance.store.lock();
        let memory_guard = wasm_instance.memory.lock();

        if let Some(memory) = memory_guard.as_ref() {
            let current_size = memory.data_size(&*store);
            
            // Simple allocation strategy: find next available offset
            let allocations = wasm_instance.memory_allocations.read();
            let mut next_offset = 0u32;
            
            // Find the next available offset after existing allocations
            for allocation in allocations.values() {
                let end_offset = allocation.offset + allocation.size;
                if end_offset > next_offset {
                    next_offset = end_offset;
                }
            }
            drop(allocations);

            // Ensure we have enough memory
            if next_offset + size > current_size as u32 {
                // Try to grow memory
                let pages_needed = ((next_offset + size - current_size as u32 + 65535) / 65536) as u64;
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

            let allocation_id = Uuid::new_v4().to_string();

            // Create allocation record
            let allocation = super::MemoryAllocation {
                id: allocation_id.clone(),
                offset: next_offset,
                size,
                allocation_type,
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
            };

            wasm_instance.memory_allocations.write().insert(allocation_id.clone(), allocation);

            Ok(JsMemoryAllocation {
                success: true,
                allocation_id: Some(allocation_id),
                offset: Some(next_offset),
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
    pub fn write_memory(
        &self,
        instance_id: String,
        offset: u32,
        data: Buffer,
    ) -> NapiResult<JsMemoryAllocation> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        let mut store = wasm_instance.store.lock();
        let memory_guard = wasm_instance.memory.lock();

        if let Some(memory) = memory_guard.as_ref() {
            let memory_data = memory.data_mut(&mut *store);
            let data_slice = data.as_ref();
            
            if offset as usize + data_slice.len() <= memory_data.len() {
                memory_data[offset as usize..offset as usize + data_slice.len()]
                    .copy_from_slice(data_slice);

                // Update last accessed time for any allocations that overlap
                let mut allocations = wasm_instance.memory_allocations.write();
                for allocation in allocations.values_mut() {
                    let alloc_end = allocation.offset + allocation.size;
                    let write_end = offset + data_slice.len() as u32;
                    
                    // Check for overlap
                    if !(offset >= alloc_end || write_end <= allocation.offset) {
                        allocation.last_accessed = SystemTime::now();
                    }
                }

                Ok(JsMemoryAllocation {
                    success: true,
                    allocation_id: None,
                    offset: Some(offset),
                    size: Some(data_slice.len() as u32),
                    error: None,
                })
            } else {
                Ok(JsMemoryAllocation {
                    success: false,
                    allocation_id: None,
                    offset: None,
                    size: None,
                    error: Some("Memory write out of bounds".to_string()),
                })
            }
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

    /// Read data from WASM memory
    #[napi]
    pub fn read_memory(
        &self,
        instance_id: String,
        offset: u32,
        size: u32,
    ) -> NapiResult<Buffer> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        let store = wasm_instance.store.lock();
        let memory_guard = wasm_instance.memory.lock();

        if let Some(memory) = memory_guard.as_ref() {
            let memory_data = memory.data(&*store);
            
            if offset as usize + size as usize <= memory_data.len() {
                let data = memory_data[offset as usize..offset as usize + size as usize].to_vec();

                // Update last accessed time for any allocations that overlap
                let mut allocations = wasm_instance.memory_allocations.write();
                for allocation in allocations.values_mut() {
                    let alloc_end = allocation.offset + allocation.size;
                    let read_end = offset + size;
                    
                    // Check for overlap
                    if !(offset >= alloc_end || read_end <= allocation.offset) {
                        allocation.last_accessed = SystemTime::now();
                    }
                }

                Ok(data.into())
            } else {
                Err(NapiError::new(Status::InvalidArg, "Memory read out of bounds"))
            }
        } else {
            Err(NapiError::new(Status::InvalidArg, "No memory available in instance"))
        }
    }

    /// Create memory view for direct access
    #[napi]
    pub fn create_memory_view(
        &mut self,
        instance_id: String,
        offset: u32,
        size: u32,
        view_type: String,
    ) -> NapiResult<JsMemoryView> {
        // Validate view type
        match view_type.as_str() {
            "uint8" | "uint16" | "uint32" | "int8" | "int16" | "int32" | "float32" | "float64" => {}
            _ => {
                return Ok(JsMemoryView {
                    success: false,
                    view_id: None,
                    offset: 0,
                    size: 0,
                    view_type: String::new(),
                    error: Some("Invalid view type".to_string()),
                });
            }
        }

        // Check if instance exists
        let instances = INSTANCE_REGISTRY.read();
        if !instances.contains_key(&instance_id) {
            return Ok(JsMemoryView {
                success: false,
                view_id: None,
                offset: 0,
                size: 0,
                view_type: String::new(),
                error: Some("Instance not found".to_string()),
            });
        }
        drop(instances);

        let view_id = Uuid::new_v4().to_string();
        let config = JsMemoryViewConfig {
            offset,
            size,
            view_type: view_type.clone(),
        };

        self.memory_views.insert(view_id.clone(), config);

        Ok(JsMemoryView {
            success: true,
            view_id: Some(view_id),
            offset,
            size,
            view_type,
            error: None,
        })
    }

    /// Get memory statistics
    #[napi]
    pub fn get_memory_stats(&self, instance_id: String) -> NapiResult<JsMemoryStats> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
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
            
            // Calculate fragmentation ratio
            let fragmentation_ratio = if data_size > 0 {
                let used_space = total_allocated as f64;
                let total_space = data_size as f64;
                1.0 - (used_space / total_space).min(1.0)
            } else {
                0.0
            };

            Ok(JsMemoryStats {
                total_pages: size as u32,
                used_pages: (data_size / 65536) as u32,
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

    /// Compact memory to reduce fragmentation
    #[napi]
    pub fn compact_memory(&self, instance_id: String) -> NapiResult<JsMemoryCompactionResult> {
        let start_time = std::time::Instant::now();

        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?
            .clone();
        drop(instances);

        // Get fragmentation before compaction
        let fragmentation_before = {
            let stats = self.get_memory_stats(instance_id.clone())?;
            stats.fragmentation_ratio
        };

        // Perform compaction by removing old allocations
        let mut allocations = wasm_instance.memory_allocations.write();
        let initial_count = allocations.len();
        let now = SystemTime::now();

        // Remove allocations that haven't been accessed in the last hour
        allocations.retain(|_, allocation| {
            now.duration_since(allocation.last_accessed)
                .map(|d| d.as_secs() < 3600)
                .unwrap_or(true)
        });

        let removed_count = initial_count - allocations.len();
        let bytes_freed = removed_count as u32 * 1024; // Estimate

        drop(allocations);

        // Get fragmentation after compaction
        let fragmentation_after = {
            let stats = self.get_memory_stats(instance_id)?;
            stats.fragmentation_ratio
        };

        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(JsMemoryCompactionResult {
            success: true,
            bytes_freed,
            fragmentation_before,
            fragmentation_after,
            duration_ms,
        })
    }

    /// Garbage collect unused allocations
    #[napi]
    pub fn garbage_collect(&self, instance_id: String) -> NapiResult<u32> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

        let mut allocations = wasm_instance.memory_allocations.write();
        let initial_count = allocations.len();

        // Remove allocations older than 10 minutes
        let now = SystemTime::now();
        allocations.retain(|_, allocation| {
            now.duration_since(allocation.created_at)
                .map(|d| d.as_secs() < 600)
                .unwrap_or(true)
        });

        let collected_count = initial_count - allocations.len();
        Ok(collected_count as u32)
    }

    /// Free specific allocation
    #[napi]
    pub fn free_allocation(
        &self,
        instance_id: String,
        allocation_id: String,
    ) -> NapiResult<bool> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

        let removed = wasm_instance.memory_allocations.write().remove(&allocation_id);
        Ok(removed.is_some())
    }

    /// List all allocations for instance
    #[napi]
    pub fn list_allocations(&self, instance_id: String) -> NapiResult<Vec<String>> {
        let instances = INSTANCE_REGISTRY.read();
        let wasm_instance = instances.get(&instance_id)
            .ok_or_else(|| NapiError::new(Status::InvalidArg, "Instance not found"))?;

        let allocations = wasm_instance.memory_allocations.read();
        let allocation_info: Vec<String> = allocations.values().map(|alloc| {
            format!("{}:{}:{}:{}", alloc.id, alloc.offset, alloc.size, alloc.allocation_type)
        }).collect();

        Ok(allocation_info)
    }

    /// Remove memory view
    #[napi]
    pub fn remove_memory_view(&mut self, view_id: String) -> bool {
        self.memory_views.remove(&view_id).is_some()
    }

    /// List active memory views
    #[napi]
    pub fn list_memory_views(&self) -> Vec<String> {
        self.memory_views.keys().cloned().collect()
    }
}

impl Default for WasmMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}