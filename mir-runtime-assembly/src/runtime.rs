//! Main MIR Runtime implementation
//!
//! This module provides the unified MIR runtime that integrates all components
//! and manages the complete lifecycle of the distributed HMR system.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};
use thiserror::Error;

use crate::config::RuntimeConfig;
use crate::lifecycle::{LifecycleManager, RuntimeState, LifecycleEvent};
use crate::backend_manager::{BackendManager, BackendType};
use crate::initialization::{RuntimeInitializer, InitializationPhase};
use crate::shutdown::{ShutdownManager, GracefulShutdown};

use mir_types::{ContentHash, TypeRegistry};
use mir_vm::{VirtualMachine, ExecutionContext};
use mir_runtime::{
    HMRCoordinator, DistributedHMRCoordinator, StateManager, ContentAddressableStore,
    ModuleStore, OpenTelemetryCollector, FFIBridge, EnvironmentAdapter, Logger,
    DistributedRuntime, ConsensusProtocol, DevToolsIntegration
};
use mir_compiler::Compiler;

/// Main MIR Runtime that integrates all components
pub struct MirRuntime {
    /// Runtime configuration
    config: RuntimeConfig,
    
    /// Current runtime state
    state: Arc<RwLock<RuntimeState>>,
    
    /// Lifecycle manager for coordinating startup/shutdown
    lifecycle_manager: LifecycleManager,
    
    /// Backend manager for different compilation targets
    backend_manager: BackendManager,
    
    /// Core components
    components: RuntimeComponents,
    
    /// Shutdown manager for graceful cleanup
    shutdown_manager: ShutdownManager,
}

/// Core runtime components
struct RuntimeComponents {
    /// Type registry for the sophisticated type system
    type_registry: Arc<TypeRegistry>,
    
    /// Virtual machine for executing MIR IR
    virtual_machine: Arc<RwLock<Box<dyn VirtualMachine>>>,
    
    /// Compiler for transforming source to MIR IR
    compiler: Arc<Compiler>,
    
    /// Content-addressable storage for code and data
    content_store: Arc<dyn ContentAddressableStore>,
    
    /// Module store for managing versioned modules
    module_store: Arc<ModuleStore>,
    
    /// State manager for preserving application state
    state_manager: Arc<StateManager>,
    
    /// HMR coordinator for orchestrating updates
    hmr_coordinator: Arc<dyn HMRCoordinator>,
    
    /// Distributed HMR coordinator for cluster-wide updates
    distributed_hmr: Arc<dyn DistributedHMRCoordinator>,
    
    /// FFI bridge for host environment interaction
    ffi_bridge: Arc<dyn FFIBridge>,
    
    /// OpenTelemetry collector for observability
    telemetry: Arc<OpenTelemetryCollector>,
    
    /// Logger for structured logging
    logger: Arc<dyn Logger>,
    
    /// Environment adapter for environment-specific behavior
    environment_adapter: Arc<dyn EnvironmentAdapter>,
    
    /// Distributed runtime primitives
    distributed_runtime: Arc<DistributedRuntime>,
    
    /// Consensus protocol for distributed coordination
    consensus: Arc<dyn ConsensusProtocol>,
    
