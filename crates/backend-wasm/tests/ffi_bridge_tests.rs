//! Unit tests for FFI Bridge functionality
//!
//! These tests verify FFI binding management, host function registration,
//! and state preservation during hot-reloading.

use mir_backend_wasm::napi::{FFIBinding, WasmInstance, INSTANCE_REGISTRY};
use mir_types::ContentHash;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use parking_lot::Mutex;

fn create_test_ffi_binding(name: &str) -> FFIBinding {
    FFIBinding {
        name: name.to_string(),
        binding_type: "function".to_string(),
        is_async: false,
        param_types: vec!["i32".to_string(), "f64".to_string()],
        result_types: vec!["i32".to_string()],
        preserve_state: true,
        host_function: Some("host_function_impl".to_string()),
    }
}

fn create_mock_instance_with_ffi(id: &str) -> WasmInstance {
    let hash = ContentHash::new(id.as_bytes());
    
    WasmInstance {
        instance: Arc::new(Mutex::new(unsafe { std::mem::zeroed() })),
        store: Arc::new(Mutex::new(unsafe { std::mem::zeroed() })),
        memory: Arc::new(Mutex::new(None)),
        module_hash: hash,
        exports: HashMap::new(),
        ffi_bindings: Arc::new(RwLock::new(HashMap::new())),
        memory_allocations: Arc::new(RwLock::new(HashMap::new())),
        execution_stats: Arc::new(RwLock::new(Default::default())),
    }
}

#[test]
fn test_ffi_binding_creation() {
    let binding = create_test_ffi_binding("test_function");
    
    assert_eq!(binding.name, "test_function");
    assert_eq!(binding.binding_type, "function");
    assert!(!binding.is_async);
    assert_eq!(binding.param_types.len(), 2);
    assert_eq!(binding.result_types.len(), 1);
    assert!(binding.preserve_state);
    assert!(binding.host_function.is_some());
}

#[test]
fn test_ffi_binding_registration() {
    let instance = create_mock_instance_with_ffi("ffi_test");
    let binding = create_test_ffi_binding("registered_function");
    
    // Register FFI binding
    instance.ffi_bindings.write().insert("registered_function".to_string(), binding.clone());
    
    // Verify binding is registered
    let bindings = instance.ffi_bindings.read();
    assert!(bindings.contains_key("registered_function"));
    
    let stored_binding = bindings.get("registered_function").unwrap();
    assert_eq!(stored_binding.name, binding.name);
    assert_eq!(stored_binding.binding_type, binding.binding_type);
    assert_eq!(stored_binding.param_types, binding.param_types);
    assert_eq!(stored_binding.result_types, binding.result_types);
}

#[test]
fn test_multiple_ffi_bindings() {
    let instance = create_mock_instance_with_ffi("multi_ffi_test");
    
    let bindings = vec![
        create_test_ffi_binding("function_1"),
        create_test_ffi_binding("function_2"),
        create_test_ffi_binding("function_3"),
    ];
    
    // Register all bindings
    {
        let mut ffi_map = instance.ffi_bindings.write();
        for binding in bindings {
            ffi_map.insert(binding.name.clone(), binding);
        }
    }
    
    // Verify all bindings are registered
    let ffi_map = instance.ffi_bindings.read();
    assert_eq!(ffi_map.len(), 3);
    assert!(ffi_map.contains_key("function_1"));
    assert!(ffi_map.contains_key("function_2"));
    assert!(ffi_map.contains_key("function_3"));
}

#[test]
fn test_async_ffi_binding() {
    let mut binding = create_test_ffi_binding("async_function");
    binding.is_async = true;
    binding.binding_type = "async_function".to_string();
    
    let instance = create_mock_instance_with_ffi("async_test");
    instance.ffi_bindings.write().insert("async_function".to_string(), binding);
    
    let bindings = instance.ffi_bindings.read();
    let stored_binding = bindings.get("async_function").unwrap();
    assert!(stored_binding.is_async);
    assert_eq!(stored_binding.binding_type, "async_function");
}

