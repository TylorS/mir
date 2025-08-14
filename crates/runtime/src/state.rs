//! State management for hot-module-reloading

use mir_types::{ContentHash, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State snapshot for preserving application state during updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub module_hash: ContentHash,
    pub state_data: HashMap<String, Value>,
    pub timestamp: u64,
}

/// State manager for handling state preservation and restoration
pub struct StateManager {
    snapshots: HashMap<ContentHash, StateSnapshot>,
}

impl StateManager {
    pub fn new() -> Self {
        StateManager {
            snapshots: HashMap::new(),
        }
    }
    
    pub fn create_snapshot(&mut self, module_hash: ContentHash, state: HashMap<String, Value>) -> StateSnapshot {
        let snapshot = StateSnapshot {
            module_hash,
            state_data: state,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.snapshots.insert(module_hash, snapshot.clone());
        snapshot
    }
    
    pub fn get_snapshot(&self, module_hash: ContentHash) -> Option<&StateSnapshot> {
        self.snapshots.get(&module_hash)
    }
}