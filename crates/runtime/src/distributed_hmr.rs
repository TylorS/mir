//! Distributed HMR coordination
//! 
//! This module implements distributed hot-module-reloading coordination,
//! including cluster-wide updates, rolling updates, and rollback mechanisms.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant};
use serde::{Deserialize, Serialize};
use crate::crdt::NodeId;
use crate::consensus::{ConsensusProtocol, ProposalType, ProposalId, ConsensusError};
use crate::hmr_coordinator::UpdatePlan;
use mir_types::ContentHash;

/// Result type for distributed HMR operations
pub type DistributedHMRResult<T> = Result<T, DistributedHMRError>;

/// Errors that can occur during distributed HMR operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedHMRError {
    ConsensusError(String),
    NodeNotFound(NodeId),
    UpdateNotFound(UpdateId),
    InsufficientNodes(String),
    UpdateFailed(String),
    RollbackFailed(String),
    PolicyViolation(String),
    NetworkError(String),
    TimeoutError(String),
    InvalidState(String),
    SerializationError(String),
}

impl std::fmt::Display for DistributedHMRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributedHMRError::ConsensusError(msg) => write!(f, "Consensus error: {msg}"),
            DistributedHMRError::NodeNotFound(node_id) => write!(f, "Node not found: {node_id:?}"),
            DistributedHMRError::UpdateNotFound(update_id) => write!(f, "Update not found: {update_id:?}"),
            DistributedHMRError::InsufficientNodes(msg) => write!(f, "Insufficient nodes: {msg}"),
            DistributedHMRError::UpdateFailed(msg) => write!(f, "Update failed: {msg}"),
            DistributedHMRError::RollbackFailed(msg) => write!(f, "Rollback failed: {msg}"),
            DistributedHMRError::PolicyViolation(msg) => write!(f, "Policy violation: {msg}"),
            DistributedHMRError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            DistributedHMRError::TimeoutError(msg) => write!(f, "Timeout error: {msg}"),
            DistributedHMRError::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            DistributedHMRError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl std::error::Error for DistributedHMRError {}

impl From<ConsensusError> for DistributedHMRError {
    fn from(error: ConsensusError) -> Self {
        DistributedHMRError::ConsensusError(error.to_string())
    }
}

/// Unique identifier for distributed updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UpdateId(pub u64);

impl UpdateId {
    pub fn new(id: u64) -> Self {
        UpdateId(id)
    }
}

/// Status of a distributed update
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedUpdateStatus {
    Proposed,
    Approved,
    InProgress,
    RollingOut,
    Completed,
    Failed(String),
    RolledBack,
    Cancelled,
}

/// Strategy for rolling updates across the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum RollingUpdateStrategy {
    #[default]
    Sequential,                    // Update nodes one by one
    Parallel(usize),              // Update N nodes at a time
    Percentage(f32),              // Update X% of nodes at a time
    Canary { canary_nodes: Vec<NodeId>, percentage: f32 }, // Canary deployment
}


/// Policy for distributed updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub require_consensus: bool,
    pub min_healthy_nodes: usize,
    pub max_concurrent_updates: usize,
    pub rolling_strategy: RollingUpdateStrategy,
    pub timeout: Duration,
    pub auto_rollback_on_failure: bool,
    pub health_check_interval: Duration,
    pub max_rollback_attempts: u32,
}

impl Default for UpdatePolicy {
    fn default() -> Self {
        UpdatePolicy {
            require_consensus: true,
            min_healthy_nodes: 1,
            max_concurrent_updates: 1,
            rolling_strategy: RollingUpdateStrategy::Sequential,
            timeout: Duration::from_secs(300), // 5 minutes
            auto_rollback_on_failure: true,
            health_check_interval: Duration::from_secs(10),
            max_rollback_attempts: 3,
        }
    }
}

/// Information about a node's update status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeUpdateStatus {
    pub node_id: NodeId,
    pub status: NodeUpdateState,
    pub last_updated: SystemTime,
    pub error_message: Option<String>,
    pub rollback_count: u32,
}

/// State of a node during update process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeUpdateState {
    Pending,
    Updating,
    Updated,
    Failed,
    RolledBack,
    Healthy,
    Unhealthy,
}

/// Result of a distributed update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedUpdateResult {
    pub update_id: UpdateId,
    pub status: DistributedUpdateStatus,
    pub node_results: HashMap<NodeId, NodeUpdateStatus>,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub total_duration: Option<Duration>,
    pub success_count: usize,
    pub failure_count: usize,
    pub rollback_count: usize,
}

impl DistributedUpdateResult {
    pub fn new(update_id: UpdateId) -> Self {
        DistributedUpdateResult {
            update_id,
            status: DistributedUpdateStatus::Proposed,
            node_results: HashMap::new(),
            started_at: SystemTime::now(),
            completed_at: None,
            total_duration: None,
            success_count: 0,
            failure_count: 0,
            rollback_count: 0,
        }
    }
    
