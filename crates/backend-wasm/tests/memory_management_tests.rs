//! Unit tests for WASM Memory Management
//!
//! These tests verify memory allocation, deallocation, and management functionality.

use mir_backend_wasm::napi::{
    WasmInstance, MemoryAllocation, ExecutionStats, INSTANCE_REGISTRY,
};
use mir_types::ContentHash;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::SystemTime;
use parking_lot::Mutex;

fn create_mock_wasm_instance(id: &str) -> WasmInstance {
    let hash = ContentHash::new(id.as_bytes());
    
    WasmInstance {
        instance: Arc::new(Mutex::new(unsafe { std::mem::zeroed() })), // Mock instance
        store: Arc::new(Mutex::new(unsafe { std::mem::zeroed() })), // Mock store
        memory: Arc::new(Mutex::new(None)), // No memory initially
        module_hash: hash,
        exports: HashMap::new(),
        ffi_bindings: Arc::new(RwLock::new(HashMap::new())),
        memory_allocations: Arc::new(RwLock::new(HashMap::new())),
        execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
    }
}

#[test]
fn test_memory_allocation_tracking() {
    let instance = create_mock_wasm_instance("memory_test");
    
    // Create a memory allocation
    let allocation = MemoryAllocation {
        id: "alloc_1".to_string(),
        offset: 1024,
        size: 512,
        allocation_type: "buffer".to_string(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
    };
    
    // Add allocation to instance
    instance.memory_allocations.write().insert("alloc_1".to_string(), allocation.clone());
    
    // Verify allocation is tracked
    let allocations = instance.memory_allocations.read();
    assert!(allocations.contains_key("alloc_1"));
    
    let stored_allocation = allocations.get("alloc_1").unwrap();
    assert_eq!(stored_allocation.id, "alloc_1");
    assert_eq!(stored_allocation.offset, 1024);
    assert_eq!(stored_allocation.size, 512);
    assert_eq!(stored_allocation.allocation_type, "buffer");
}

#[test]
fn test_multiple_allocations() {
    let instance = create_mock_wasm_instance("multi_alloc_test");
    
    // Create multiple allocations
    let allocations = vec![
        MemoryAllocation {
            id: "alloc_1".to_string(),
            offset: 0,
            size: 256,
            allocation_type: "buffer".to_string(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        },
        MemoryAllocation {
            id: "alloc_2".to_string(),
            offset: 256,
            size: 512,
            allocation_type: "array".to_string(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        },
        MemoryAllocation {
            id: "alloc_3".to_string(),
            offset: 768,
            size: 128,
            allocation_type: "string".to_string(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        },
    ];
    
    // Add all allocations
    {
        let mut alloc_map = instance.memory_allocations.write();
        for allocation in allocations {
            alloc_map.insert(allocation.id.clone(), allocation);
        }
    }
    
    // Verify all allocations are tracked
    let alloc_map = instance.memory_allocations.read();
    assert_eq!(alloc_map.len(), 3);
    assert!(alloc_map.contains_key("alloc_1"));
    assert!(alloc_map.contains_key("alloc_2"));
    assert!(alloc_map.contains_key("alloc_3"));
    
    // Verify total allocated size
    let total_size: u32 = alloc_map.values().map(|a| a.size).sum();
    assert_eq!(total_size, 256 + 512 + 128);
}

#[test]
fn test_allocation_deallocation() {
    let instance = create_mock_wasm_instance("dealloc_test");
    
    // Add allocation
    let allocation = MemoryAllocation {
        id: "temp_alloc".to_string(),
        offset: 1024,
        size: 256,
        allocation_type: "temporary".to_string(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
    };
    
    instance.memory_allocations.write().insert("temp_alloc".to_string(), allocation);
    
    // Verify allocation exists
    assert!(instance.memory_allocations.read().contains_key("temp_alloc"));
    
    // Remove allocation
    let removed = instance.memory_allocations.write().remove("temp_alloc");
    assert!(removed.is_some());
    
    // Verify allocation is gone
    assert!(!instance.memory_allocations.read().contains_key("temp_alloc"));
}

#[test]
fn test_execution_statistics() {
    let instance = create_mock_wasm_instance("stats_test");
    
    // Initial stats should be default
    {
        let stats = instance.execution_stats.read();
        assert_eq!(stats.execution_count, 0);
        assert_eq!(stats.hot_reload_count, 0);
        assert_eq!(stats.total_execution_time_ms, 0.0);
        assert_eq!(stats.memory_usage_bytes, 0);
        assert_eq!(stats.ffi_call_count, 0);
        assert_eq!(stats.error_count, 0);
        assert!(stats.last_execution_time.is_none());
    }
    
    // Update stats
    {
        let mut stats = instance.execution_stats.write();
        stats.execution_count = 5;
        stats.hot_reload_count = 2;
        stats.total_execution_time_ms = 123.45;
        stats.memory_usage_bytes = 2048;
        stats.ffi_call_count = 10;
        stats.error_count = 1;
        stats.last_execution_time = Some(SystemTime::now());
    }
    
    // Verify updated stats
    {
        let stats = instance.execution_stats.read();
        assert_eq!(stats.execution_count, 5);
        assert_eq!(stats.hot_reload_count, 2);
        assert_eq!(stats.total_execution_time_ms, 123.45);
        assert_eq!(stats.memory_usage_bytes, 2048);
        assert_eq!(stats.ffi_call_count, 10);
        assert_eq!(stats.error_count, 1);
        assert!(stats.last_execution_time.is_some());
    }
}

#[test]
fn test_concurrent_allocation_access() {
    use std::thread;
    use std::sync::Arc;
    
    let instance = Arc::new(create_mock_wasm_instance("concurrent_test"));
    
    // Spawn multiple threads that add allocations
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let instance = Arc::clone(&instance);
            thread::spawn(move || {
                let allocation = MemoryAllocation {
                    id: format!("alloc_{}", i),
                    offset: i * 256,
                    size: 256,
                    allocation_type: "concurrent".to_string(),
                    created_at: SystemTime::now(),
                    last_accessed: SystemTime::now(),
                };
                
                instance.memory_allocations.write().insert(
                    format!("alloc_{}", i),
                    allocation,
                );
            })
        })
        .collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all allocations were added
    let allocations = instance.memory_allocations.read();
    assert_eq!(allocations.len(), 10);
    
    for i in 0..10 {
        assert!(allocations.contains_key(&format!("alloc_{}", i)));
    }
}

#[test]
fn test_memory_fragmentation_calculation() {
    let instance = create_mock_wasm_instance("fragmentation_test");
    
    // Add allocations with gaps (simulating fragmentation)
    let allocations = vec![
        MemoryAllocation {
            id: "alloc_1".to_string(),
            offset: 0,
            size: 100,
            allocation_type: "buffer".to_string(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        },
        MemoryAllocation {
            id: "alloc_2".to_string(),
            offset: 200, // Gap of 100 bytes
            size: 150,
            allocation_type: "buffer".to_string(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        },
        MemoryAllocation {
            id: "alloc_3".to_string(),
            offset: 400, // Gap of 50 bytes
            size: 200,
            allocation_type: "buffer".to_string(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
        },
    ];
    
    {
        let mut alloc_map = instance.memory_allocations.write();
        for allocation in allocations {
            alloc_map.insert(allocation.id.clone(), allocation);
        }
    }
    
    // Calculate fragmentation metrics
    let alloc_map = instance.memory_allocations.read();
    let total_allocated: u32 = alloc_map.values().map(|a| a.size).sum();
    let highest_offset = alloc_map.values().map(|a| a.offset + a.size).max().unwrap_or(0);
    
    assert_eq!(total_allocated, 450); // 100 + 150 + 200
    assert_eq!(highest_offset, 600); // 400 + 200
    
    // Fragmentation ratio would be (600 - 450) / 600 = 0.25 (25% fragmented)
    let fragmentation_ratio = (highest_offset - total_allocated) as f64 / highest_offset as f64;
    assert!((fragmentation_ratio - 0.25).abs() < 0.001);
}

#[test]
fn test_allocation_age_tracking() {
    use std::time::Duration;
    
    let instance = create_mock_wasm_instance("age_test");
    let now = SystemTime::now();
    
    // Create allocation with specific timestamps
    let old_allocation = MemoryAllocation {
        id: "old_alloc".to_string(),
        offset: 0,
        size: 256,
        allocation_type: "old".to_string(),
        created_at: now - Duration::from_secs(3600), // 1 hour ago
        last_accessed: now - Duration::from_secs(1800), // 30 minutes ago
    };
    
    let recent_allocation = MemoryAllocation {
        id: "recent_alloc".to_string(),
        offset: 256,
        size: 256,
        allocation_type: "recent".to_string(),
        created_at: now - Duration::from_secs(60), // 1 minute ago
        last_accessed: now - Duration::from_secs(10), // 10 seconds ago
    };
    
    {
        let mut alloc_map = instance.memory_allocations.write();
        alloc_map.insert("old_alloc".to_string(), old_allocation);
        alloc_map.insert("recent_alloc".to_string(), recent_allocation);
    }
    
    // Test age-based cleanup (keep allocations accessed within 1 hour)
    let cutoff_time = now - Duration::from_secs(3600);
    {
        let mut alloc_map = instance.memory_allocations.write();
        alloc_map.retain(|_, allocation| {
            allocation.last_accessed > cutoff_time
        });
    }
    
    // Only recent allocation should remain
    let alloc_map = instance.memory_allocations.read();
    assert_eq!(alloc_map.len(), 1);
    assert!(alloc_map.contains_key("recent_alloc"));
    assert!(!alloc_map.contains_key("old_alloc"));
}

#[test]
fn test_allocation_type_categorization() {
    let instance = create_mock_wasm_instance("type_test");
    
    let allocation_types = vec!["buffer", "array", "string", "object", "function"];
    
    // Create allocations of different types
    {
        let mut alloc_map = instance.memory_allocations.write();
        for (i, alloc_type) in allocation_types.iter().enumerate() {
            let allocation = MemoryAllocation {
                id: format!("alloc_{}", i),
                offset: (i * 256) as u32,
                size: 256,
                allocation_type: alloc_type.to_string(),
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
            };
            alloc_map.insert(format!("alloc_{}", i), allocation);
        }
    }
    
    // Count allocations by type
    let alloc_map = instance.memory_allocations.read();
    let mut type_counts = HashMap::new();
    
    for allocation in alloc_map.values() {
        *type_counts.entry(allocation.allocation_type.clone()).or_insert(0) += 1;
    }
    
    // Verify each type has exactly one allocation
    for alloc_type in allocation_types {
        assert_eq!(type_counts.get(alloc_type), Some(&1));
    }
}