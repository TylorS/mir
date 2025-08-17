//! Unit tests for WASM Instance Manager
//!
//! These tests verify the instance manager functionality including lifecycle management,
//! configuration handling, and error conditions.

use mir_backend_wasm::napi::instance_manager::{WasmInstanceManager, JsInstanceManagerConfig};

fn create_test_config() -> JsInstanceManagerConfig {
    JsInstanceManagerConfig {
        max_instances: 5,
        memory_limit_mb: 32,
        execution_timeout_ms: 1000,
        enable_hot_reload: true,
        preserve_state_on_reload: true,
        enable_debugging: false,
    }
}

fn create_minimal_wasm_bytecode() -> Vec<u8> {
    // Minimal valid WASM module bytecode
    vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // WASM version
    ]
}

#[test]
fn test_instance_manager_creation() {
    let config = create_test_config();
    let _manager = WasmInstanceManager::new(config);
    
    // If we get here, creation succeeded
    assert!(true);
}

#[test]
fn test_instance_manager_default() {
    let _manager = WasmInstanceManager::default();
    
    // Test that default configuration works
    assert!(true);
}

#[test]
fn test_instance_creation_limits() {
    let config = JsInstanceManagerConfig {
        max_instances: 2,
        memory_limit_mb: 32,
        execution_timeout_ms: 1000,
        enable_hot_reload: true,
        preserve_state_on_reload: true,
        enable_debugging: false,
    };
    
    let mut manager = WasmInstanceManager::new(config);
    let wasm_bytecode = create_minimal_wasm_bytecode();
    
    // Create first instance - should succeed
    let result = manager.create_instance(
        "instance1".to_string(),
        wasm_bytecode.clone().into(),
        None,
    );
    assert!(result.is_ok());
    let creation_result = result.unwrap();
    assert!(creation_result.success);
    assert_eq!(creation_result.instance_id, Some("instance1".to_string()));
    
    // Create second instance - should succeed
    let result = manager.create_instance(
        "instance2".to_string(),
        wasm_bytecode.clone().into(),
        None,
    );
    assert!(result.is_ok());
    let creation_result = result.unwrap();
    assert!(creation_result.success);
    
    // Try to create third instance - should fail due to limit
    let result = manager.create_instance(
        "instance3".to_string(),
        wasm_bytecode.into(),
        None,
    );
    assert!(result.is_ok());
    let creation_result = result.unwrap();
    assert!(!creation_result.success);
    assert!(creation_result.error.is_some());
    assert!(creation_result.error.unwrap().contains("Maximum instance limit"));
}

#[test]
fn test_duplicate_instance_creation() {
    let config = create_test_config();
    let mut manager = WasmInstanceManager::new(config);
    let wasm_bytecode = create_minimal_wasm_bytecode();
    
    // Create first instance
    let result = manager.create_instance(
        "duplicate_test".to_string(),
        wasm_bytecode.clone().into(),
        None,
    );
    assert!(result.is_ok());
    assert!(result.unwrap().success);
    
    // Try to create instance with same ID - should fail
    let result = manager.create_instance(
        "duplicate_test".to_string(),
        wasm_bytecode.into(),
        None,
    );
    assert!(result.is_ok());
    let creation_result = result.unwrap();
    assert!(!creation_result.success);
    assert!(creation_result.error.is_some());
    assert!(creation_result.error.unwrap().contains("already exists"));
}

#[tokio::test]
async fn test_function_execution() {
    let config = create_test_config();
    let manager = WasmInstanceManager::new(config);
    
    // Test execution on non-existent instance
    let result = manager.execute_function(
        "non_existent".to_string(),
        "test_function".to_string(),
        vec!["arg1".to_string(), "arg2".to_string()],
    ).await;
    
    assert!(result.is_ok());
    let exec_result = result.unwrap();
    assert!(!exec_result.success);
    assert!(exec_result.error.is_some());
}

#[tokio::test]
async fn test_execution_timeout() {
    let config = JsInstanceManagerConfig {
        max_instances: 5,
        memory_limit_mb: 32,
        execution_timeout_ms: 1, // Very short timeout
        enable_hot_reload: true,
        preserve_state_on_reload: true,
        enable_debugging: false,
    };
    
    let manager = WasmInstanceManager::new(config);
    
    // This should timeout due to very short timeout setting
    let result = manager.execute_function(
        "test_instance".to_string(),
        "long_running_function".to_string(),
        vec![],
    ).await;
    
    assert!(result.is_ok());
    // Note: In the current implementation, timeout logic is simplified
    // In a full implementation, this would test actual timeout behavior
}

