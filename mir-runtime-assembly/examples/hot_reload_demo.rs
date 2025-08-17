//! Hot-Module-Reloading Demo
//!
//! This example demonstrates the hot-module-reloading capabilities of the MIR runtime,
//! showing how modules can be updated without losing application state.

use mir_runtime_assembly::{
    MirRuntime, RuntimeConfig, Environment, UpdatePolicy, ValidationLevel
};
use std::time::Duration;
use tracing::{info, warn, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Hot-Module-Reloading Demo");

    // Create runtime configuration optimized for development and hot-reloading
    let config = RuntimeConfig::builder()
        .with_environment(Environment::Development {
            auto_reload: true,
            file_watcher_enabled: true,
            aggressive_optimization: false, // Disable aggressive optimization for faster reloads
        })
        .with_backend("wasm")
        .with_backend("js")
        .with_max_concurrent_executions(2)
        .with_execution_timeout(Duration::from_secs(10))
        .build()?;

    // Create and start the runtime
    let mut runtime = MirRuntime::new(config).await?;
    runtime.start().await?;
    
    info!("Runtime started - ready for hot-reloading demo");

    // Simulate loading an initial module
    info!("=== Phase 1: Loading Initial Module ===");
    let initial_module_data = create_sample_module_v1();
    let initial_hash = runtime.components().content_store().store(&initial_module_data);
    
    info!("Initial module stored with hash: {}", initial_hash);
    
    // Execute the initial module
    match runtime.execute_module(initial_hash).await {
        Ok(result) => info!("Initial execution result: {:?}", result),
        Err(e) => warn!("Initial execution failed: {}", e),
    }

    // Wait a bit to simulate runtime operation
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Simulate a hot-reload with a compatible change
    info!("=== Phase 2: Hot-Reloading with Compatible Change ===");
    let updated_module_data = create_sample_module_v2_compatible();
    
    match runtime.hot_reload_module(initial_hash, &updated_module_data).await {
        Ok(()) => info!("Hot-reload successful - compatible change applied"),
        Err(e) => error!("Hot-reload failed: {}", e),
    }

    // Execute the updated module
    let updated_hash = runtime.components().content_store().store(&updated_module_data);
    match runtime.execute_module(updated_hash).await {
        Ok(result) => info!("Updated execution result: {:?}", result),
        Err(e) => warn!("Updated execution failed: {}", e),
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Simulate a hot-reload with a breaking change
    info!("=== Phase 3: Hot-Reloading with Breaking Change ===");
    let breaking_module_data = create_sample_module_v3_breaking();
    
    match runtime.hot_reload_module(updated_hash, &breaking_module_data).await {
        Ok(()) => info!("Hot-reload successful - breaking change handled"),
        Err(e) => {
            warn!("Hot-reload failed as expected for breaking change: {}", e);
            info!("This demonstrates the runtime's safety mechanisms");
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Demonstrate state preservation
    info!("=== Phase 4: State Preservation Demo ===");
    info!("In a real application, the runtime would:");
    info!("1. Extract current application state");
    info!("2. Apply the module update");
    info!("3. Migrate state to new schema (if needed)");
    info!("4. Restore state in the updated module");
    info!("5. Resume execution without losing data");

    // Show runtime statistics
    let runtime_info = runtime.get_runtime_info().await;
    info!("=== Runtime Statistics ===");
    info!("Runtime state: {:?}", runtime_info.state);
    info!("Uptime: {:?}", runtime_info.uptime);
    info!("Component health: {:?}", runtime_info.component_health);

    // Graceful shutdown
    info!("=== Shutting Down ===");
    runtime.shutdown().await?;
    
    info!("Hot-Module-Reloading demo completed successfully!");
    Ok(())
}

/// Create a sample module (version 1)
fn create_sample_module_v1() -> Vec<u8> {
    // In a real implementation, this would be actual MIR IR
    // For demo purposes, we'll use JSON to represent a simple module
    let module = serde_json::json!({
        "version": "1.0.0",
        "name": "sample_module",
        "functions": [
            {
                "name": "greet",
                "parameters": ["name"],
                "body": "Hello, " + "name" + "!"
            }
        ],
        "state": {
            "counter": 0,
            "messages": []
        }
    });
    
    serde_json::to_vec(&module).unwrap()
}

/// Create a sample module (version 2 - compatible change)
fn create_sample_module_v2_compatible() -> Vec<u8> {
    // This version adds a new function but doesn't change existing interfaces
    let module = serde_json::json!({
        "version": "2.0.0",
        "name": "sample_module",
        "functions": [
            {
                "name": "greet",
                "parameters": ["name"],
                "body": "Hello, " + "name" + "!"
            },
            {
                "name": "farewell",
                "parameters": ["name"],
                "body": "Goodbye, " + "name" + "!"
            }
        ],
        "state": {
            "counter": 0,
            "messages": [],
            "last_greeted": null  // New field with default value
        }
    });
    
    serde_json::to_vec(&module).unwrap()
}

/// Create a sample module (version 3 - breaking change)
fn create_sample_module_v3_breaking() -> Vec<u8> {
    // This version changes the signature of an existing function
    let module = serde_json::json!({
        "version": "3.0.0",
        "name": "sample_module",
        "functions": [
            {
                "name": "greet",
                "parameters": ["name", "language"],  // Breaking: added required parameter
                "body": "greet_in_language(name, language)"
            },
            {
                "name": "farewell",
                "parameters": ["name"],
                "body": "Goodbye, " + "name" + "!"
            }
        ],
        "state": {
            "counter": 0,
            "messages": [],
            "preferred_language": "en"  // Changed field name (breaking)
        }
    });
    
    serde_json::to_vec(&module).unwrap()
}