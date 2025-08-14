//! Runtime configuration

use serde::{Deserialize, Serialize};

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub environment: Environment,
    pub hmr_enabled: bool,
    pub telemetry_enabled: bool,
    pub storage_backend: StorageBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Production,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    InMemory,
    FileSystem { path: String },
    Distributed { nodes: Vec<String> },
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            environment: Environment::Development,
            hmr_enabled: true,
            telemetry_enabled: true,
            storage_backend: StorageBackend::InMemory,
        }
    }
}