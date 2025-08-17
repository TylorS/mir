//! Distributed MIR Runtime Example
//!
//! This example demonstrates how to set up a distributed MIR runtime
//! with multiple nodes and consensus-based hot-module-reloading.

use mir_runtime_assembly::{
    MirRuntime, RuntimeConfig, Environment, UpdatePolicy, ValidationLevel,
    ConsensusConfig, HMRConfig
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

    info!("Starting Distributed MIR Runtime Example");

    // Create distributed runtime configuration
    let hmr_config = HMRConfig {
        environment: Environment::Production {
            require_explicit_deployment: true,
            staged_rollout: true,
            canary_percentage: 10.0,
        },
        update_policy: UpdatePolicy::Consensus,
        validation_level: ValidationLevel::Extensive,
        rollback_strategy: mir_runtime_assembly::RollbackStrategy::Automatic,
        consensus_timeout: Duration::from_secs(30),
        state_preservation: mir_runtime_assembly::StatePreservationConfig::default(),
    };

    let consensus_config = ConsensusConfig {
        node_id: "node-1".to_string(),
        cluster_nodes: vec![
            "node-1".to_string(),
            "node-2".to_string(),
            "node-3".to_string(),
        ],
        consensus_timeout: Duration::from_secs(10),
        heartbeat_interval: Duration::from_secs(1),
        election_timeout: Duration::from_secs(5),
    };

    let config = RuntimeConfig::builder()
        .with_hmr_config(hmr_config)
        .with_consensus_config(consensus_config)
        .with_backend("wasm")
        .with_max_concurrent_executions(8)
        .with_execution_timeout(Duration::from_secs(60))
        .build()?;

    info!("Distributed runtime configuration created");

    // Create and start the runtime
    let mut runtime = MirRuntime::new(config).await?;
    
    info!("Distributed runtime created, starting...");
    runtime.start().await?;
    
    info!("Distributed runtime started successfully!");

    // Get runtime information
    let runtime_info = runtime.get_runtime_info().await;
    info!("Runtime info: {:?}", runtime_info);

    // Simulate distributed operations
    info!("Distributed runtime is now operational. Features available:");
    info!("1. Consensus-based hot-module-reloading");
    info!("2. Distributed state management");
    info!("3. Cluster-wide update coordination");
    info!("4. Fault-tolerant execution");
    info!("5. Rolling updates with canary deployments");

    // Simulate some runtime operation time
    tokio::time::sleep(Duration::from_secs(3)).await;

    // In a real distributed setup, you would:
    // - Connect to other nodes in the cluster
    // - Participate in consensus protocols
    // - Handle node failures and network partitions
    // - Coordinate distributed hot-reloading
    
    info!("Simulating distributed operations...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Graceful shutdown
    info!("Shutting down distributed runtime...");
    runtime.shutdown().await?;
    
    info!("Distributed runtime shut down successfully!");
    Ok(())
}