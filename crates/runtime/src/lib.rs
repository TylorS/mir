//! MIR Runtime - Core runtime services and coordination
//! 
//! This crate provides the runtime services for MIR, including:
//! - Content-addressable storage
//! - State management
//! - Distributed coordination primitives
//! - OpenTelemetry integration

pub mod storage;
pub mod state;
pub mod distributed;
pub mod telemetry;
pub mod config;

pub use storage::ContentAddressableStore;
pub use state::StateManager;
pub use distributed::DistributedRuntime;
pub use telemetry::OpenTelemetryCollector;
pub use config::RuntimeConfig;