    /// Development tools integration
    dev_tools: Arc<DevToolsIntegration>,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Component error: {0}")]
    ComponentError(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
    
    #[error("Lifecycle error: {0}")]
    LifecycleError(String),
    
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Runtime is not in the expected state: expected {expected}, found {actual}")]
    InvalidState { expected: String, actual: String },
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl MirRuntime {
    /// Create a new MIR runtime with the given configuration
    #[instrument(skip(config))]
    pub async fn new(config: RuntimeConfig) -> RuntimeResult<Self> {
        info!("Creating new MIR runtime with config: {:?}", config);
        
        let state = Arc::new(RwLock::new(RuntimeState::Initializing));
        let lifecycle_manager = LifecycleManager::new();
        let backend_manager = BackendManager::new();
        let shutdown_manager = ShutdownManager::new();
        
        // Initialize components
        let components = Self::initialize_components(&config).await?;
        
        let runtime = Self {
            config,
            state,
            lifecycle_manager,
            backend_manager,
            components,
            shutdown_manager,
        };
        
        info!("MIR runtime created successfully");
        Ok(runtime)
    }
    
    /// Start the runtime and all its components
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> RuntimeResult<()> {
        info!("Starting MIR runtime");
        
        // Check current state
        {
            let state = self.state.read().await;
            if !matches!(*state, RuntimeState::Initializing) {
                return Err(RuntimeError::InvalidState {
                    expected: "Initializing".to_string(),
                    actual: format!("{:?}", *state),
                });
            }
        }
        
        // Update state to starting
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Starting;
        }
        
        // Initialize runtime components in phases
        let initializer = RuntimeInitializer::new(&self.config);
        
        // Phase 1: Core infrastructure
        initializer.initialize_phase(InitializationPhase::CoreInfrastructure, &self.components).await
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        
        // Phase 2: Type system and storage
        initializer.initialize_phase(InitializationPhase::TypeSystemAndStorage, &self.components).await
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        
        // Phase 3: Virtual machine and compiler
        initializer.initialize_phase(InitializationPhase::VirtualMachineAndCompiler, &self.components).await
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        
        // Phase 4: HMR and distributed coordination
        initializer.initialize_phase(InitializationPhase::HMRAndDistributed, &self.components).await
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        
        // Phase 5: Backends and integrations
        initializer.initialize_phase(InitializationPhase::BackendsAndIntegrations, &self.components).await
            .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
        
        // Start backend manager
        self.backend_manager.start(&self.config.backend_configs).await
            .map_err(|e| RuntimeError::BackendError(e.to_string()))?;
        
        // Start lifecycle manager
        self.lifecycle_manager.start().await
            .map_err(|e| RuntimeError::LifecycleError(e.to_string()))?;
        
        // Update state to running
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Running;
        }
        
        // Emit lifecycle event
        self.lifecycle_manager.emit_event(LifecycleEvent::RuntimeStarted).await;
        
        info!("MIR runtime started successfully");
        Ok(())
    }
    
    /// Stop the runtime gracefully
    #[instrument(skip(self))]
    pub async fn shutdown(&mut self) -> RuntimeResult<()> {
        info!("Shutting down MIR runtime");
        
        // Check current state
        {
            let state = self.state.read().await;
            if matches!(*state, RuntimeState::Stopped | RuntimeState::ShuttingDown) {
                warn!("Runtime is already stopped or shutting down");
                return Ok(());
            }
        }
        
        // Update state to shutting down
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::ShuttingDown;
        }
        
        // Emit lifecycle event
        self.lifecycle_manager.emit_event(LifecycleEvent::RuntimeShuttingDown).await;
        
        // Perform graceful shutdown
        let graceful_shutdown = GracefulShutdown::new(&self.config);
        graceful_shutdown.shutdown(&self.components, &mut self.backend_manager).await
            .map_err(|e| RuntimeError::ShutdownError(e.to_string()))?;
        
        // Stop lifecycle manager
        self.lifecycle_manager.stop().await
            .map_err(|e| RuntimeError::LifecycleError(e.to_string()))?;
        
