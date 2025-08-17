//! Runtime lifecycle management
//!
//! This module manages the lifecycle of the MIR runtime, including state transitions,
//! event handling, and coordination between different phases of runtime operation.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error, debug, instrument};
use thiserror::Error;

/// Runtime lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Runtime is being initialized
    Initializing,
    
    /// Runtime is starting up
    Starting,
    
    /// Runtime is fully operational
    Running,
    
    /// Runtime is shutting down gracefully
    ShuttingDown,
    
    /// Runtime has stopped
    Stopped,
    
    /// Runtime encountered an error
    Error,
}

/// Lifecycle events that can occur during runtime operation
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// Runtime has started successfully
    RuntimeStarted,
    
    /// Runtime is shutting down
    RuntimeShuttingDown,
    
    /// Runtime has stopped
    RuntimeStopped,
    
    /// A component has started
    ComponentStarted { component: String },
    
    /// A component has stopped
    ComponentStopped { component: String },
    
    /// A component encountered an error
    ComponentError { component: String, error: String },
    
    /// State transition occurred
    StateTransition { from: RuntimeState, to: RuntimeState },
    
    /// Custom event
    Custom { event_type: String, data: serde_json::Value },
}

/// Lifecycle manager for coordinating runtime state and events
pub struct LifecycleManager {
    /// Current runtime state
    state: Arc<RwLock<RuntimeState>>,
    
    /// Event broadcaster
    event_sender: broadcast::Sender<LifecycleEvent>,
    
    /// Event receiver for internal use
    _event_receiver: broadcast::Receiver<LifecycleEvent>,
    
    /// Runtime start time
    start_time: Option<Instant>,
    
    /// Event handlers
    event_handlers: Arc<RwLock<Vec<Box<dyn LifecycleEventHandler>>>>,
}

/// Trait for handling lifecycle events
pub trait LifecycleEventHandler: Send + Sync {
    /// Handle a lifecycle event
    fn handle_event(&self, event: &LifecycleEvent);
    
    /// Get the handler name
    fn name(&self) -> &str;
}

