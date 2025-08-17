//! Basic MIR Runtime Example
//!
//! This example demonstrates how to create, configure, and run the MIR runtime
//! with basic hot-module-reloading capabilities.

use mir_runtime_assembly::{MirRuntime, RuntimeConfig, Environment, UpdatePolicy, ValidationLevel};
use std::time::Duration;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting MIR Runtime Example");

    // Create runtime configuration
    let config = RuntimeConfig::builder()
        .with_environment(Environment::Development {
            auto_reload: true,
            file_watcher_enabled: true,
            aggressive_optimization: false,
        })
        .with_backend("wasm")
        .with_backend("js")
        .with_max_concurrent_executions(4)
        .with_execution_timeout(Duration::from_secs(30))
        .with_sandbox(true)
        .build()?;

    info!("Runtime configuration created: {:?}", config);

    // Create and start the runtime
    let mut runtime = MirRuntime::new(config).await?;
    
    info!("Runtime created, starting...");
    runtime.start().await?;
    
    info!("Runtime started successfully!");

    // Get runtime information
    let runtime_info = runtime.get_runtime_info().await;
    info!("Runtime info: {:?}", runtime_info);

    // Simulate some runtime operation
    info!("Runtime is now operational. In a real application, you would:");
    info!("1. Load and execute modules");
    info!("2. Handle hot-reloading events");
    info!("3. Coordinate distributed updates");
    info!("4. Monitor runtime health");

    // Wait a bit to simulate runtime operation
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Graceful shutdown
    info!("Shutting down runtime...");
    runtime.shutdown().await?;
    
    info!("Runtime shut down successfully!");
    Ok(())
}