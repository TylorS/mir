//! Unit tests for NAPI bindings structure
//!
//! These tests verify that the NAPI bindings compile and have the correct structure.
//! Full integration tests require a Node.js runtime environment.

use mir_backend_wasm::WasmNapiBindings;

#[test]
fn test_napi_bindings_creation() {
    // Test that we can create NAPI bindings
    let _bindings = WasmNapiBindings::new();
    
    // If we get here, creation succeeded
    assert!(true);
}

#[test]
fn test_instance_management_basic() {
    let bindings = WasmNapiBindings::new();
    
    // Test listing instances (should be empty initially)
    let instances = bindings.list_instances();
    assert!(instances.is_empty());
    
    // Test disposing non-existent instance
    let mut bindings = bindings;
    let result = bindings.dispose_instance("non_existent".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[test]
fn test_optimization_level_setting() {
    let mut bindings = WasmNapiBindings::new();
    
    // Test setting valid optimization levels
    assert!(bindings.set_optimization_level("none".to_string()).is_ok());
    assert!(bindings.set_optimization_level("size".to_string()).is_ok());
    assert!(bindings.set_optimization_level("speed".to_string()).is_ok());
    assert!(bindings.set_optimization_level("aggressive".to_string()).is_ok());
    
    // Test setting invalid optimization level
    let result = bindings.set_optimization_level("invalid".to_string());
    assert!(result.is_err());
}

#[test]
fn test_memory_operations_error_handling() {
    let bindings = WasmNapiBindings::new();
    
    // Test operations on non-existent instance
    let result = bindings.allocate_memory(
        "non_existent".to_string(),
        1024,
        "buffer".to_string(),
    );
    assert!(!result.success);
    assert!(result.error.is_some());
    
    let result = bindings.read_memory("non_existent".to_string(), 0, 100);
    assert!(result.is_err());
    
    let result = bindings.write_memory(
        "non_existent".to_string(),
        0,
        vec![1, 2, 3, 4].into(),
    );
    assert!(!result);
    
    let result = bindings.free_memory(
        "non_existent".to_string(),
        "allocation_id".to_string(),
    );
    assert!(!result);
}

#[test]
fn test_statistics_operations() {
    let bindings = WasmNapiBindings::new();
    
    // Test getting stats for non-existent instance
    let result = bindings.get_memory_stats("non_existent".to_string());
    assert!(result.is_err());
    
    let result = bindings.get_instance_stats("non_existent".to_string());
    assert!(result.is_err());
    
    // Test global FFI stats (should work even without instances)
    let result = bindings.get_ffi_stats();
    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.total_host_bindings, 0);
    assert_eq!(stats.total_wasm_exports, 0);
}

#[test]
fn test_value_conversion() {
    let bindings = WasmNapiBindings::new();
    
    // Test valid conversions
    let conversion = mir_backend_wasm::napi::JsValueConversion {
        source_value: "42".to_string(),
        source_type: "string".to_string(),
        target_type: "i32".to_string(),
        conversion_options: None,
    };
    
    let result = bindings.convert_js_to_wasm(conversion);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
    
    // Test invalid conversion
    let conversion = mir_backend_wasm::napi::JsValueConversion {
        source_value: "not_a_number".to_string(),
        source_type: "string".to_string(),
        target_type: "i32".to_string(),
        conversion_options: None,
    };
    
    let result = bindings.convert_js_to_wasm(conversion);
    assert!(result.is_err());
}

#[test]
fn test_batch_conversion() {
    let bindings = WasmNapiBindings::new();
    
    let conversions = vec![
        mir_backend_wasm::napi::JsValueConversion {
            source_value: "42".to_string(),
            source_type: "string".to_string(),
            target_type: "i32".to_string(),
            conversion_options: None,
        },
        mir_backend_wasm::napi::JsValueConversion {
            source_value: "3.14".to_string(),
            source_type: "string".to_string(),
            target_type: "f32".to_string(),
            conversion_options: None,
        },
        mir_backend_wasm::napi::JsValueConversion {
            source_value: "invalid".to_string(),
            source_type: "string".to_string(),
            target_type: "i32".to_string(),
            conversion_options: None,
        },
    ];
    
    let result = bindings.batch_convert_values(conversions);
    assert!(result.is_ok());
    
    let batch_result = result.unwrap();
    assert_eq!(batch_result.converted_values.len(), 3);
    assert_eq!(batch_result.errors.len(), 3);
    assert!(batch_result.success_rate > 0.0 && batch_result.success_rate < 1.0);
}

#[test]
fn test_garbage_collection_operations() {
    let bindings = WasmNapiBindings::new();
    
    // Test garbage collection on non-existent instance
    let result = bindings.garbage_collect("non_existent".to_string());
    assert!(result.is_err());
    
    // Test memory compaction on non-existent instance
    let result = bindings.compact_memory("non_existent".to_string());
    assert!(result.is_err());
}

// Note: Full NAPI integration tests require Node.js runtime and cannot be run
// in a standard Rust test environment. The NAPI bindings are designed to be
// called from JavaScript/Node.js, not from Rust unit tests.
//
// To test the full functionality:
// 1. Build the NAPI module: `cargo build --release`
// 2. Create Node.js bindings
// 3. Run the JavaScript example: `node examples/napi_usage.js`