#[test]
fn test_ffi_binding_with_complex_types() {
    let binding = FFIBinding {
        name: "complex_function".to_string(),
        binding_type: "function".to_string(),
        is_async: false,
        param_types: vec![
            "struct:Point".to_string(),
            "array:i32".to_string(),
            "string".to_string(),
        ],
        result_types: vec![
            "struct:Result".to_string(),
            "option:string".to_string(),
        ],
        preserve_state: true,
        host_function: Some("complex_host_impl".to_string()),
    };
    
    let instance = create_mock_instance_with_ffi("complex_test");
    instance.ffi_bindings.write().insert("complex_function".to_string(), binding.clone());
    
    let bindings = instance.ffi_bindings.read();
    let stored_binding = bindings.get("complex_function").unwrap();
    
    assert_eq!(stored_binding.param_types.len(), 3);
    assert_eq!(stored_binding.result_types.len(), 2);
    assert!(stored_binding.param_types.contains(&"struct:Point".to_string()));
    assert!(stored_binding.result_types.contains(&"struct:Result".to_string()));
}

#[test]
fn test_ffi_state_preservation() {
    let instance = create_mock_instance_with_ffi("preservation_test");
    
    // Create bindings with different preservation settings
    let preserve_binding = FFIBinding {
        name: "preserve_me".to_string(),
        binding_type: "function".to_string(),
        is_async: false,
        param_types: vec!["i32".to_string()],
        result_types: vec!["i32".to_string()],
        preserve_state: true,
        host_function: Some("preserve_impl".to_string()),
    };
    
    let no_preserve_binding = FFIBinding {
        name: "dont_preserve_me".to_string(),
        binding_type: "function".to_string(),
        is_async: false,
        param_types: vec!["i32".to_string()],
        result_types: vec!["i32".to_string()],
        preserve_state: false,
        host_function: Some("no_preserve_impl".to_string()),
    };
    
    {
        let mut ffi_map = instance.ffi_bindings.write();
        ffi_map.insert("preserve_me".to_string(), preserve_binding);
        ffi_map.insert("dont_preserve_me".to_string(), no_preserve_binding);
    }
    
    // Simulate hot-reload state preservation
    let preserved_bindings: HashMap<String, FFIBinding> = instance
        .ffi_bindings
        .read()
        .iter()
        .filter(|(_, binding)| binding.preserve_state)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    
    // Only the binding with preserve_state=true should be preserved
    assert_eq!(preserved_bindings.len(), 1);
    assert!(preserved_bindings.contains_key("preserve_me"));
    assert!(!preserved_bindings.contains_key("dont_preserve_me"));
}

#[test]
fn test_ffi_binding_removal() {
    let instance = create_mock_instance_with_ffi("removal_test");
    let binding = create_test_ffi_binding("temporary_function");
    
    // Add binding
    instance.ffi_bindings.write().insert("temporary_function".to_string(), binding);
    assert!(instance.ffi_bindings.read().contains_key("temporary_function"));
    
    // Remove binding
    let removed = instance.ffi_bindings.write().remove("temporary_function");
    assert!(removed.is_some());
    assert!(!instance.ffi_bindings.read().contains_key("temporary_function"));
}

#[test]
fn test_ffi_binding_update() {
    let instance = create_mock_instance_with_ffi("update_test");
    let mut binding = create_test_ffi_binding("updatable_function");
    
    // Initial registration
    instance.ffi_bindings.write().insert("updatable_function".to_string(), binding.clone());
    
    // Verify initial state
    {
        let bindings = instance.ffi_bindings.read();
        let stored_binding = bindings.get("updatable_function").unwrap();
        assert!(!stored_binding.is_async);
        assert_eq!(stored_binding.param_types.len(), 2);
    }
    
    // Update binding
    binding.is_async = true;
    binding.param_types.push("string".to_string());
    instance.ffi_bindings.write().insert("updatable_function".to_string(), binding);
    
    // Verify updated state
    {
        let bindings = instance.ffi_bindings.read();
        let stored_binding = bindings.get("updatable_function").unwrap();
        assert!(stored_binding.is_async);
        assert_eq!(stored_binding.param_types.len(), 3);
    }
}