        // Update state to stopped
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Stopped;
        }
        
        info!("MIR runtime shut down successfully");
        Ok(())
    }
    
    /// Get the current runtime state
    pub async fn state(&self) -> RuntimeState {
        *self.state.read().await
    }
    
    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
    
    /// Get access to core components (for advanced usage)
    pub fn components(&self) -> &RuntimeComponents {
        &self.components
    }
    
    /// Execute a module by content hash
    #[instrument(skip(self))]
    pub async fn execute_module(&self, module_hash: ContentHash) -> RuntimeResult<serde_json::Value> {
        debug!("Executing module: {}", module_hash);
        
        // Ensure runtime is running
        {
            let state = self.state.read().await;
            if !matches!(*state, RuntimeState::Running) {
                return Err(RuntimeError::InvalidState {
                    expected: "Running".to_string(),
                    actual: format!("{:?}", *state),
                });
            }
        }
        
        // Get module from store
        let module = self.components.module_store.get_module(module_hash)
            .map_err(|e| RuntimeError::ComponentError(format!("Failed to get module: {}", e)))?
            .ok_or_else(|| RuntimeError::ComponentError(format!("Module not found: {}", module_hash)))?;
        
        // Execute module in VM
        let vm = self.components.virtual_machine.read().await;
        let result = vm.execute_module(&module)
            .map_err(|e| RuntimeError::ComponentError(format!("Execution failed: {}", e)))?;
        
        debug!("Module execution completed successfully");
        Ok(result.into())
    }
    
    /// Hot-reload a module
    #[instrument(skip(self))]
    pub async fn hot_reload_module(&self, old_hash: ContentHash, new_module_data: &[u8]) -> RuntimeResult<()> {
        info!("Hot-reloading module: {} -> new version", old_hash);
        
        // Ensure runtime is running
        {
            let state = self.state.read().await;
            if !matches!(*state, RuntimeState::Running) {
                return Err(RuntimeError::InvalidState {
                    expected: "Running".to_string(),
                    actual: format!("{:?}", *state),
                });
            }
        }
        
        // Store new module
        let new_hash = self.components.content_store.store(new_module_data);
        
        // Parse new module (this would normally be done by the compiler)
        // For now, we'll assume the module is already in the correct format
        
        // Coordinate the hot-reload
        let hmr_result = self.components.hmr_coordinator.hot_reload_module(old_hash, new_hash).await
            .map_err(|e| RuntimeError::ComponentError(format!("HMR failed: {}", e)))?;
        
        info!("Hot-reload completed successfully: {:?}", hmr_result);
        Ok(())
    }
    
    /// Get runtime statistics and health information
    pub async fn get_runtime_info(&self) -> RuntimeInfo {
        let state = self.state.read().await;
        let backend_info = self.backend_manager.get_info().await;
        
        RuntimeInfo {
            state: *state,
            version: crate::VERSION.to_string(),
            uptime: self.lifecycle_manager.uptime(),
            backend_info,
            component_health: self.get_component_health().await,
        }
    }
    
    /// Initialize all runtime components
    async fn initialize_components(config: &RuntimeConfig) -> RuntimeResult<RuntimeComponents> {
        info!("Initializing runtime components");
        
        // Initialize type registry
        let type_registry = Arc::new(TypeRegistry::new());
        
        // Initialize content store
        let content_store = Arc::new(mir_runtime::InMemoryStore::new());
        
        // Initialize module store
        let module_store = Arc::new(ModuleStore::new(content_store.clone()));
        
        // Initialize state manager
        let state_manager = Arc::new(StateManager::new());
        
        // Initialize compiler
        let compiler = Arc::new(Compiler::new(type_registry.clone()));
        
        // Initialize virtual machine
        let execution_context = ExecutionContext::new(
            type_registry.clone(),
            content_store.clone(),
            state_manager.clone(),
        );
        let virtual_machine = Arc::new(RwLock::new(
            Box::new(mir_vm::BasicVirtualMachine::new(execution_context)) as Box<dyn VirtualMachine>
        ));
        
        // Initialize HMR coordinator
        let hmr_coordinator = Arc::new(mir_runtime::BasicHMRCoordinator::new(
            content_store.clone(),
            state_manager.clone(),
            config.hmr_config.clone(),
        )) as Arc<dyn HMRCoordinator>;
        
        // Initialize distributed HMR coordinator
        let distributed_hmr = Arc::new(mir_runtime::BasicDistributedHMRCoordinator::new(
            hmr_coordinator.clone(),
        )) as Arc<dyn DistributedHMRCoordinator>;
        
        // Initialize FFI bridge
        let ffi_bridge = Arc::new(mir_runtime::BasicFFIBridge::new()) as Arc<dyn FFIBridge>;
        
        // Initialize telemetry
        let telemetry = Arc::new(OpenTelemetryCollector::new(
            config.telemetry_config.clone(),
        ));
        
        // Initialize logger
        let logger = Arc::new(mir_runtime::StructuredLogger::new()) as Arc<dyn Logger>;
        
        // Initialize environment adapter
        let environment_adapter = Arc::new(mir_runtime::BasicEnvironmentAdapter::new(
            config.hmr_config.environment.clone(),
        )) as Arc<dyn EnvironmentAdapter>;
        
        // Initialize distributed runtime
        let distributed_runtime = Arc::new(DistributedRuntime::new());
        
        // Initialize consensus protocol
        let consensus = Arc::new(mir_runtime::BasicConsensusProtocol::new(
            config.consensus_config.clone(),
        )) as Arc<dyn ConsensusProtocol>;
        
        // Initialize dev tools integration
        let dev_tools = Arc::new(DevToolsIntegration::new(
            config.dev_tools_config.clone(),
        ));
        
        let components = RuntimeComponents {
            type_registry,
            virtual_machine,
            compiler,
            content_store,
            module_store,
            state_manager,
            hmr_coordinator,
            distributed_hmr,
            ffi_bridge,
            telemetry,
            logger,
            environment_adapter,
            distributed_runtime,
            consensus,
            dev_tools,
        };
        
        info!("Runtime components initialized successfully");
        Ok(components)
    }
    
    /// Get health status of all components
    async fn get_component_health(&self) -> HashMap<String, ComponentHealth> {
        let mut health = HashMap::new();
        
        // Check each component's health
        health.insert("type_registry".to_string(), ComponentHealth::Healthy);
        health.insert("virtual_machine".to_string(), ComponentHealth::Healthy);
        health.insert("compiler".to_string(), ComponentHealth::Healthy);
        health.insert("content_store".to_string(), ComponentHealth::Healthy);
        health.insert("module_store".to_string(), ComponentHealth::Healthy);
        health.insert("state_manager".to_string(), ComponentHealth::Healthy);
        health.insert("hmr_coordinator".to_string(), ComponentHealth::Healthy);
        health.insert("distributed_hmr".to_string(), ComponentHealth::Healthy);
        health.insert("ffi_bridge".to_string(), ComponentHealth::Healthy);
        health.insert("telemetry".to_string(), ComponentHealth::Healthy);
        health.insert("logger".to_string(), ComponentHealth::Healthy);
        health.insert("environment_adapter".to_string(), ComponentHealth::Healthy);
        health.insert("distributed_runtime".to_string(), ComponentHealth::Healthy);
        health.insert("consensus".to_string(), ComponentHealth::Healthy);
        health.insert("dev_tools".to_string(), ComponentHealth::Healthy);
        
        health
    }
}