    pub fn is_complete(&self) -> bool {
        matches!(self.status, 
            DistributedUpdateStatus::Completed | 
            DistributedUpdateStatus::Failed(_) | 
            DistributedUpdateStatus::RolledBack |
            DistributedUpdateStatus::Cancelled
        )
    }
    
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            self.success_count as f32 / total as f32
        }
    }
    
    pub fn update_node_status(&mut self, node_id: NodeId, status: NodeUpdateState, error: Option<String>) {
        let node_status = self.node_results.entry(node_id).or_insert_with(|| {
            NodeUpdateStatus {
                node_id,
                status: NodeUpdateState::Pending,
                last_updated: SystemTime::now(),
                error_message: None,
                rollback_count: 0,
            }
        });
        
        // Update counters based on status change
        match (&node_status.status, &status) {
            (NodeUpdateState::Failed, NodeUpdateState::Updated) => {
                self.failure_count -= 1;
                self.success_count += 1;
            }
            (NodeUpdateState::Updated, NodeUpdateState::Failed) => {
                self.success_count -= 1;
                self.failure_count += 1;
            }
            (NodeUpdateState::Pending, NodeUpdateState::Updated) => {
                self.success_count += 1;
            }
            (NodeUpdateState::Pending, NodeUpdateState::Failed) => {
                self.failure_count += 1;
            }
            (_, NodeUpdateState::RolledBack) => {
                self.rollback_count += 1;
            }
            _ => {}
        }
        
        let is_rollback = matches!(status, NodeUpdateState::RolledBack);
        node_status.status = status;
        node_status.last_updated = SystemTime::now();
        node_status.error_message = error;
        
        if is_rollback {
            node_status.rollback_count += 1;
        }
    }
    
    pub fn finalize(&mut self, status: DistributedUpdateStatus) {
        self.status = status;
        self.completed_at = Some(SystemTime::now());
        self.total_duration = self.started_at.elapsed().ok();
    }
}

/// Information about a distributed update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedUpdate {
    pub id: UpdateId,
    pub module_hash: ContentHash,
    pub update_plan: UpdatePlan,
    pub target_nodes: Vec<NodeId>,
    pub policy: UpdatePolicy,
    pub created_at: SystemTime,
    pub created_by: NodeId,
    pub consensus_proposal_id: Option<ProposalId>,
}

/// Distributed HMR coordinator for cluster-wide updates
pub trait DistributedHMRCoordinator {
    /// Propose a distributed update
    fn propose_update(
        &mut self, 
        module_hash: ContentHash, 
        update_plan: UpdatePlan, 
        target_nodes: Vec<NodeId>
    ) -> DistributedHMRResult<UpdateId>;
    
    /// Execute a distributed update
    fn execute_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<DistributedUpdateResult>;
    
    /// Get the status of a distributed update
    fn get_update_status(&self, update_id: UpdateId) -> Option<&DistributedUpdateResult>;
    
    /// Cancel a distributed update
    fn cancel_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<()>;
    
    /// Perform cluster-wide rollback
    fn rollback_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<DistributedUpdateResult>;
    
    /// Set update policy for the cluster
    fn set_update_policy(&mut self, policy: UpdatePolicy) -> DistributedHMRResult<()>;
    
    /// Get current update policy
    fn get_update_policy(&self) -> &UpdatePolicy;
    
    /// Get all active updates
    fn get_active_updates(&self) -> Vec<UpdateId>;
    
    /// Check cluster health
    fn check_cluster_health(&self) -> ClusterHealthStatus;
    
    /// Force update on specific nodes (bypass consensus)
    fn force_update_nodes(&mut self, update_id: UpdateId, nodes: Vec<NodeId>) -> DistributedHMRResult<()>;
}

/// Health status of the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealthStatus {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub unhealthy_nodes: usize,
    pub updating_nodes: usize,
    pub failed_nodes: usize,
    pub last_check: SystemTime,
    pub overall_health: OverallHealth,
}

/// Overall health assessment of the cluster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallHealth {
    Healthy,
    Degraded,
    Critical,
    Unavailable,
}

impl ClusterHealthStatus {
    pub fn new(total_nodes: usize) -> Self {
        ClusterHealthStatus {
            total_nodes,
            healthy_nodes: 0,
            unhealthy_nodes: 0,
            updating_nodes: 0,
            failed_nodes: 0,
            last_check: SystemTime::now(),
            overall_health: OverallHealth::Unavailable,
        }
    }
    
