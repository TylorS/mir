# MIR Runtime Assembly

The MIR Runtime Assembly is the complete distributed hot-module-reloading runtime for MIR. It integrates all MIR components into a unified runtime system that enables seamless code updates across distributed systems while preserving application state.

## Features

- **Hot-Module-Reloading**: Update running code without losing state or breaking connections
- **Distributed Coordination**: Built-in consensus mechanisms for coordinating updates across multiple nodes
- **Content-Addressable Storage**: All code and data are hashed and versioned for precise change detection
- **Schema-Driven Migrations**: Automatic data migration when types evolve
- **Multiple Backends**: Supports WASM, JavaScript/TypeScript, and WASI targets
- **Built-in Observability**: First-class OpenTelemetry integration with automatic instrumentation
- **FFI Integration**: Foreign function interface for host environment interaction

## Quick Start

### Basic Usage

```rust
use mir_runtime_assembly::{MirRuntime, RuntimeConfig, Environment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create runtime configuration
    let config = RuntimeConfig::builder()
        .with_environment(Environment::Development {
            auto_reload: true,
            file_watcher_enabled: true,
            aggressive_optimization: false,
        })
        .with_backend("wasm")
        .build()?;

    // Create and start the runtime
    let mut runtime = MirRuntime::new(config).await?;
    runtime.start().await?;

    // Runtime is now ready to execute modules and handle hot-reloading
    
    // Graceful shutdown
    runtime.shutdown().await?;
    Ok(())
}
```

### Distributed Setup

```rust
use mir_runtime_assembly::{
    MirRuntime, RuntimeConfig, Environment, UpdatePolicy, 
    ConsensusConfig, HMRConfig
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hmr_config = HMRConfig {
        environment: Environment::Production {
            require_explicit_deployment: true,
            staged_rollout: true,
            canary_percentage: 10.0,
        },
        update_policy: UpdatePolicy::Consensus,
        // ... other config
    };

    let consensus_config = ConsensusConfig {
        node_id: "node-1".to_string(),
        cluster_nodes: vec!["node-1".to_string(), "node-2".to_string()],
        // ... other config
    };

    let config = RuntimeConfig::builder()
        .with_hmr_config(hmr_config)
        .with_consensus_config(consensus_config)
        .build()?;

    let mut runtime = MirRuntime::new(config).await?;
    runtime.start().await?;

    // Distributed runtime is now operational
    
    runtime.shutdown().await?;
    Ok(())
}
```

## Architecture

The runtime is organized into several key components:

### Core Components

- **Type Registry**: Manages the sophisticated type system with RTTI support
- **Virtual Machine**: Executes MIR IR with optimization and state management
- **HMR Coordinator**: Orchestrates hot-reloading across distributed nodes
- **Content-Addressable Store**: Provides versioned storage for code and data
- **FFI Bridge**: Manages foreign function interface with host environments
- **OpenTelemetry Integration**: Provides comprehensive observability

### Backend Support

- **WASM Backend**: Compiles to WebAssembly for efficient execution
- **JavaScript Backend**: Generates JavaScript/TypeScript code
- **WASI Backend**: Supports WebAssembly System Interface

### Distributed Features

- **Consensus Protocol**: Coordinates updates across multiple nodes
- **CRDT Types**: Built-in conflict-free replicated data types
- **Distributed Primitives**: Event loops, schedulers, queues, pub-sub channels

## Configuration

The runtime supports extensive configuration for different environments and use cases:

### Environment Configuration

```rust
// Development environment
Environment::Development {
    auto_reload: true,
    file_watcher_enabled: true,
    aggressive_optimization: false,
}

// Production environment
Environment::Production {
    require_explicit_deployment: true,
    staged_rollout: true,
    canary_percentage: 10.0,
}

// Testing environment
Environment::Testing {
    deterministic_execution: true,
    chaos_testing_enabled: false,
}
```

### Backend Configuration

```rust
let config = RuntimeConfig::builder()
    .with_backend("wasm")      // Enable WASM backend
    .with_backend("js")        // Enable JavaScript backend
    .with_backend("wasi")      // Enable WASI backend
    .build()?;
```

### Security Configuration

```rust
let config = RuntimeConfig::builder()
    .with_sandbox(true)        // Enable sandboxing
    .with_max_concurrent_executions(4)
    .with_execution_timeout(Duration::from_secs(30))
    .build()?;
```

## Hot-Module-Reloading

The runtime provides sophisticated hot-module-reloading capabilities:

### Automatic Change Detection

The runtime automatically detects changes in modules and determines which components need to be updated:

```rust
// The runtime will automatically detect this change and update only affected components
runtime.hot_reload_module(old_hash, new_module_data).await?;
```

### State Preservation

During hot-reloading, the runtime preserves application state:

- **Compatible Changes**: State is automatically migrated
- **Schema Changes**: User-defined migration functions are executed
- **Breaking Changes**: Clear error messages and migration guidance

### Distributed Coordination

In distributed deployments, the runtime coordinates updates across all nodes:

- **Consensus-based Updates**: All nodes must agree before applying changes
- **Rolling Updates**: Updates are applied gradually across the cluster
- **Rollback Support**: Failed updates are automatically rolled back

## Observability

The runtime includes comprehensive observability features:

### OpenTelemetry Integration

```rust
// Automatic instrumentation
let config = RuntimeConfig::builder()
    .with_telemetry_config(InstrumentationConfig {
        auto_instrument: true,
        sampling_rate: 1.0,
        export_endpoint: "http://jaeger:14268/api/traces".to_string(),
    })
    .build()?;
```

### Structured Logging