#[derive(Error, Debug)]
pub enum LifecycleError {
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: RuntimeState, to: RuntimeState },
    
    #[error("Event handling error: {0}")]
    EventHandlingError(String),
    
    #[error("Lifecycle manager not started")]
    NotStarted,
    
    #[error("Lifecycle manager already started")]
    AlreadyStarted,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        
        Self {
            state: Arc::new(RwLock::new(RuntimeState::Initializing)),
            event_sender,
            _event_receiver: event_receiver,
            start_time: None,
            event_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start the lifecycle manager
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<(), LifecycleError> {
        if self.start_time.is_some() {
            return Err(LifecycleError::AlreadyStarted);
        }
        
        self.start_time = Some(Instant::now());
        info!("Lifecycle manager started");
        Ok(())
    }
    
    /// Stop the lifecycle manager
    #[instrument(skip(self))]
    pub async fn stop(&mut self) -> Result<(), LifecycleError> {
        if self.start_time.is_none() {
            return Err(LifecycleError::NotStarted);
        }
        
        // Transition to stopped state
        self.transition_state(RuntimeState::Stopped).await?;
        
        // Emit stopped event
        self.emit_event(LifecycleEvent::RuntimeStopped).await;
        
        info!("Lifecycle manager stopped");
        Ok(())
    }
    
    /// Get the current runtime state
    pub async fn current_state(&self) -> RuntimeState {
        *self.state.read().await
    }
    
    /// Transition to a new state
    #[instrument(skip(self))]
    pub async fn transition_state(&self, new_state: RuntimeState) -> Result<(), LifecycleError> {
        let mut state = self.state.write().await;
        let old_state = *state;
        
        // Validate state transition
        if !self.is_valid_transition(old_state, new_state) {
            return Err(LifecycleError::InvalidStateTransition {
                from: old_state,
                to: new_state,
            });
        }
        
        *state = new_state;
        drop(state);
        
        // Emit state transition event
        self.emit_event(LifecycleEvent::StateTransition {
            from: old_state,
            to: new_state,
        }).await;
        
        info!("State transition: {:?} -> {:?}", old_state, new_state);
        Ok(())
    }
    
    /// Emit a lifecycle event
    #[instrument(skip(self, event))]
    pub async fn emit_event(&self, event: LifecycleEvent) {
        debug!("Emitting lifecycle event: {:?}", event);
        
        // Send event to broadcast channel
        if let Err(e) = self.event_sender.send(event.clone()) {
            warn!("Failed to broadcast lifecycle event: {}", e);
        }
        
        // Handle event with registered handlers
        let handlers = self.event_handlers.read().await;
        for handler in handlers.iter() {
            handler.handle_event(&event);
        }
    }
    
    /// Subscribe to lifecycle events
    pub fn subscribe(&self) -> broadcast::Receiver<LifecycleEvent> {
        self.event_sender.subscribe()
    }
    
    /// Register an event handler
    pub async fn register_handler(&self, handler: Box<dyn LifecycleEventHandler>) {
        let mut handlers = self.event_handlers.write().await;
        info!("Registering lifecycle event handler: {}", handler.name());
        handlers.push(handler);
    }
    
    /// Get runtime uptime
    pub fn uptime(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default()
    }
    
    /// Check if a state transition is valid
    fn is_valid_transition(&self, from: RuntimeState, to: RuntimeState) -> bool {
        use RuntimeState::*;
        
        match (from, to) {
            // From Initializing
            (Initializing, Starting) => true,
            (Initializing, Error) => true,
            
            // From Starting
            (Starting, Running) => true,
            (Starting, Error) => true,
            (Starting, ShuttingDown) => true,
            
            // From Running
            (Running, ShuttingDown) => true,
            (Running, Error) => true,
            
            // From ShuttingDown
            (ShuttingDown, Stopped) => true,
            (ShuttingDown, Error) => true,
            
            // From Error
            (Error, ShuttingDown) => true,
            (Error, Stopped) => true,
            
            // Same state (no-op)
            (state1, state2) if state1 == state2 => true,
            
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default lifecycle event handler that logs events
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl LifecycleEventHandler for LoggingEventHandler {
    fn handle_event(&self, event: &LifecycleEvent) {
        match event {
            LifecycleEvent::RuntimeStarted => {
                info!("Runtime started successfully");
            }
            LifecycleEvent::RuntimeShuttingDown => {
                info!("Runtime is shutting down");
            }
            LifecycleEvent::RuntimeStopped => {
                info!("Runtime has stopped");
            }
            LifecycleEvent::ComponentStarted { component } => {
                info!("Component '{}' started", component);
            }
            LifecycleEvent::ComponentStopped { component } => {
                info!("Component '{}' stopped", component);
            }
            LifecycleEvent::ComponentError { component, error } => {
                error!("Component '{}' error: {}", component, error);
            }
            LifecycleEvent::StateTransition { from, to } => {
                info!("State transition: {:?} -> {:?}", from, to);
            }
            LifecycleEvent::Custom { event_type, data } => {
                debug!("Custom event '{}': {:?}", event_type, data);
            }
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Metrics event handler that tracks runtime metrics
pub struct MetricsEventHandler {
    name: String,
    // In a real implementation, this would contain metrics collectors
}

impl MetricsEventHandler {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl LifecycleEventHandler for MetricsEventHandler {
    fn handle_event(&self, event: &LifecycleEvent) {
        // In a real implementation, this would update metrics based on events
        match event {
            LifecycleEvent::RuntimeStarted => {
                // Increment runtime_starts_total counter
            }
            LifecycleEvent::ComponentError { component: _, error: _ } => {
                // Increment component_errors_total counter
            }
            LifecycleEvent::StateTransition { from: _, to } => {
                // Update runtime_state gauge
                debug!("Would update runtime_state metric to: {:?}", to);
            }
            _ => {}
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for RuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeState::Initializing => write!(f, "Initializing"),
            RuntimeState::Starting => write!(f, "Starting"),
            RuntimeState::Running => write!(f, "Running"),
            RuntimeState::ShuttingDown => write!(f, "ShuttingDown"),
            RuntimeState::Stopped => write!(f, "Stopped"),
            RuntimeState::Error => write!(f, "Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let manager = LifecycleManager::new();
        assert_eq!(manager.current_state().await, RuntimeState::Initializing);
        assert_eq!(manager.uptime(), Duration::ZERO);
    }
    
    #[tokio::test]
    async fn test_state_transitions() {
        let manager = LifecycleManager::new();
        
        // Valid transitions
        assert!(manager.transition_state(RuntimeState::Starting).await.is_ok());
        assert_eq!(manager.current_state().await, RuntimeState::Starting);
        
        assert!(manager.transition_state(RuntimeState::Running).await.is_ok());
        assert_eq!(manager.current_state().await, RuntimeState::Running);
        
        assert!(manager.transition_state(RuntimeState::ShuttingDown).await.is_ok());
        assert_eq!(manager.current_state().await, RuntimeState::ShuttingDown);
        
        assert!(manager.transition_state(RuntimeState::Stopped).await.is_ok());
        assert_eq!(manager.current_state().await, RuntimeState::Stopped);
    }
    
    #[tokio::test]
    async fn test_invalid_state_transitions() {
        let manager = LifecycleManager::new();
        
        // Invalid transition: Initializing -> Running (must go through Starting)
        assert!(manager.transition_state(RuntimeState::Running).await.is_err());
        
        // Invalid transition: Stopped -> Running
        manager.transition_state(RuntimeState::Starting).await.unwrap();
        manager.transition_state(RuntimeState::Running).await.unwrap();
        manager.transition_state(RuntimeState::ShuttingDown).await.unwrap();
        manager.transition_state(RuntimeState::Stopped).await.unwrap();
        
        assert!(manager.transition_state(RuntimeState::Running).await.is_err());
    }
    
    #[tokio::test]
    async fn test_event_handling() {
        let manager = LifecycleManager::new();
        let mut receiver = manager.subscribe();
        
        // Emit an event
        manager.emit_event(LifecycleEvent::RuntimeStarted).await;
        
        // Receive the event
        let event = receiver.recv().await.unwrap();
        assert!(matches!(event, LifecycleEvent::RuntimeStarted));
    }
    
    #[tokio::test]
    async fn test_uptime_tracking() {
        let mut manager = LifecycleManager::new();
        
        // Initially no uptime
        assert_eq!(manager.uptime(), Duration::ZERO);
        
        // Start the manager
        manager.start().await.unwrap();
        
        // Wait a bit
        sleep(Duration::from_millis(10)).await;
        
        // Should have some uptime now
        assert!(manager.uptime() > Duration::ZERO);
    }
}