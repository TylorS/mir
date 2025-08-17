//! MIR Runtime - Complete Distributed Hot-Module-Reloading Runtime
//!
//! This is the main runtime assembly that integrates all MIR components into a unified
//! runtime system. It provides:
//!
//! - Runtime initialization and configuration
//! - Component lifecycle management
//! - Graceful shutdown and cleanup
//! - Backend selection and management
//! - Distributed coordination
//! - Hot-module-reloading orchestration
//!
//! # Example Usage
//!
//! ```rust
//! use mir_runtime::{MirRuntime, RuntimeConfig, Environment};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = RuntimeConfig::new()
//!         .with_environment(Environment::Development { 
//!             auto_reload: true,
//!             file_watcher_enabled: true,
//!             aggressive_optimization: false,
//!         })
//!         .with_backend("wasm");
//!
//!     let mut runtime = MirRuntime::new(config).await?;
//!     runtime.start().await?;
//!
//!     // Runtime is now ready to execute modules and handle hot-reloading
//!     
//!     runtime.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod runtime;
pub mod config;
pub mod lifecycle;
pub mod backend_manager;
pub mod initialization;
pub mod shutdown;

// Re-export core types and traits
pub use mir_ast::*;
pub use mir_types::*;
pub use mir_vm::*;
pub use mir_runtime::*;
pub use mir_compiler::*;

// Re-export main runtime components
pub use runtime::{MirRuntime, RuntimeError, RuntimeResult};
pub use config::{RuntimeConfig, RuntimeConfigBuilder, BackendConfig};
pub use lifecycle::{RuntimeState, LifecycleManager, LifecycleEvent};
pub use backend_manager::{BackendManager, BackendType, BackendInstance};
pub use initialization::{RuntimeInitializer, InitializationPhase, InitializationError};
pub use shutdown::{ShutdownManager, ShutdownPhase, ShutdownError, GracefulShutdown};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");