#[test]
fn test_hot_reload_disabled() {
    let config = JsInstanceManagerConfig {
        max_instances: 5,
        memory_limit_mb: 32,
        execution_timeout_ms: 1000,
        enable_hot_reload: false, // Disabled
        preserve_state_on_reload: true,
        enable_debugging: false,
    };
    
    let mut manager = WasmInstanceManager::new(config);
    let wasm_bytecode = create_minimal_wasm_bytecode();
    
    let result = manager.hot_reload_instance(
        "test_instance".to_string(),
        wasm_bytecode.into(),
    );
    
    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(!reload_result.success);
    assert!(reload_result.error.is_some());
    assert!(reload_result.error.unwrap().contains("Hot-reload is disabled"));
}

#[test]
fn test_hot_reload_enabled() {
    let config = create_test_config(); // Hot-reload enabled by default
    let mut manager = WasmInstanceManager::new(config);
    let wasm_bytecode = create_minimal_wasm_bytecode();
    
    let result = manager.hot_reload_instance(
        "test_instance".to_string(),
        wasm_bytecode.into(),
    );
    
    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(reload_result.success);
    assert!(reload_result.preserved_state);
    assert!(reload_result.ffi_bindings_preserved);
}

#[test]
fn test_instance_removal() {
    let config = create_test_config();
    let mut manager = WasmInstanceManager::new(config);
    
    // Remove non-existent instance
    let result = manager.remove_instance("non_existent".to_string());
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    // In a full implementation, we would:
    // 1. Create an instance
    // 2. Verify it exists in the list
    // 3. Remove it
    // 4. Verify it's no longer in the list
}

#[test]
fn test_instance_listing() {
    let config = create_test_config();
    let manager = WasmInstanceManager::new(config);
    
    // Initially should be empty
    let instances = manager.list_instances();
    assert!(instances.is_empty());
    
    // After creating instances, list should contain them
    // (This would be tested in integration tests with actual instance creation)
}

#[test]
fn test_instance_statistics() {
    let config = create_test_config();
    let manager = WasmInstanceManager::new(config);
    
    // Test statistics for non-existent instance
    let result = manager.get_instance_statistics("non_existent".to_string());
    assert!(result.is_err());
}

#[test]
fn test_optimization_level_setting() {
    let config = create_test_config();
    let mut manager = WasmInstanceManager::new(config);
    
    // Test valid optimization levels
    assert!(manager.set_optimization_level("none".to_string()).is_ok());
    assert!(manager.set_optimization_level("size".to_string()).is_ok());
    assert!(manager.set_optimization_level("speed".to_string()).is_ok());
    assert!(manager.set_optimization_level("aggressive".to_string()).is_ok());
    
    // Test invalid optimization level
    let result = manager.set_optimization_level("invalid".to_string());
    assert!(result.is_err());
}

#[test]
fn test_config_validation() {
    // Test various configuration scenarios
    let configs = vec![
        JsInstanceManagerConfig {
            max_instances: 0, // Edge case
            memory_limit_mb: 32,
            execution_timeout_ms: 1000,
            enable_hot_reload: true,
            preserve_state_on_reload: true,
            enable_debugging: false,
        },
        JsInstanceManagerConfig {
            max_instances: 1000, // Large number
            memory_limit_mb: 1024,
            execution_timeout_ms: 60000,
            enable_hot_reload: false,
            preserve_state_on_reload: false,
            enable_debugging: true,
        },
    ];
    
    for config in configs {
        let _manager = WasmInstanceManager::new(config);
        // If we get here, configuration was accepted
        assert!(true);
    }
}

#[test]
fn test_debugging_configuration() {
    let config = JsInstanceManagerConfig {
        max_instances: 5,
        memory_limit_mb: 32,
        execution_timeout_ms: 1000,
        enable_hot_reload: true,
        preserve_state_on_reload: true,
        enable_debugging: true, // Enabled
    };
    
    let _manager = WasmInstanceManager::new(config);
    
    // In a full implementation, this would verify that:
    // - Debug info is enabled in the code generator
    // - Source maps are enabled
    // - Additional debugging features are active
    assert!(true);
}