    pub fn update_health(&mut self) {
        self.last_check = SystemTime::now();
        
        let healthy_percentage = if self.total_nodes > 0 {
            self.healthy_nodes as f32 / self.total_nodes as f32
        } else {
            0.0
        };
        
        self.overall_health = if healthy_percentage >= 0.8 {
            OverallHealth::Healthy
        } else if healthy_percentage >= 0.5 {
            OverallHealth::Degraded
        } else if healthy_percentage > 0.0 {
            OverallHealth::Critical
        } else {
            OverallHealth::Unavailable
        };
    }
}

/// Basic implementation of distributed HMR coordinator
#[derive(Debug)]
pub struct BasicDistributedHMRCoordinator<C: ConsensusProtocol> {
    node_id: NodeId,
    consensus: C,
    updates: HashMap<UpdateId, DistributedUpdate>,
    update_results: HashMap<UpdateId, DistributedUpdateResult>,
    next_update_id: u64,
    policy: UpdatePolicy,
    cluster_nodes: HashMap<NodeId, NodeUpdateStatus>,
    last_health_check: Instant,
}

impl<C: ConsensusProtocol> BasicDistributedHMRCoordinator<C> {
    pub fn new(node_id: NodeId, consensus: C, policy: UpdatePolicy) -> Self {
        BasicDistributedHMRCoordinator {
            node_id,
            consensus,
            updates: HashMap::new(),
            update_results: HashMap::new(),
            next_update_id: 0,
            policy,
            cluster_nodes: HashMap::new(),
            last_health_check: Instant::now(),
        }
    }
    
    fn next_update_id(&mut self) -> UpdateId {
        let id = UpdateId::new(self.next_update_id);
        self.next_update_id += 1;
        id
    }
    
    fn validate_update_policy(&self, target_nodes: &[NodeId]) -> DistributedHMRResult<()> {
        // Check minimum healthy nodes requirement
        let healthy_nodes = self.cluster_nodes.values()
            .filter(|status| matches!(status.status, NodeUpdateState::Healthy))
            .count();
        
        if healthy_nodes < self.policy.min_healthy_nodes {
            return Err(DistributedHMRError::PolicyViolation(
                format!("Insufficient healthy nodes: {} < {}", healthy_nodes, self.policy.min_healthy_nodes)
            ));
        }
        
        // Check concurrent updates limit
        let active_updates = self.update_results.values()
            .filter(|result| !result.is_complete())
            .count();
        
        if active_updates >= self.policy.max_concurrent_updates {
            return Err(DistributedHMRError::PolicyViolation(
                format!("Too many concurrent updates: {} >= {}", active_updates, self.policy.max_concurrent_updates)
            ));
        }
        
        // Validate target nodes exist
        for node_id in target_nodes {
            if !self.cluster_nodes.contains_key(node_id) {
                return Err(DistributedHMRError::NodeNotFound(*node_id));
            }
        }
        
        Ok(())
    }
    