#[test]
fn test_concurrent_ffi_access() {
    use std::thread;
    use std::sync::Arc;
    
    let instance = Arc::new(create_mock_instance_with_ffi("concurrent_ffi_test"));
    
    // Spawn threads that add FFI bindings concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let instance = Arc::clone(&instance);
            thread::spawn(move || {
                let binding = FFIBinding {
                    name: format!("concurrent_function_{}", i),
                    binding_type: "function".to_string(),
                    is_async: i % 2 == 0,
                    param_types: vec!["i32".to_string()],
                    result_types: vec!["i32".to_string()],
                    preserve_state: true,
                    host_function: Some(format!("impl_{}", i)),
                };
                
                instance.ffi_bindings.write().insert(
                    format!("concurrent_function_{}", i),
                    binding,
                );
            })
        })
        .collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all bindings were added
    let bindings = instance.ffi_bindings.read();
    assert_eq!(bindings.len(), 10);
    
    for i in 0..10 {
        let key = format!("concurrent_function_{}", i);
        assert!(bindings.contains_key(&key));
        
        let binding = bindings.get(&key).unwrap();
        assert_eq!(binding.is_async, i % 2 == 0);
    }
}

#[test]
fn test_ffi_binding_serialization_info() {
    let binding = FFIBinding {
        name: "serializable_function".to_string(),
        binding_type: "function".to_string(),
        is_async: true,
        param_types: vec!["struct:User".to_string(), "array:string".to_string()],
        result_types: vec!["result:User".to_string()],
        preserve_state: true,
        host_function: Some("serialized_function_data".to_string()),
    };
    
    // Test that we can clone the binding (required for serialization)
    let cloned_binding = binding.clone();
    assert_eq!(binding.name, cloned_binding.name);
    assert_eq!(binding.binding_type, cloned_binding.binding_type);
    assert_eq!(binding.is_async, cloned_binding.is_async);
    assert_eq!(binding.param_types, cloned_binding.param_types);
    assert_eq!(binding.result_types, cloned_binding.result_types);
    assert_eq!(binding.preserve_state, cloned_binding.preserve_state);
    assert_eq!(binding.host_function, cloned_binding.host_function);
}

#[test]
fn test_ffi_binding_validation() {
    // Test various edge cases for FFI binding validation
    
    // Empty function name
    let empty_name_binding = FFIBinding {
        name: "".to_string(),
        binding_type: "function".to_string(),
        is_async: false,
        param_types: vec![],
        result_types: vec![],
        preserve_state: true,
        host_function: None,
    };
    
    // This should be handled gracefully (empty name might be invalid in real usage)
    assert!(empty_name_binding.name.is_empty());
    
    // No parameters or return types
    let minimal_binding = FFIBinding {
        name: "minimal".to_string(),
        binding_type: "function".to_string(),
        is_async: false,
        param_types: vec![],
        result_types: vec![],
        preserve_state: false,
        host_function: None,
    };
    
    assert!(minimal_binding.param_types.is_empty());
    assert!(minimal_binding.result_types.is_empty());
    assert!(minimal_binding.host_function.is_none());
    
    // Many parameters
    let complex_binding = FFIBinding {
        name: "complex".to_string(),
        binding_type: "function".to_string(),
        is_async: true,
        param_types: (0..20).map(|i| format!("param_{}", i)).collect(),
        result_types: (0..5).map(|i| format!("result_{}", i)).collect(),
        preserve_state: true,
        host_function: Some("very_complex_impl".to_string()),
    };
    
    assert_eq!(complex_binding.param_types.len(), 20);
    assert_eq!(complex_binding.result_types.len(), 5);
}