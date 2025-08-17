# MIR Runtime Debugging and Troubleshooting Guide

This guide provides comprehensive information for debugging and troubleshooting issues with the MIR Runtime.

## Table of Contents

1. [Debugging Setup](#debugging-setup)
2. [Common Issues](#common-issues)
3. [Logging and Tracing](#logging-and-tracing)
4. [Performance Debugging](#performance-debugging)
5. [Hot-Reload Debugging](#hot-reload-debugging)
6. [Distributed System Debugging](#distributed-system-debugging)
7. [Memory and Resource Debugging](#memory-and-resource-debugging)
8. [Development Tools](#development-tools)

## Debugging Setup

### Enable Debug Mode

```rust
use mir_runtime_assembly::{RuntimeConfig, Environment};

let config = RuntimeConfig::builder()
    .with_environment(Environment::Development {
        auto_reload: true,
        file_watcher_enabled: true,
        aggressive_optimization: false, // Disable optimizations for debugging
    })
    .build()?;
```

### Logging Configuration

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Comprehensive logging setup
tracing_subscriber::registry()
    .with(
        tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(true)
    )
    .with(tracing_subscriber::filter::LevelFilter::DEBUG)
    .init();
```

### Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug

# Enable backtrace on panic
export RUST_BACKTRACE=1

# Enable full backtrace
export RUST_BACKTRACE=full

# Enable MIR-specific debugging
export MIR_DEBUG=1
export MIR_TRACE_EXECUTION=1
export MIR_TRACE_HMR=1
```

## Common Issues

### Runtime Startup Issues

#### Issue: Runtime fails to start

**Symptoms:**
- Runtime panics during initialization
- Configuration validation errors
- Component initialization failures

**Debugging Steps:**

1. **Validate Configuration**
   ```rust
   let config = RuntimeConfig::new();
   match config.validate() {
       Ok(()) => println!("Configuration is valid"),
       Err(e) => println!("Configuration error: {}", e),
   }
   ```

2. **Check System Resources**
   ```bash
   # Check available memory
   free -h
   
   # Check available disk space
   df -h
   
   # Check open file limits
   ulimit -n
   ```

3. **Enable Detailed Logging**
   ```rust
   use tracing::{info, debug, error};
   
   debug!("Starting runtime initialization");
   match runtime.start().await {
       Ok(()) => info!("Runtime started successfully"),
       Err(e) => {
           error!("Runtime startup failed: {}", e);
           // Print detailed error chain
           let mut source = e.source();
           while let Some(err) = source {
               error!("Caused by: {}", err);
               source = err.source();
           }
       }
   }
   ```

#### Issue: Backend initialization fails

**Symptoms:**
- Specific backend (WASM, JS, WASI) fails to initialize
- Backend not found errors

**Debugging Steps:**

1. **Check Backend Availability**
   ```rust
   let available_backends = runtime.backend_manager().available_backends().await;
   println!("Available backends: {:?}", available_backends);
   ```

2. **Check Backend Dependencies**
   ```bash
   # For WASM backend
   ldd libwasmtime.so
   
   # For Node.js integration
   node --version
   npm list
   ```

3. **Test Backend Individually**
   ```rust
   let backend_config = BackendConfig {
       backend_type: "wasm".to_string(),
       enabled: true,
       settings: HashMap::new(),
       optimization_level: OptimizationLevel::None,
       include_debug_info: true,
   };
   
   let backend = WasmBackend::new();
   match backend.initialize(&backend_config).await {
       Ok(()) => println!("WASM backend initialized successfully"),
       Err(e) => println!("WASM backend initialization failed: {}", e),
   }
   ```

### Module Execution Issues

#### Issue: Module execution fails

**Symptoms:**
- Runtime errors during module execution
- Unexpected results or crashes
- Performance issues

**Debugging Steps:**

1. **Enable Execution Tracing**
   ```rust
   use tracing::{instrument, debug, span, Level};
   
   #[instrument]
   async fn debug_execute_module(runtime: &MirRuntime, module_hash: ContentHash) {
       let _span = span!(Level::DEBUG, "module_execution", module_hash = %module_hash);
       
       debug!("Starting module execution");
       match runtime.execute_module(module_hash).await {
           Ok(result) => {
               debug!("Execution successful: {:?}", result);
           }
           Err(e) => {
               debug!("Execution failed: {}", e);
           }
       }
   }
   ```

2. **Check Module Validity**
   ```rust
   let module = runtime.components().module_store().get_module(module_hash)?;
   match module.validate() {
       Ok(()) => println!("Module is valid"),
       Err(e) => println!("Module validation failed: {}", e),
   }
   ```

3. **Inspect VM State**
   ```rust
   let vm = runtime.components().virtual_machine().read().await;
   let execution_context = vm.get_execution_context();
   println!("VM state: {:?}", execution_context.get_state());
   ```

## Logging and Tracing

### Structured Logging

```rust
use tracing::{info, warn, error, debug};

// Log with structured data
info!(
    module_hash = %module_hash,
    execution_time_ms = execution_time.as_millis(),
    memory_usage_bytes = memory_usage,
    "Module execution completed"
);

// Log errors with context
error!(
    error = %e,
    module_hash = %module_hash,
    function_name = "execute_module",
    "Module execution failed"
);
```

### Distributed Tracing

```rust
use tracing::{span, Level, Instrument};

async fn traced_operation() -> Result<(), Error> {
    let span = span!(Level::INFO, "distributed_operation", operation_id = "op123");
    
    async {
        // Your operation here
        let result = some_async_operation().await;
        
        // Nested span for sub-operations
        let sub_span = span!(Level::DEBUG, "sub_operation", step = "validation");
        let _enter = sub_span.enter();
        
        validate_result(&result)?;
        
        Ok(())
    }
    .instrument(span)
    .await
}
```

### Custom Debug Output

```rust
impl std::fmt::Debug for CustomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomType")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("metadata", &self.metadata)
            .finish()
    }
}
```

## Performance Debugging

### Execution Profiling

```rust
use std::time::Instant;

async fn profile_execution(runtime: &MirRuntime, module_hash: ContentHash) {
    let start = Instant::now();
    
    match runtime.execute_module(module_hash).await {
        Ok(result) => {
            let duration = start.elapsed();
            info!(
                execution_time_ms = duration.as_millis(),
                result_size = result.to_string().len(),
                "Execution profiling"
            );
        }
        Err(e) => {
            error!("Execution failed: {}", e);
        }
    }
}
```

### Memory Usage Tracking

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn get_memory_usage() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}
```

### Performance Metrics Collection

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct PerformanceMetrics {
    execution_times: Vec<Duration>,
    memory_usage: Vec<usize>,
    error_count: usize,
    success_count: usize,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            execution_times: Vec::new(),
            memory_usage: Vec::new(),
            error_count: 0,
            success_count: 0,
        }
    }
    
    fn record_execution(&mut self, duration: Duration, memory: usize, success: bool) {
        self.execution_times.push(duration);
        self.memory_usage.push(memory);
        
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }
    }
    
    fn average_execution_time(&self) -> Duration {
        if self.execution_times.is_empty() {
            return Duration::ZERO;
        }
        
        let total: Duration = self.execution_times.iter().sum();
        total / self.execution_times.len() as u32
    }
    
    fn success_rate(&self) -> f64 {
        let total = self.success_count + self.error_count;
        if total == 0 {
            return 0.0;
        }
        
        self.success_count as f64 / total as f64
    }
}
```

## Hot-Reload Debugging

### Change Analysis Debugging

```rust
use mir_runtime_assembly::{ChangeAnalysis, CompatibilityLevel};

async fn debug_hot_reload(
    runtime: &MirRuntime,
    old_hash: ContentHash,
    new_module_data: &[u8]
) -> Result<(), Box<dyn std::error::Error>> {
    // Store new module
    let new_hash = runtime.components().content_store().store(new_module_data);
    
    // Analyze changes
    let analysis = runtime.components().hmr_coordinator()
        .analyze_change(old_hash, new_hash).await?;
    
    debug!("Change analysis: {:?}", analysis);
    
    match analysis.compatibility {
        CompatibilityLevel::FullyCompatible => {
            info!("Change is fully compatible - hot reload should succeed");
        }
        CompatibilityLevel::BackwardCompatible => {
            info!("Change is backward compatible - migration may be needed");
            debug!("Required migrations: {:?}", analysis.required_migrations);
        }
        CompatibilityLevel::RequiresMigration => {
            warn!("Change requires migration - check migration functions");
            debug!("Schema changes: {:?}", analysis.schema_changes);
        }
        CompatibilityLevel::Breaking => {
            error!("Breaking change detected - hot reload will fail");
            debug!("Breaking changes: {:?}", analysis.breaking_changes);
        }
    }
    
    // Attempt hot reload with detailed logging
    match runtime.hot_reload_module(old_hash, new_module_data).await {
        Ok(()) => {
            info!("Hot reload successful");
        }
        Err(e) => {
            error!("Hot reload failed: {}", e);
            
            // Check if rollback occurred
            let current_module = runtime.components().module_store()
                .get_current_module().await?;
            if current_module.content_hash() == old_hash {
                info!("Rollback successful - system is stable");
            } else {
                error!("System may be in inconsistent state");
            }
        }
    }
    
    Ok(())
}
```

### State Migration Debugging

```rust
async fn debug_state_migration(
    runtime: &MirRuntime,
    old_schema: &Schema,
    new_schema: &Schema
) -> Result<(), Box<dyn std::error::Error>> {
    let migration_system = runtime.components().state_migration_system();
    
    // Check if migration is possible
    let migration_path = migration_system
        .compute_migration_path(old_schema.hash(), new_schema.hash())?;
    
    debug!("Migration path: {:?}", migration_path);
    
    if migration_path.is_empty() {
        warn!("No migration path found - manual migration required");
        return Ok(());
    }
    
    // Test migration with sample data
    let sample_data = create_sample_data(old_schema);
    
    match migration_system.migrate_data(sample_data, &migration_path).await {
        Ok(migrated_data) => {
            info!("Migration test successful");
            debug!("Migrated data: {:?}", migrated_data);
        }
        Err(e) => {
            error!("Migration test failed: {}", e);
        }
    }
    
    Ok(())
}
```

## Distributed System Debugging

### Consensus Debugging

```rust
async fn debug_consensus(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let consensus = runtime.components().consensus();
    
    // Check consensus status
    let status = consensus.get_status().await?;
    debug!("Consensus status: {:?}", status);
    
    // Check node health
    let nodes = consensus.get_cluster_nodes().await?;
    for node in nodes {
        let health = consensus.check_node_health(&node).await?;
        info!(node_id = %node, health = ?health, "Node health check");
    }
    
    // Monitor consensus rounds
    let mut consensus_events = consensus.subscribe_to_events().await?;
    
    tokio::spawn(async move {
        while let Some(event) = consensus_events.recv().await {
            debug!("Consensus event: {:?}", event);
        }
    });
    
    Ok(())
}
```

### Network Debugging

```rust
use std::net::SocketAddr;
use tokio::net::TcpStream;

async fn debug_network_connectivity(nodes: &[String]) {
    for node in nodes {
        match node.parse::<SocketAddr>() {
            Ok(addr) => {
                match TcpStream::connect(addr).await {
                    Ok(_) => {
                        info!(node = %node, "Network connectivity OK");
                    }
                    Err(e) => {
                        error!(node = %node, error = %e, "Network connectivity failed");
                    }
                }
            }
            Err(e) => {
                error!(node = %node, error = %e, "Invalid node address");
            }
        }
    }
}
```

### Distributed State Debugging

```rust
async fn debug_distributed_state(runtime: &MirRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let distributed_runtime = runtime.components().distributed_runtime();
    
    // Check CRDT states
    let crdts = distributed_runtime.get_all_crdts().await?;
    for (name, crdt) in crdts {
        debug!(crdt_name = %name, state = ?crdt.state(), "CRDT state");
    }
    
    // Check event loop status
    let event_loops = distributed_runtime.get_event_loops().await?;
    for (name, event_loop) in event_loops {
        let pending_tasks = event_loop.get_pending_tasks().await;
        debug!(
            event_loop = %name,
            pending_tasks = pending_tasks.len(),
            "Event loop status"
        );
    }
    
    Ok(())
}
```

## Memory and Resource Debugging

### Memory Leak Detection

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct MemoryTracker {
    allocations: Arc<Mutex<HashMap<usize, (usize, String)>>>,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn track_allocation(&self, ptr: usize, size: usize, location: String) {
        let mut allocations = self.allocations.lock().unwrap();
        allocations.insert(ptr, (size, location));
    }
    
    fn track_deallocation(&self, ptr: usize) {
        let mut allocations = self.allocations.lock().unwrap();
        allocations.remove(&ptr);
    }
    
    fn get_active_allocations(&self) -> Vec<(usize, usize, String)> {
        let allocations = self.allocations.lock().unwrap();
        allocations
            .iter()
            .map(|(ptr, (size, location))| (*ptr, *size, location.clone()))
            .collect()
    }
    
    fn total_allocated(&self) -> usize {
        let allocations = self.allocations.lock().unwrap();
        allocations.values().map(|(size, _)| size).sum()
    }
}
```

### Resource Usage Monitoring

```rust
use std::fs;
use std::process;

#[derive(Debug)]
struct ResourceUsage {
    memory_rss: usize,
    memory_vms: usize,
    cpu_time: f64,
    open_files: usize,
}

impl ResourceUsage {
    fn current() -> Result<Self, Box<dyn std::error::Error>> {
        let pid = process::id();
        
        // Read memory usage from /proc/pid/status
        let status = fs::read_to_string(format!("/proc/{}/status", pid))?;
        let mut memory_rss = 0;
        let mut memory_vms = 0;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                memory_rss = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("VmSize:") {
                memory_vms = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        // Read CPU time from /proc/pid/stat
        let stat = fs::read_to_string(format!("/proc/{}/stat", pid))?;
        let fields: Vec<&str> = stat.split_whitespace().collect();
        let utime: f64 = fields.get(13).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let stime: f64 = fields.get(14).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let cpu_time = (utime + stime) / 100.0; // Convert from clock ticks to seconds
        
        // Count open files
        let fd_dir = fs::read_dir(format!("/proc/{}/fd", pid))?;
        let open_files = fd_dir.count();
        
        Ok(ResourceUsage {
            memory_rss: memory_rss * 1024, // Convert from KB to bytes
            memory_vms: memory_vms * 1024,
            cpu_time,
            open_files,
        })
    }
}

async fn monitor_resources(runtime: &MirRuntime) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    
    loop {
        interval.tick().await;
        
        match ResourceUsage::current() {
            Ok(usage) => {
                info!(
                    memory_rss_mb = usage.memory_rss / 1024 / 1024,
                    memory_vms_mb = usage.memory_vms / 1024 / 1024,
                    cpu_time_seconds = usage.cpu_time,
                    open_files = usage.open_files,
                    "Resource usage"
                );
                
                // Check for resource leaks
                if usage.open_files > 1000 {
                    warn!("High number of open files detected - possible file descriptor leak");
                }
                
                if usage.memory_rss > 1024 * 1024 * 1024 { // 1GB
                    warn!("High memory usage detected - possible memory leak");
                }
            }
            Err(e) => {
                error!("Failed to read resource usage: {}", e);
            }
        }
    }
}
```

## Development Tools

### Interactive Debugger

```rust
use std::io::{self, Write};

struct InteractiveDebugger {
    runtime: Arc<MirRuntime>,
}

impl InteractiveDebugger {
    fn new(runtime: Arc<MirRuntime>) -> Self {
        Self { runtime }
    }
    
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("MIR Runtime Interactive Debugger");
        println!("Type 'help' for available commands");
        
        loop {
            print!("> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            match input {
                "help" => self.show_help(),
                "status" => self.show_status().await,
                "modules" => self.list_modules().await,
                "backends" => self.list_backends().await,
                "health" => self.show_health().await,
                "gc" => self.trigger_gc().await,
                "quit" | "exit" => break,
                cmd if cmd.starts_with("execute ") => {
                    let hash = &cmd[8..];
                    self.execute_module(hash).await;
                }
                cmd if cmd.starts_with("inspect ") => {
                    let hash = &cmd[8..];
                    self.inspect_module(hash).await;
                }
                _ => println!("Unknown command: {}", input),
            }
        }
        
        Ok(())
    }
    
    fn show_help(&self) {
        println!("Available commands:");
        println!("  help              - Show this help message");
        println!("  status            - Show runtime status");
        println!("  modules           - List all modules");
        println!("  backends          - List available backends");
        println!("  health            - Show component health");
        println!("  gc                - Trigger garbage collection");
        println!("  execute <hash>    - Execute a module");
        println!("  inspect <hash>    - Inspect a module");
        println!("  quit/exit         - Exit debugger");
    }
    
    async fn show_status(&self) {
        let info = self.runtime.get_runtime_info().await;
        println!("Runtime Status:");
        println!("  State: {:?}", info.state);
        println!("  Version: {}", info.version);
        println!("  Uptime: {:?}", info.uptime);
    }
    
    async fn list_modules(&self) {
        let module_store = self.runtime.components().module_store();
        match module_store.list_modules().await {
            Ok(modules) => {
                println!("Loaded Modules:");
                for module in modules {
                    println!("  {}: {}", module.content_hash(), module.name());
                }
            }
            Err(e) => println!("Error listing modules: {}", e),
        }
    }
    
    async fn list_backends(&self) {
        let backend_info = self.runtime.get_runtime_info().await.backend_info;
        println!("Available Backends:");
        for (name, info) in backend_info {
            println!("  {}: {:?}", name, info);
        }
    }
    
    async fn show_health(&self) {
        let info = self.runtime.get_runtime_info().await;
        println!("Component Health:");
        for (component, health) in info.component_health {
            println!("  {}: {:?}", component, health);
        }
    }
    
    async fn trigger_gc(&self) {
        println!("Triggering garbage collection...");
        // In a real implementation, this would trigger GC
        println!("Garbage collection completed");
    }
    
    async fn execute_module(&self, hash_str: &str) {
        match hash_str.parse::<ContentHash>() {
            Ok(hash) => {
                match self.runtime.execute_module(hash).await {
                    Ok(result) => println!("Execution result: {:?}", result),
                    Err(e) => println!("Execution failed: {}", e),
                }
            }
            Err(e) => println!("Invalid hash: {}", e),
        }
    }
    
    async fn inspect_module(&self, hash_str: &str) {
        match hash_str.parse::<ContentHash>() {
            Ok(hash) => {
                let module_store = self.runtime.components().module_store();
                match module_store.get_module(hash) {
                    Ok(Some(module)) => {
                        println!("Module Information:");
                        println!("  Hash: {}", module.content_hash());
                        println!("  Name: {}", module.name());
                        println!("  Size: {} bytes", module.size());
                        println!("  Exports: {:?}", module.exports());
                        println!("  Imports: {:?}", module.imports());
                    }
                    Ok(None) => println!("Module not found"),
                    Err(e) => println!("Error inspecting module: {}", e),
                }
            }
            Err(e) => println!("Invalid hash: {}", e),
        }
    }
}
```

### Automated Testing Tools

```rust
#[cfg(test)]
mod debug_tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_runtime_startup() {
        let config = RuntimeConfig::builder()
            .with_environment(Environment::Testing {
                deterministic_execution: true,
                chaos_testing_enabled: false,
            })
            .build()
            .unwrap();
        
        let mut runtime = MirRuntime::new(config).await.unwrap();
        
        // Test startup with timeout
        let startup_result = timeout(Duration::from_secs(30), runtime.start()).await;
        assert!(startup_result.is_ok(), "Runtime startup timed out");
        assert!(startup_result.unwrap().is_ok(), "Runtime startup failed");
        
        // Test runtime state
        assert_eq!(runtime.state().await, RuntimeState::Running);
        
        // Test shutdown
        let shutdown_result = timeout(Duration::from_secs(10), runtime.shutdown()).await;
        assert!(shutdown_result.is_ok(), "Runtime shutdown timed out");
        assert!(shutdown_result.unwrap().is_ok(), "Runtime shutdown failed");
    }
    
    #[tokio::test]
    async fn test_hot_reload_compatibility() {
        let mut runtime = create_test_runtime().await;
        runtime.start().await.unwrap();
        
        let v1_module = create_test_module_v1();
        let v2_module = create_test_module_v2();
        
        let v1_hash = runtime.components().content_store().store(&v1_module);
        
        // Test hot reload
        let reload_result = runtime.hot_reload_module(v1_hash, &v2_module).await;
        assert!(reload_result.is_ok(), "Hot reload should succeed for compatible changes");
        
        runtime.shutdown().await.unwrap();
    }
    
    async fn create_test_runtime() -> MirRuntime {
        let config = RuntimeConfig::builder()
            .with_environment(Environment::Testing {
                deterministic_execution: true,
                chaos_testing_enabled: false,
            })
            .build()
            .unwrap();
        
        MirRuntime::new(config).await.unwrap()
    }
    
    fn create_test_module_v1() -> Vec<u8> {
        // Create a simple test module
        vec![1, 2, 3, 4] // Placeholder
    }
    
    fn create_test_module_v2() -> Vec<u8> {
        // Create a compatible updated module
        vec![1, 2, 3, 5] // Placeholder
    }
}
```

This debugging guide provides comprehensive tools and techniques for troubleshooting the MIR Runtime. Use these tools to identify and resolve issues during development and production deployment.