    fn execute_rolling_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<()> {
        let update = self.updates.get(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?
            .clone();
        
        // Set status first
        {
            let result = self.update_results.get_mut(&update_id)
                .ok_or(DistributedHMRError::UpdateNotFound(update_id))?;
            result.status = DistributedUpdateStatus::RollingOut;
        }
        
        let strategy = update.policy.rolling_strategy.clone();
        match strategy {
            RollingUpdateStrategy::Sequential => {
                self.execute_sequential_update_impl(&update, update_id)
            }
            RollingUpdateStrategy::Parallel(batch_size) => {
                self.execute_parallel_update_impl(&update, update_id, batch_size)
            }
            RollingUpdateStrategy::Percentage(percentage) => {
                let batch_size = ((update.target_nodes.len() as f32 * percentage / 100.0).ceil() as usize).max(1);
                self.execute_parallel_update_impl(&update, update_id, batch_size)
            }
            RollingUpdateStrategy::Canary { canary_nodes, percentage } => {
                self.execute_canary_update_impl(&update, update_id, &canary_nodes, percentage)
            }
        }
    }
    
    fn execute_sequential_update_impl(&mut self, update: &DistributedUpdate, update_id: UpdateId) -> DistributedHMRResult<()> {
        for node_id in &update.target_nodes {
            {
                let result = self.update_results.get_mut(&update_id).unwrap();
                result.update_node_status(*node_id, NodeUpdateState::Updating, None);
            }
            
            // Simulate update execution (in real implementation, this would be network calls)
            let success = self.simulate_node_update(*node_id, &update.update_plan);
            
            let result = self.update_results.get_mut(&update_id).unwrap();
            if success {
                result.update_node_status(*node_id, NodeUpdateState::Updated, None);
            } else {
                let error_msg = format!("Update failed on node {node_id:?}");
                result.update_node_status(*node_id, NodeUpdateState::Failed, Some(error_msg.clone()));
                
                if update.policy.auto_rollback_on_failure {
                    return Err(DistributedHMRError::UpdateFailed(error_msg));
                }
            }
        }
        
        Ok(())
    }
    
    fn execute_parallel_update_impl(&mut self, update: &DistributedUpdate, update_id: UpdateId, batch_size: usize) -> DistributedHMRResult<()> {
        let batches: Vec<_> = update.target_nodes.chunks(batch_size).collect();
        
        for batch in batches {
            // Mark all nodes in batch as updating
            {
                let result = self.update_results.get_mut(&update_id).unwrap();
                for node_id in batch {
                    result.update_node_status(*node_id, NodeUpdateState::Updating, None);
                }
            }
            
            // Simulate parallel execution
            let mut batch_failures = Vec::new();
            for node_id in batch {
                let success = self.simulate_node_update(*node_id, &update.update_plan);
                
                let result = self.update_results.get_mut(&update_id).unwrap();
                if success {
                    result.update_node_status(*node_id, NodeUpdateState::Updated, None);
                } else {
                    let error_msg = format!("Update failed on node {node_id:?}");
                    result.update_node_status(*node_id, NodeUpdateState::Failed, Some(error_msg.clone()));
                    batch_failures.push((*node_id, error_msg));
                }
            }
            
            // Check if batch failed and auto-rollback is enabled
            if !batch_failures.is_empty() && update.policy.auto_rollback_on_failure {
                let error_msg = format!("Batch update failed on {} nodes", batch_failures.len());
                return Err(DistributedHMRError::UpdateFailed(error_msg));
            }
        }
        
        Ok(())
    }
    
    fn execute_canary_update_impl(&mut self, update: &DistributedUpdate, update_id: UpdateId, canary_nodes: &[NodeId], percentage: f32) -> DistributedHMRResult<()> {
        // First, update canary nodes
        for node_id in canary_nodes {
            if update.target_nodes.contains(node_id) {
                {
                    let result = self.update_results.get_mut(&update_id).unwrap();
                    result.update_node_status(*node_id, NodeUpdateState::Updating, None);
                }
                
                let success = self.simulate_node_update(*node_id, &update.update_plan);
                
                let result = self.update_results.get_mut(&update_id).unwrap();
                if success {
                    result.update_node_status(*node_id, NodeUpdateState::Updated, None);
                } else {
                    let error_msg = format!("Canary update failed on node {node_id:?}");
                    result.update_node_status(*node_id, NodeUpdateState::Failed, Some(error_msg.clone()));
                    return Err(DistributedHMRError::UpdateFailed(error_msg));
                }
            }
        }
        
        // If canary succeeded, update remaining nodes
        let remaining_nodes: Vec<_> = update.target_nodes.iter()
            .filter(|node_id| !canary_nodes.contains(node_id))
            .copied()
            .collect();
        
        let batch_size = ((remaining_nodes.len() as f32 * percentage / 100.0).ceil() as usize).max(1);
        
        let batches: Vec<_> = remaining_nodes.chunks(batch_size).collect();
        
        for batch in batches {
            {
                let result = self.update_results.get_mut(&update_id).unwrap();
                for node_id in batch {
                    result.update_node_status(*node_id, NodeUpdateState::Updating, None);
                }
            }
            
            for node_id in batch {
                let success = self.simulate_node_update(*node_id, &update.update_plan);
                
                let result = self.update_results.get_mut(&update_id).unwrap();
                if success {
                    result.update_node_status(*node_id, NodeUpdateState::Updated, None);
                } else {
                    let error_msg = format!("Update failed on node {node_id:?}");
                    result.update_node_status(*node_id, NodeUpdateState::Failed, Some(error_msg.clone()));
                    
                    if update.policy.auto_rollback_on_failure {
                        return Err(DistributedHMRError::UpdateFailed(error_msg));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn simulate_node_update(&self, _node_id: NodeId, _update_plan: &UpdatePlan) -> bool {
        // In a real implementation, this would:
        // 1. Send update request to the node
        // 2. Wait for confirmation
        // 3. Verify the update was successful
        // 4. Check node health after update
        
        // For simulation, we'll just return success most of the time
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        _node_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        // 90% success rate for simulation
        (hash % 10) != 0
    }
    
    fn execute_rollback(&mut self, update_id: UpdateId) -> DistributedHMRResult<DistributedUpdateResult> {
        let _update = self.updates.get(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?
            .clone();
        
        // Find nodes that were successfully updated and need rollback
        let nodes_to_rollback: Vec<NodeId> = {
            let result = self.update_results.get(&update_id)
                .ok_or(DistributedHMRError::UpdateNotFound(update_id))?;
            
            result.node_results.iter()
                .filter(|(_, status)| matches!(status.status, NodeUpdateState::Updated))
                .map(|(node_id, _)| *node_id)
                .collect()
        };
        
        if nodes_to_rollback.is_empty() {
            let result = self.update_results.get_mut(&update_id)
                .ok_or(DistributedHMRError::UpdateNotFound(update_id))?;
            result.finalize(DistributedUpdateStatus::RolledBack);
            return Ok(result.clone());
        }
        
        // Execute rollback on each node
        let mut rollback_results = Vec::new();
        for &node_id in &nodes_to_rollback {
            let success = self.simulate_node_rollback(node_id);
            rollback_results.push((node_id, success));
        }
        
        let result = self.update_results.get_mut(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?;
        
        for (node_id, success) in rollback_results {
            if success {
                result.update_node_status(node_id, NodeUpdateState::RolledBack, None);
            } else {
                let error_msg = format!("Rollback failed on node {node_id:?}");
                result.update_node_status(node_id, NodeUpdateState::Failed, Some(error_msg));
            }
        }
        
        result.finalize(DistributedUpdateStatus::RolledBack);
        Ok(result.clone())
    }
    
    fn simulate_node_rollback(&self, _node_id: NodeId) -> bool {
        // In a real implementation, this would:
        // 1. Send rollback request to the node
        // 2. Wait for confirmation
        // 3. Verify the rollback was successful
        // 4. Check node health after rollback
        
        // For simulation, we'll return success most of the time
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        _node_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        // 95% success rate for rollback simulation
        (hash % 20) != 0
    }
    
    pub fn add_cluster_node(&mut self, node_id: NodeId) {
        self.cluster_nodes.insert(node_id, NodeUpdateStatus {
            node_id,
            status: NodeUpdateState::Healthy,
            last_updated: SystemTime::now(),
            error_message: None,
            rollback_count: 0,
        });
    }
    
    pub fn remove_cluster_node(&mut self, node_id: NodeId) {
        self.cluster_nodes.remove(&node_id);
    }
    
    fn update_cluster_health(&mut self) {
        let now = Instant::now();
        
        if now.duration_since(self.last_health_check) < self.policy.health_check_interval {
            return;
        }
        
        self.last_health_check = now;
        
        // Collect nodes that need health checks
        let nodes_to_check: Vec<NodeId> = self.cluster_nodes.iter()
            .filter(|(_, status)| matches!(status.status, NodeUpdateState::Healthy | NodeUpdateState::Unhealthy))
            .map(|(node_id, _)| *node_id)
            .collect();
        
        // Simulate health checks (in real implementation, this would be network calls)
        for node_id in nodes_to_check {
            let is_healthy = self.simulate_node_health_check(node_id);
            
            let new_status = if is_healthy {
                NodeUpdateState::Healthy
            } else {
                NodeUpdateState::Unhealthy
            };
            
            if let Some(status) = self.cluster_nodes.get_mut(&node_id) {
                if status.status != new_status {
                    status.status = new_status;
                    status.last_updated = SystemTime::now();
                }
            }
        }
    }
    
    fn simulate_node_health_check(&self, _node_id: NodeId) -> bool {
        // In a real implementation, this would ping the node and check its status
        // For simulation, we'll return healthy most of the time
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        _node_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        // 95% healthy rate for simulation
        (hash % 20) != 0
    }
}

impl<C: ConsensusProtocol> DistributedHMRCoordinator for BasicDistributedHMRCoordinator<C> {
    fn propose_update(
        &mut self, 
        module_hash: ContentHash, 
        update_plan: UpdatePlan, 
        target_nodes: Vec<NodeId>
    ) -> DistributedHMRResult<UpdateId> {
        // Validate the update against policy
        self.validate_update_policy(&target_nodes)?;
        
        let update_id = self.next_update_id();
        
        let update = DistributedUpdate {
            id: update_id,
            module_hash,
            update_plan,
            target_nodes: target_nodes.clone(),
            policy: self.policy.clone(),
            created_at: SystemTime::now(),
            created_by: self.node_id,
            consensus_proposal_id: None,
        };
        
        // If consensus is required, propose through consensus protocol
        let consensus_proposal_id = if self.policy.require_consensus {
            let proposal_data = serde_json::to_vec(&update)
                .map_err(|e| DistributedHMRError::SerializationError(e.to_string()))?;
            
            let proposal_id = self.consensus.propose(ProposalType::UpdateCoordination, proposal_data)?;
            Some(proposal_id)
        } else {
            None
        };
        
        let mut update = update;
        update.consensus_proposal_id = consensus_proposal_id;
        
        // Create initial result
        let mut result = DistributedUpdateResult::new(update_id);
        for node_id in &target_nodes {
            result.update_node_status(*node_id, NodeUpdateState::Pending, None);
        }
        
        self.updates.insert(update_id, update);
        self.update_results.insert(update_id, result);
        
        Ok(update_id)
    }
    
    fn execute_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<DistributedUpdateResult> {
        let update = self.updates.get(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?
            .clone();
        
        // Check consensus if required
        if let Some(proposal_id) = update.consensus_proposal_id {
            match self.consensus.get_proposal_status(proposal_id) {
                Some(crate::consensus::ProposalStatus::Accepted) => {
                    // Consensus reached, proceed with update
                }
                Some(crate::consensus::ProposalStatus::Rejected) => {
                    let result = self.update_results.get_mut(&update_id).unwrap();
                    result.finalize(DistributedUpdateStatus::Cancelled);
                    return Err(DistributedHMRError::UpdateFailed("Update rejected by consensus".to_string()));
                }
                Some(crate::consensus::ProposalStatus::Voting) => {
                    return Err(DistributedHMRError::InvalidState("Consensus still in progress".to_string()));
                }
                _ => {
                    return Err(DistributedHMRError::InvalidState("Invalid consensus state".to_string()));
                }
            }
        }
        
        // Update cluster health before executing
        self.update_cluster_health();
        
        // Execute the rolling update
        match self.execute_rolling_update(update_id) {
            Ok(()) => {
                let result = self.update_results.get_mut(&update_id).unwrap();
                result.finalize(DistributedUpdateStatus::Completed);
                Ok(result.clone())
            }
            Err(e) => {
                // Handle failure and potential rollback
                if update.policy.auto_rollback_on_failure {
                    match self.execute_rollback(update_id) {
                        Ok(result) => Ok(result),
                        Err(rollback_error) => {
                            let result = self.update_results.get_mut(&update_id).unwrap();
                            result.finalize(DistributedUpdateStatus::Failed(rollback_error.to_string()));
                            Err(DistributedHMRError::RollbackFailed(rollback_error.to_string()))
                        }
                    }
                } else {
                    let result = self.update_results.get_mut(&update_id).unwrap();
                    result.finalize(DistributedUpdateStatus::Failed(e.to_string()));
                    Err(e)
                }
            }
        }
    }
    
    fn get_update_status(&self, update_id: UpdateId) -> Option<&DistributedUpdateResult> {
        self.update_results.get(&update_id)
    }
    
    fn cancel_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<()> {
        let result = self.update_results.get_mut(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?;
        
        if result.is_complete() {
            return Err(DistributedHMRError::InvalidState("Update already completed".to_string()));
        }
        
        result.finalize(DistributedUpdateStatus::Cancelled);
        Ok(())
    }
    
    fn rollback_update(&mut self, update_id: UpdateId) -> DistributedHMRResult<DistributedUpdateResult> {
        self.execute_rollback(update_id)
    }
    
    fn set_update_policy(&mut self, policy: UpdatePolicy) -> DistributedHMRResult<()> {
        self.policy = policy;
        Ok(())
    }
    
    fn get_update_policy(&self) -> &UpdatePolicy {
        &self.policy
    }
    
    fn get_active_updates(&self) -> Vec<UpdateId> {
        self.update_results.iter()
            .filter(|(_, result)| !result.is_complete())
            .map(|(id, _)| *id)
            .collect()
    }
    
    fn check_cluster_health(&self) -> ClusterHealthStatus {
        let mut health = ClusterHealthStatus::new(self.cluster_nodes.len());
        
        for status in self.cluster_nodes.values() {
            match status.status {
                NodeUpdateState::Healthy => health.healthy_nodes += 1,
                NodeUpdateState::Unhealthy => health.unhealthy_nodes += 1,
                NodeUpdateState::Updating => health.updating_nodes += 1,
                NodeUpdateState::Failed => health.failed_nodes += 1,
                _ => {}
            }
        }
        
        health.update_health();
        health
    }
    
    fn force_update_nodes(&mut self, update_id: UpdateId, nodes: Vec<NodeId>) -> DistributedHMRResult<()> {
        let update = self.updates.get(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?
            .clone();
        
        let _result = self.update_results.get_mut(&update_id)
            .ok_or(DistributedHMRError::UpdateNotFound(update_id))?;
        
        // Force update on specified nodes
        for node_id in nodes {
            if !update.target_nodes.contains(&node_id) {
                return Err(DistributedHMRError::NodeNotFound(node_id));
            }
            
            {
                let result = self.update_results.get_mut(&update_id).unwrap();
                result.update_node_status(node_id, NodeUpdateState::Updating, None);
            }
            
            let success = self.simulate_node_update(node_id, &update.update_plan);
            
            let result = self.update_results.get_mut(&update_id).unwrap();
            if success {
                result.update_node_status(node_id, NodeUpdateState::Updated, None);
            } else {
                let error_msg = format!("Forced update failed on node {node_id:?}");
                result.update_node_status(node_id, NodeUpdateState::Failed, Some(error_msg));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::{BasicConsensusProtocol, ConsensusConfig};
    use crate::change_analysis::ChangeAnalysis;
    use std::time::Duration;
    
    fn create_test_update_plan() -> UpdatePlan {
        UpdatePlan {
            update_id: ContentHash::zero(),
            module_hash: ContentHash::zero(),
            new_module_hash: ContentHash::zero(),
            change_analysis: ChangeAnalysis {
                changed_module_hash: ContentHash::zero(),
                affected_modules: Vec::new(),
                schema_changes: Vec::new(),
                compatibility: crate::change_analysis::CompatibilityLevel::FullyCompatible,
                migration_required: false,
                optimizations: Vec::new(),
                dependency_changes: Vec::new(),
            },
            steps: Vec::new(),
            rollback_plan: None,
            validation_results: Vec::new(),
            estimated_duration: Duration::from_secs(10),
            priority: crate::hmr_coordinator::UpdatePriority::Normal,
            dependencies: Vec::new(),
            status: crate::hmr_coordinator::UpdateStatus::Planned,
            created_at: 0,
            started_at: None,
            completed_at: None,
        }
    }
    
    fn create_test_coordinator() -> BasicDistributedHMRCoordinator<BasicConsensusProtocol> {
        let node_id = NodeId::new(1);
        let consensus_config = ConsensusConfig::default();
        let consensus = BasicConsensusProtocol::new(node_id, consensus_config);
        let policy = UpdatePolicy::default();
        
        let mut coordinator = BasicDistributedHMRCoordinator::new(node_id, consensus, policy);
        
        // Add some test nodes
        coordinator.add_cluster_node(NodeId::new(1));
        coordinator.add_cluster_node(NodeId::new(2));
        coordinator.add_cluster_node(NodeId::new(3));
        
        coordinator
    }
    
    #[test]
    fn test_propose_update() {
        let mut coordinator = create_test_coordinator();
        
        let module_hash = ContentHash::zero();
        let update_plan = create_test_update_plan();
        let target_nodes = vec![NodeId::new(1), NodeId::new(2)];
        
        let update_id = coordinator.propose_update(module_hash, update_plan, target_nodes).unwrap();
        
        assert!(coordinator.updates.contains_key(&update_id));
        assert!(coordinator.update_results.contains_key(&update_id));
        
        let result = coordinator.get_update_status(update_id).unwrap();
        assert_eq!(result.status, DistributedUpdateStatus::Proposed);
        assert_eq!(result.node_results.len(), 2);
    }
    
    #[test]
    fn test_update_policy_validation() {
        let mut coordinator = create_test_coordinator();
        
        // Set policy requiring minimum healthy nodes
        let mut policy = UpdatePolicy::default();
        policy.min_healthy_nodes = 5; // More than available nodes
        coordinator.set_update_policy(policy).unwrap();
        
        let module_hash = ContentHash::zero();
        let update_plan = create_test_update_plan();
        let target_nodes = vec![NodeId::new(1)];
        
        let result = coordinator.propose_update(module_hash, update_plan, target_nodes);
        assert!(matches!(result, Err(DistributedHMRError::PolicyViolation(_))));
    }
    
    #[test]
    fn test_cluster_health_check() {
        let coordinator = create_test_coordinator();
        
        let health = coordinator.check_cluster_health();
        
        assert_eq!(health.total_nodes, 3);
        assert!(health.healthy_nodes > 0);
        assert!(matches!(health.overall_health, OverallHealth::Healthy | OverallHealth::Degraded));
    }
    
    #[test]
    fn test_distributed_update_result() {
        let update_id = UpdateId::new(1);
        let mut result = DistributedUpdateResult::new(update_id);
        
        assert_eq!(result.update_id, update_id);
        assert_eq!(result.status, DistributedUpdateStatus::Proposed);
        assert!(!result.is_complete());
        
        // Update node status
        result.update_node_status(NodeId::new(1), NodeUpdateState::Updated, None);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.success_rate(), 1.0);
        
        result.update_node_status(NodeId::new(2), NodeUpdateState::Failed, Some("Error".to_string()));
        assert_eq!(result.failure_count, 1);
        assert_eq!(result.success_rate(), 0.5);
        
        result.finalize(DistributedUpdateStatus::Completed);
        assert!(result.is_complete());
        assert!(result.completed_at.is_some());
    }
    
    #[test]
    fn test_rolling_update_strategies() {
        let _update_id = UpdateId::new(1);
        
        // Test different rolling strategies
        let strategies = vec![
            RollingUpdateStrategy::Sequential,
            RollingUpdateStrategy::Parallel(2),
            RollingUpdateStrategy::Percentage(50.0),
            RollingUpdateStrategy::Canary { 
                canary_nodes: vec![NodeId::new(1)], 
                percentage: 100.0 
            },
        ];
        
        for strategy in strategies {
            let policy = UpdatePolicy {
                rolling_strategy: strategy,
                ..Default::default()
            };
            
            // Verify policy is valid
            assert!(policy.max_concurrent_updates > 0);
            assert!(policy.timeout > Duration::from_secs(0));
        }
    }
    
    #[test]
    fn test_distributed_update_result_node_status_updates() {
        let update_id = UpdateId::new(1);
        let mut result = DistributedUpdateResult::new(update_id);
        
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);
        
        // Add successful update
        result.update_node_status(node1, NodeUpdateState::Updated, None);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 0);
        
        // Add failed update
        result.update_node_status(node2, NodeUpdateState::Failed, Some("Test error".to_string()));
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 1);
        
        // Test success rate
        assert_eq!(result.success_rate(), 0.5);
        
        // Test rollback
        result.update_node_status(node1, NodeUpdateState::RolledBack, None);
        assert_eq!(result.rollback_count, 1);
    }
    
    #[test]
    fn test_cluster_health_status() {
        let mut health = ClusterHealthStatus::new(5);
        
        // 4/5 = 80% healthy -> Healthy
        health.healthy_nodes = 4;
        health.unhealthy_nodes = 1;
        health.update_health();
        
        assert_eq!(health.overall_health, OverallHealth::Healthy);
        
        // 2/5 = 40% healthy -> Critical (less than 50%)
        health.healthy_nodes = 2;
        health.unhealthy_nodes = 3;
        health.update_health();
        
        assert_eq!(health.overall_health, OverallHealth::Critical);
        
        // 3/5 = 60% healthy -> Degraded (between 50% and 80%)
        health.healthy_nodes = 3;
        health.unhealthy_nodes = 2;
        health.update_health();
        
        assert_eq!(health.overall_health, OverallHealth::Degraded);
        
        // 1/5 = 20% healthy -> Critical
        health.healthy_nodes = 1;
        health.unhealthy_nodes = 4;
        health.update_health();
        
        assert_eq!(health.overall_health, OverallHealth::Critical);
        
        // 0/5 = 0% healthy -> Unavailable
        health.healthy_nodes = 0;
        health.unhealthy_nodes = 5;
        health.update_health();
        
        assert_eq!(health.overall_health, OverallHealth::Unavailable);
    }
    
    #[test]
    fn test_basic_distributed_hmr_coordinator() {
        let node_id = NodeId::new(1);
        let consensus_config = ConsensusConfig::default();
        let consensus = BasicConsensusProtocol::new(node_id, consensus_config);
        let policy = UpdatePolicy::default();
        
        let mut coordinator = BasicDistributedHMRCoordinator::new(node_id, consensus, policy);
        
        // Add some cluster nodes
        coordinator.add_cluster_node(NodeId::new(1));
        coordinator.add_cluster_node(NodeId::new(2));
        coordinator.add_cluster_node(NodeId::new(3));
        
        let health = coordinator.check_cluster_health();
        assert_eq!(health.total_nodes, 3);
        
        // Test update proposal
        let module_hash = ContentHash::zero();
        let update_plan = create_test_update_plan();
        let target_nodes = vec![NodeId::new(1), NodeId::new(2)];
        
        // This should work since we have enough healthy nodes
        let update_id = coordinator.propose_update(module_hash, update_plan, target_nodes).unwrap();
        
        assert!(coordinator.get_update_status(update_id).is_some());
        assert_eq!(coordinator.get_active_updates(), vec![update_id]);
    }
    
    #[test]
    fn test_rollback_functionality() {
        let node_id = NodeId::new(1);
        let consensus_config = ConsensusConfig::default();
        let consensus = BasicConsensusProtocol::new(node_id, consensus_config);
        let mut policy = UpdatePolicy::default();
        policy.require_consensus = false;
        
        let mut coordinator = BasicDistributedHMRCoordinator::new(node_id, consensus, policy);
        
        // Add cluster nodes
        coordinator.add_cluster_node(NodeId::new(1));
        coordinator.add_cluster_node(NodeId::new(2));
        
        let module_hash = ContentHash::zero();
        let update_plan = create_test_update_plan();
        let target_nodes = vec![NodeId::new(1), NodeId::new(2)];
        
        let update_id = coordinator.propose_update(module_hash, update_plan, target_nodes).unwrap();
        let _result = coordinator.execute_update(update_id).unwrap();
        
        // Test rollback
        let rollback_result = coordinator.rollback_update(update_id).unwrap();
        assert!(matches!(rollback_result.status, DistributedUpdateStatus::RolledBack));
        assert!(rollback_result.rollback_count > 0);
    }
}