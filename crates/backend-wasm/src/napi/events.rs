//! Event system for WASM instance lifecycle management

use std::sync::{Arc, Weak};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Events that can occur during instance lifecycle
#[derive(Debug, Clone)]
pub enum InstanceEvent {
    Created { instance_id: String, memory_usage: u32 },
    FunctionExecuted { instance_id: String, function_name: String, duration_ms: f64 },
    HotReloaded { instance_id: String, success: bool },
    MemoryAllocated { instance_id: String, size: u32 },
    Error { instance_id: String, error: String },
    Destroyed { instance_id: String },
}

/// Observer trait for instance events
pub trait InstanceEventObserver: Send + Sync {
    fn on_event(&self, event: &InstanceEvent);
}

/// Event publisher for instance events
#[derive(Default)]
pub struct InstanceEventPublisher {
    observers: RwLock<HashMap<String, Weak<dyn InstanceEventObserver>>>,
}

impl InstanceEventPublisher {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Subscribe to instance events
    pub fn subscribe(&self, id: String, observer: Arc<dyn InstanceEventObserver>) {
        self.observers.write().insert(id, Arc::downgrade(&observer));
    }
    
    /// Unsubscribe from instance events
    pub fn unsubscribe(&self, id: &str) {
        self.observers.write().remove(id);
    }
    
    /// Publish an event to all observers
    pub fn publish(&self, event: InstanceEvent) {
        let observers = self.observers.read();
        let mut to_remove = Vec::new();
        
        for (id, weak_observer) in observers.iter() {
            if let Some(observer) = weak_observer.upgrade() {
                observer.on_event(&event);
            } else {
                // Observer has been dropped, mark for removal
                to_remove.push(id.clone());
            }
        }
        
        // Clean up dropped observers
        drop(observers);
        if !to_remove.is_empty() {
            let mut observers = self.observers.write();
            for id in to_remove {
                observers.remove(&id);
            }
        }
    }
}

/// Metrics collector that observes instance events
pub struct MetricsCollector {
    instance_count: Arc<RwLock<u32>>,
    total_executions: Arc<RwLock<u64>>,
    total_errors: Arc<RwLock<u64>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            instance_count: Arc::new(RwLock::new(0)),
            total_executions: Arc::new(RwLock::new(0)),
            total_errors: Arc::new(RwLock::new(0)),
        }
    }
    
    pub fn get_metrics(&self) -> (u32, u64, u64) {
        (
            *self.instance_count.read(),
            *self.total_executions.read(),
            *self.total_errors.read(),
        )
    }
}

impl InstanceEventObserver for MetricsCollector {
    fn on_event(&self, event: &InstanceEvent) {
        match event {
            InstanceEvent::Created { .. } => {
                *self.instance_count.write() += 1;
            }
            InstanceEvent::FunctionExecuted { .. } => {
                *self.total_executions.write() += 1;
            }
            InstanceEvent::Error { .. } => {
                *self.total_errors.write() += 1;
            }
            InstanceEvent::Destroyed { .. } => {
                *self.instance_count.write() = self.instance_count.read().saturating_sub(1);
            }
            _ => {}
        }
    }
}