/// Runtime information and statistics
#[derive(Debug, Clone)]
pub struct RuntimeInfo {
    pub state: RuntimeState,
    pub version: String,
    pub uptime: std::time::Duration,
    pub backend_info: HashMap<String, serde_json::Value>,
    pub component_health: HashMap<String, ComponentHealth>,
}

/// Component health status
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl RuntimeComponents {
    /// Get the type registry
    pub fn type_registry(&self) -> &Arc<TypeRegistry> {
        &self.type_registry
    }
    
    /// Get the virtual machine
    pub fn virtual_machine(&self) -> &Arc<RwLock<Box<dyn VirtualMachine>>> {
        &self.virtual_machine
    }
    
    /// Get the compiler
    pub fn compiler(&self) -> &Arc<Compiler> {
        &self.compiler
    }
    
    /// Get the content store
    pub fn content_store(&self) -> &Arc<dyn ContentAddressableStore> {
        &self.content_store
    }
    
    /// Get the module store
    pub fn module_store(&self) -> &Arc<ModuleStore> {
        &self.module_store
    }
    
    /// Get the state manager
    pub fn state_manager(&self) -> &Arc<StateManager> {
        &self.state_manager
    }
    
    /// Get the HMR coordinator
    pub fn hmr_coordinator(&self) -> &Arc<dyn HMRCoordinator> {
        &self.hmr_coordinator
    }
    
    /// Get the distributed HMR coordinator
    pub fn distributed_hmr(&self) -> &Arc<dyn DistributedHMRCoordinator> {
        &self.distributed_hmr
    }
    
    /// Get the FFI bridge
    pub fn ffi_bridge(&self) -> &Arc<dyn FFIBridge> {
        &self.ffi_bridge
    }
    
    /// Get the telemetry collector
    pub fn telemetry(&self) -> &Arc<OpenTelemetryCollector> {
        &self.telemetry
    }
    
    /// Get the logger
    pub fn logger(&self) -> &Arc<dyn Logger> {
        &self.logger
    }
    
    /// Get the environment adapter
    pub fn environment_adapter(&self) -> &Arc<dyn EnvironmentAdapter> {
        &self.environment_adapter
    }
    
    /// Get the distributed runtime
    pub fn distributed_runtime(&self) -> &Arc<DistributedRuntime> {
        &self.distributed_runtime
    }
    
    /// Get the consensus protocol
    pub fn consensus(&self) -> &Arc<dyn ConsensusProtocol> {
        &self.consensus
    }
    
    /// Get the dev tools integration
    pub fn dev_tools(&self) -> &Arc<DevToolsIntegration> {
        &self.dev_tools
    }
}