```rust
// The runtime provides structured logging out of the box
// Logs include context about HMR operations, distributed coordination, etc.
```

### Metrics and Tracing

- **Performance Metrics**: Execution time, memory usage, instruction counts
- **HMR Metrics**: Update frequency, success rates, rollback statistics
- **Distributed Metrics**: Consensus latency, node health, network partitions

## Development Tools Integration

The runtime integrates with common development tools:

### File Watchers

```rust
let dev_tools_config = DevToolsConfig {
    file_watcher_enabled: true,
    watch_patterns: vec!["src/**/*.rs".to_string()],
    ignore_patterns: vec!["target/**".to_string()],
};
```

### Build Tool Integration

- **Webpack**: Automatic integration with webpack dev server
- **Vite**: Support for Vite's hot-reloading
- **Custom Build Tools**: Extensible plugin system

### Version Control Integration

- **Git Integration**: Track code versions and enable rollbacks to specific commits
- **CI/CD APIs**: Automated deployment integration

## Examples

See the `examples/` directory for complete examples:

- `basic_runtime.rs`: Basic runtime setup and usage
- `distributed_runtime.rs`: Distributed runtime with consensus
- `hot_reload_demo.rs`: Hot-reloading demonstration
- `observability_demo.rs`: Observability and monitoring setup

## API Reference

### MirRuntime

The main runtime class that coordinates all components.

#### Methods

- `new(config: RuntimeConfig) -> Result<Self, RuntimeError>`: Create a new runtime
- `start() -> Result<(), RuntimeError>`: Start the runtime
- `shutdown() -> Result<(), RuntimeError>`: Gracefully shutdown the runtime
- `execute_module(hash: ContentHash) -> Result<serde_json::Value, RuntimeError>`: Execute a module
- `hot_reload_module(old_hash: ContentHash, new_data: &[u8]) -> Result<(), RuntimeError>`: Hot-reload a module
- `get_runtime_info() -> RuntimeInfo`: Get runtime statistics and health information

### RuntimeConfig

Configuration for the runtime.

#### Builder Methods

- `with_environment(env: Environment) -> Self`: Set the runtime environment
- `with_backend(backend: &str) -> Self`: Enable a backend
- `with_hmr_config(config: HMRConfig) -> Self`: Set HMR configuration
- `with_telemetry_config(config: InstrumentationConfig) -> Self`: Set telemetry configuration
- `with_max_concurrent_executions(max: usize) -> Self`: Set execution limits
- `with_execution_timeout(timeout: Duration) -> Self`: Set execution timeout
- `with_sandbox(enabled: bool) -> Self`: Enable/disable sandboxing
- `build() -> Result<RuntimeConfig, ConfigError>`: Build the configuration

## Error Handling

The runtime provides comprehensive error handling:

```rust
match runtime.execute_module(module_hash).await {
    Ok(result) => println!("Execution result: {:?}", result),
    Err(RuntimeError::ComponentError(msg)) => {
        eprintln!("Component error: {}", msg);
    }
    Err(RuntimeError::BackendError(msg)) => {
        eprintln!("Backend error: {}", msg);
    }
    Err(e) => {
        eprintln!("Runtime error: {}", e);
    }
}
```

## Performance Considerations

### Memory Management

- **Garbage Collection**: Configurable GC strategies (mark-and-sweep, generational, concurrent)
- **Memory Limits**: Configurable heap and stack size limits
- **Memory Pressure**: Automatic handling of memory pressure situations

### Execution Optimization

- **Recursion Optimization**: Automatic tail recursion optimization
- **Instruction Caching**: Compiled instruction caching for repeated executions
- **Backend Selection**: Automatic backend selection based on workload characteristics

### Distributed Performance

- **Consensus Optimization**: Optimized consensus protocols for low latency
- **Network Batching**: Batched network operations for efficiency
- **Load Balancing**: Automatic load balancing across cluster nodes

## Security

### Sandboxing

The runtime provides comprehensive sandboxing:

```rust
let security_config = SecuritySettings {
    enable_sandbox: true,
    allowed_syscalls: vec!["read", "write", "open", "close"],
    resource_limits: ResourceLimits {
        max_file_descriptors: 1024,
        max_network_connections: 100,
        max_cpu_time: Duration::from_secs(10),
        max_memory_per_execution: 256 * 1024 * 1024, // 256MB
    },
    capability_based: true,
};
```

### Capability-Based Security

- **Fine-grained Permissions**: Modules can only access explicitly granted capabilities
- **Resource Limits**: Configurable limits on system resources
- **Network Isolation**: Optional network isolation for untrusted code

## Troubleshooting

### Common Issues

1. **Runtime Won't Start**
   - Check configuration validity with `config.validate()`
   - Ensure all required backends are available
   - Check system resource availability

2. **Hot-Reloading Fails**
   - Verify module compatibility with `analyze_change()`
   - Check for breaking schema changes
   - Review migration function implementations

3. **Distributed Coordination Issues**
   - Verify network connectivity between nodes
   - Check consensus configuration
   - Monitor node health status

### Debug Mode

Enable debug mode for detailed logging:

```rust
let config = RuntimeConfig::builder()
    .with_environment(Environment::Development {
        auto_reload: true,
        file_watcher_enabled: true,
        aggressive_optimization: false,
    })
    .build()?;
```

### Monitoring

Use the built-in monitoring endpoints:

```rust
let runtime_info = runtime.get_runtime_info().await;
println!("Runtime state: {:?}", runtime_info.state);
println!("Component health: {:?}", runtime_info.component_health);
```

## Contributing

See the main MIR repository for contribution guidelines.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.