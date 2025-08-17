//! Consensus mechanism for distributed coordination
//!
//! This module implements a consensus protocol for coordinating updates across
//! distributed nodes, with support for node failure handling, network partition
//! handling, and node rejoin synchronization.

use crate::crdt::{LogicalTimestamp, NodeId, VersionVector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant, SystemTime};

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;

/// Errors that can occur during consensus operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusError {
    NodeNotFound(NodeId),
    ProposalNotFound(ProposalId),
    InsufficientNodes(String),
    NetworkPartition(String),
    TimeoutError(String),
    InvalidProposal(String),
    ConsensusNotReached(String),
    NodeAlreadyExists(NodeId),
    InvalidNodeState(NodeId, String),
    SplitBrainDetected(String),
    SerializationError(String),
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::NodeNotFound(node_id) => write!(f, "Node not found: {node_id:?}"),
            ConsensusError::ProposalNotFound(proposal_id) => {
                write!(f, "Proposal not found: {proposal_id:?}")
            }
            ConsensusError::InsufficientNodes(msg) => write!(f, "Insufficient nodes: {msg}"),
            ConsensusError::NetworkPartition(msg) => write!(f, "Network partition: {msg}"),
            ConsensusError::TimeoutError(msg) => write!(f, "Timeout error: {msg}"),
            ConsensusError::InvalidProposal(msg) => write!(f, "Invalid proposal: {msg}"),
            ConsensusError::ConsensusNotReached(msg) => write!(f, "Consensus not reached: {msg}"),
            ConsensusError::NodeAlreadyExists(node_id) => {
                write!(f, "Node already exists: {node_id:?}")
            }
            ConsensusError::InvalidNodeState(node_id, msg) => {
                write!(f, "Invalid node state for {node_id:?}: {msg}")
            }
            ConsensusError::SplitBrainDetected(msg) => write!(f, "Split-brain detected: {msg}"),
            ConsensusError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl std::error::Error for ConsensusError {}

/// Unique identifier for consensus proposals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub u64);

impl ProposalId {
    pub fn new(id: u64) -> Self {
        ProposalId(id)
    }
}

/// Unique identifier for consensus rounds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoundId(pub u64);

impl RoundId {
    pub fn new(id: u64) -> Self {
        RoundId(id)
    }
}

/// Status of a node in the consensus cluster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Inactive,
    Failed,
    Rejoining,
    Suspected,
    Partitioned,
}

/// Information about a node in the consensus cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusNode {
    pub id: NodeId,
    pub status: NodeStatus,
    pub last_seen: SystemTime,
    pub version_vector: VersionVector,
    pub address: String,
    pub heartbeat_interval: Duration,
    pub failure_detector_timeout: Duration,
}

impl ConsensusNode {
    pub fn new(id: NodeId, address: String) -> Self {
        ConsensusNode {
            id,
            status: NodeStatus::Active,
            last_seen: SystemTime::now(),
            version_vector: VersionVector::new(),
            address,
            heartbeat_interval: Duration::from_secs(1),
            failure_detector_timeout: Duration::from_secs(5),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, NodeStatus::Active | NodeStatus::Rejoining)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, NodeStatus::Failed | NodeStatus::Inactive)
    }

    pub fn update_heartbeat(&mut self) {
        self.last_seen = SystemTime::now();
        if self.status == NodeStatus::Suspected {
            self.status = NodeStatus::Active;
        }
    }

    pub fn mark_suspected(&mut self) {
        if self.status == NodeStatus::Active {
            self.status = NodeStatus::Suspected;
        }
    }

    pub fn mark_failed(&mut self) {
        self.status = NodeStatus::Failed;
    }

    pub fn mark_rejoining(&mut self) {
        self.status = NodeStatus::Rejoining;
    }

    pub fn mark_active(&mut self) {
        self.status = NodeStatus::Active;
        self.last_seen = SystemTime::now();
    }
}

/// Type of consensus proposal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    UpdateCoordination,
    NodeMembership,
    Configuration,
    StateSync,
    Custom(String),
}

/// A proposal for consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: NodeId,
    pub proposal_type: ProposalType,
    pub data: Vec<u8>,
    pub timestamp: LogicalTimestamp,
    pub round: RoundId,
    pub required_votes: usize,
    pub timeout: Duration,
}

/// Vote on a consensus proposal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Accept,
    Reject(String),
    Abstain,
}

/// Vote record for a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub voter: NodeId,
    pub vote: Vote,
    pub timestamp: LogicalTimestamp,
    pub version_vector: VersionVector,
}

/// Status of a consensus proposal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Proposed,
    Voting,
    Accepted,
    Rejected,
    Timeout,
    Cancelled,
}

/// Information about a consensus proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalInfo {
    pub proposal: Proposal,
    pub status: ProposalStatus,
    pub votes: HashMap<NodeId, VoteRecord>,
    pub created_at: SystemTime,
    pub decided_at: Option<SystemTime>,
    pub accept_count: usize,
    pub reject_count: usize,
    pub abstain_count: usize,
}

impl ProposalInfo {
    pub fn new(proposal: Proposal) -> Self {
        ProposalInfo {
            proposal,
            status: ProposalStatus::Proposed,
            votes: HashMap::new(),
            created_at: SystemTime::now(),
            decided_at: None,
            accept_count: 0,
            reject_count: 0,
            abstain_count: 0,
        }
    }

    pub fn add_vote(&mut self, vote_record: VoteRecord) {
        // Remove previous vote from this node if it exists
        if let Some(old_vote) = self.votes.get(&vote_record.voter) {
            match old_vote.vote {
                Vote::Accept => self.accept_count -= 1,
                Vote::Reject(_) => self.reject_count -= 1,
                Vote::Abstain => self.abstain_count -= 1,
            }
        }

        // Add new vote
        match vote_record.vote {
            Vote::Accept => self.accept_count += 1,
            Vote::Reject(_) => self.reject_count += 1,
            Vote::Abstain => self.abstain_count += 1,
        }

        self.votes.insert(vote_record.voter, vote_record);
    }

    pub fn has_majority(&self, total_nodes: usize) -> bool {
        let majority = (total_nodes / 2) + 1;
        self.accept_count >= majority
    }

    pub fn is_rejected(&self, total_nodes: usize) -> bool {
        let majority = (total_nodes / 2) + 1;
        self.reject_count >= majority
    }

    pub fn has_quorum(&self, total_nodes: usize) -> bool {
        let quorum = (total_nodes / 2) + 1;
        self.votes.len() >= quorum
    }
}

/// Configuration for the consensus mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub heartbeat_interval: Duration,
    pub failure_detector_timeout: Duration,
    pub proposal_timeout: Duration,
    pub max_concurrent_proposals: usize,
    pub min_cluster_size: usize,
    pub split_brain_detection_enabled: bool,
    pub auto_rejoin_enabled: bool,
    pub rejoin_timeout: Duration,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            heartbeat_interval: Duration::from_secs(1),
            failure_detector_timeout: Duration::from_secs(5),
            proposal_timeout: Duration::from_secs(30),
            max_concurrent_proposals: 10,
            min_cluster_size: 1,
            split_brain_detection_enabled: true,
            auto_rejoin_enabled: true,
            rejoin_timeout: Duration::from_secs(60),
        }
    }
}

/// Consensus mechanism for coordinating distributed updates
pub trait ConsensusProtocol {
    /// Propose a new consensus decision
    fn propose(
        &mut self,
        proposal_type: ProposalType,
        data: Vec<u8>,
    ) -> ConsensusResult<ProposalId>;

    /// Vote on a proposal
    fn vote(&mut self, proposal_id: ProposalId, vote: Vote) -> ConsensusResult<()>;

    /// Get the status of a proposal
    fn get_proposal_status(&self, proposal_id: ProposalId) -> Option<ProposalStatus>;

    /// Get detailed information about a proposal
    fn get_proposal_info(&self, proposal_id: ProposalId) -> Option<&ProposalInfo>;

    /// Add a node to the consensus cluster
    fn add_node(&mut self, node: ConsensusNode) -> ConsensusResult<()>;

    /// Remove a node from the consensus cluster
    fn remove_node(&mut self, node_id: NodeId) -> ConsensusResult<()>;

    /// Handle node failure
    fn handle_node_failure(&mut self, node_id: NodeId) -> ConsensusResult<()>;

    /// Handle node rejoin
    fn handle_node_rejoin(&mut self, node_id: NodeId) -> ConsensusResult<()>;

    /// Process heartbeats and detect failures
    fn process_heartbeats(&mut self) -> ConsensusResult<Vec<NodeId>>;

    /// Check for network partitions
    fn detect_network_partition(&self) -> ConsensusResult<Option<Vec<NodeId>>>;

    /// Handle split-brain scenarios
    fn handle_split_brain(&mut self, partition_info: Vec<NodeId>) -> ConsensusResult<()>;

    /// Get all active nodes
    fn get_active_nodes(&self) -> Vec<NodeId>;

    /// Get all nodes with their status
    fn get_all_nodes(&self) -> Vec<&ConsensusNode>;

    /// Get current cluster size
    fn cluster_size(&self) -> usize;

    /// Check if cluster has quorum
    fn has_quorum(&self) -> bool;

    /// Process pending proposals and timeouts
    fn process_proposals(&mut self) -> ConsensusResult<Vec<ProposalId>>;
}

/// Basic implementation of consensus protocol using a simplified Raft-like algorithm
#[derive(Debug)]
pub struct BasicConsensusProtocol {
    node_id: NodeId,
    nodes: HashMap<NodeId, ConsensusNode>,
    proposals: HashMap<ProposalId, ProposalInfo>,
    next_proposal_id: u64,
    next_round_id: u64,
    config: ConsensusConfig,
    last_heartbeat_check: Instant,
    suspected_nodes: HashSet<NodeId>,
    partition_history: VecDeque<(SystemTime, Vec<NodeId>)>,
}

impl BasicConsensusProtocol {
    pub fn new(node_id: NodeId, config: ConsensusConfig) -> Self {
        let mut nodes = HashMap::new();
        let self_node = ConsensusNode::new(node_id, "localhost".to_string());
        nodes.insert(node_id, self_node);

        BasicConsensusProtocol {
            node_id,
            nodes,
            proposals: HashMap::new(),
            next_proposal_id: 0,
            next_round_id: 0,
            config,
            last_heartbeat_check: Instant::now(),
            suspected_nodes: HashSet::new(),
            partition_history: VecDeque::new(),
        }
    }

    fn next_proposal_id(&mut self) -> ProposalId {
        let id = ProposalId::new(self.next_proposal_id);
        self.next_proposal_id += 1;
        id
    }

    fn next_round_id(&mut self) -> RoundId {
        let id = RoundId::new(self.next_round_id);
        self.next_round_id += 1;
        id
    }

    fn check_proposal_timeouts(&mut self) -> Vec<ProposalId> {
        let mut timed_out = Vec::new();
        let now = SystemTime::now();

        for (proposal_id, proposal_info) in self.proposals.iter_mut() {
            if proposal_info.status == ProposalStatus::Voting {
                let elapsed = now
                    .duration_since(proposal_info.created_at)
                    .unwrap_or(Duration::from_secs(0));

                if elapsed > proposal_info.proposal.timeout {
                    proposal_info.status = ProposalStatus::Timeout;
                    proposal_info.decided_at = Some(now);
                    timed_out.push(*proposal_id);
                }
            }
        }

        timed_out
    }

    fn update_proposal_status(&mut self, proposal_id: ProposalId) {
        let active_nodes = self.get_active_nodes().len();

        if let Some(proposal_info) = self.proposals.get_mut(&proposal_id) {
            if proposal_info.status == ProposalStatus::Voting {
                if proposal_info.has_majority(active_nodes) {
                    proposal_info.status = ProposalStatus::Accepted;
                    proposal_info.decided_at = Some(SystemTime::now());
                } else if proposal_info.is_rejected(active_nodes) {
                    proposal_info.status = ProposalStatus::Rejected;
                    proposal_info.decided_at = Some(SystemTime::now());
                }
            }
        }
    }

    fn detect_suspected_nodes(&mut self) -> Vec<NodeId> {
        let mut newly_suspected = Vec::new();
        let now = SystemTime::now();

        for (node_id, node) in self.nodes.iter_mut() {
            if *node_id != self.node_id && node.is_active() {
                let elapsed = now
                    .duration_since(node.last_seen)
                    .unwrap_or(Duration::from_secs(0));

                if elapsed > node.failure_detector_timeout {
                    if !self.suspected_nodes.contains(node_id) {
                        node.mark_suspected();
                        self.suspected_nodes.insert(*node_id);
                        newly_suspected.push(*node_id);
                    }

                    // If suspected for too long, mark as failed
                    if elapsed > node.failure_detector_timeout * 2 {
                        node.mark_failed();
                        self.suspected_nodes.remove(node_id);
                    }
                }
            }
        }

        newly_suspected
    }

    fn check_split_brain(&self) -> Option<Vec<NodeId>> {
        if !self.config.split_brain_detection_enabled {
            return None;
        }

        let active_nodes: Vec<NodeId> = self.get_active_nodes();
        let total_nodes = self.nodes.len();

        // Simple split-brain detection: if we have less than majority of nodes
        if active_nodes.len() < (total_nodes / 2) + 1 && total_nodes > 1 {
            let partitioned_nodes: Vec<NodeId> = self
                .nodes
                .keys()
                .filter(|&id| !active_nodes.contains(id))
                .copied()
                .collect();

            if !partitioned_nodes.is_empty() {
                return Some(partitioned_nodes);
            }
        }

        None
    }

    fn record_partition(&mut self, partitioned_nodes: Vec<NodeId>) {
        let now = SystemTime::now();
        self.partition_history.push_back((now, partitioned_nodes));

        // Keep only recent partition history
        let cutoff = now - Duration::from_secs(300); // 5 minutes
        while let Some((timestamp, _)) = self.partition_history.front() {
            if *timestamp < cutoff {
                self.partition_history.pop_front();
            } else {
                break;
            }
        }
    }
}

impl ConsensusProtocol for BasicConsensusProtocol {
    fn propose(
        &mut self,
        proposal_type: ProposalType,
        data: Vec<u8>,
    ) -> ConsensusResult<ProposalId> {
        // Check if we have quorum
        if !self.has_quorum() {
            return Err(ConsensusError::InsufficientNodes(
                "Cannot propose without quorum".to_string(),
            ));
        }

        // Check concurrent proposal limit
        let active_proposals = self
            .proposals
            .values()
            .filter(|p| matches!(p.status, ProposalStatus::Proposed | ProposalStatus::Voting))
            .count();

        if active_proposals >= self.config.max_concurrent_proposals {
            return Err(ConsensusError::InvalidProposal(
                "Too many concurrent proposals".to_string(),
            ));
        }

        let proposal_id = self.next_proposal_id();
        let round_id = self.next_round_id();
        let active_nodes = self.get_active_nodes().len();
        let required_votes = (active_nodes / 2) + 1;

        let proposal = Proposal {
            id: proposal_id,
            proposer: self.node_id,
            proposal_type,
            data,
            timestamp: LogicalTimestamp::new(0), // Should be updated with proper logical clock
            round: round_id,
            required_votes,
            timeout: self.config.proposal_timeout,
        };

        let mut proposal_info = ProposalInfo::new(proposal);
        proposal_info.status = ProposalStatus::Voting;

        self.proposals.insert(proposal_id, proposal_info);

        Ok(proposal_id)
    }

    fn vote(&mut self, proposal_id: ProposalId, vote: Vote) -> ConsensusResult<()> {
        let proposal_info = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or(ConsensusError::ProposalNotFound(proposal_id))?;

        if proposal_info.status != ProposalStatus::Voting {
            return Err(ConsensusError::InvalidProposal(
                "Proposal is not in voting state".to_string(),
            ));
        }

        let vote_record = VoteRecord {
            voter: self.node_id,
            vote,
            timestamp: LogicalTimestamp::new(0), // Should be updated with proper logical clock
            version_vector: VersionVector::new(),
        };

        proposal_info.add_vote(vote_record);
        self.update_proposal_status(proposal_id);

        Ok(())
    }

    fn get_proposal_status(&self, proposal_id: ProposalId) -> Option<ProposalStatus> {
        self.proposals
            .get(&proposal_id)
            .map(|info| info.status.clone())
    }

    fn get_proposal_info(&self, proposal_id: ProposalId) -> Option<&ProposalInfo> {
        self.proposals.get(&proposal_id)
    }

    fn add_node(&mut self, node: ConsensusNode) -> ConsensusResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(ConsensusError::NodeAlreadyExists(node.id));
        }

        self.nodes.insert(node.id, node);
        Ok(())
    }

    fn remove_node(&mut self, node_id: NodeId) -> ConsensusResult<()> {
        if node_id == self.node_id {
            return Err(ConsensusError::InvalidNodeState(
                node_id,
                "Cannot remove self from cluster".to_string(),
            ));
        }

        self.nodes
            .remove(&node_id)
            .ok_or(ConsensusError::NodeNotFound(node_id))?;

        self.suspected_nodes.remove(&node_id);

        Ok(())
    }

    fn handle_node_failure(&mut self, node_id: NodeId) -> ConsensusResult<()> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(ConsensusError::NodeNotFound(node_id))?;

        node.mark_failed();
        self.suspected_nodes.remove(&node_id);

        // Check if we still have quorum after this failure
        if !self.has_quorum() {
            return Err(ConsensusError::InsufficientNodes(
                "Lost quorum after node failure".to_string(),
            ));
        }

        Ok(())
    }

    fn handle_node_rejoin(&mut self, node_id: NodeId) -> ConsensusResult<()> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(ConsensusError::NodeNotFound(node_id))?;

        if !node.is_failed() {
            return Err(ConsensusError::InvalidNodeState(
                node_id,
                "Node is not in failed state".to_string(),
            ));
        }

        node.mark_rejoining();
        self.suspected_nodes.remove(&node_id);

        // Start rejoin process - in a real implementation, this would involve
        // state synchronization and validation

        // For now, immediately mark as active after rejoin timeout
        // In practice, this would be handled asynchronously
        node.mark_active();

        Ok(())
    }

    fn process_heartbeats(&mut self) -> ConsensusResult<Vec<NodeId>> {
        let now = Instant::now();

        // Only check heartbeats at configured intervals
        if now.duration_since(self.last_heartbeat_check) < self.config.heartbeat_interval {
            return Ok(Vec::new());
        }

        self.last_heartbeat_check = now;

        let newly_suspected = self.detect_suspected_nodes();

        Ok(newly_suspected)
    }

    fn detect_network_partition(&self) -> ConsensusResult<Option<Vec<NodeId>>> {
        Ok(self.check_split_brain())
    }

    fn handle_split_brain(&mut self, partition_info: Vec<NodeId>) -> ConsensusResult<()> {
        self.record_partition(partition_info.clone());

        // Mark partitioned nodes as such
        for node_id in &partition_info {
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.status = NodeStatus::Partitioned;
            }
        }

        // In a real implementation, this would involve more sophisticated
        // split-brain resolution strategies, such as:
        // - Quorum-based decisions
        // - External arbitration
        // - Manual intervention

        Err(ConsensusError::SplitBrainDetected(format!(
            "Split-brain detected with {} partitioned nodes",
            partition_info.len()
        )))
    }

    fn get_active_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.is_active())
            .map(|(id, _)| *id)
            .collect()
    }

    fn get_all_nodes(&self) -> Vec<&ConsensusNode> {
        self.nodes.values().collect()
    }

    fn cluster_size(&self) -> usize {
        self.nodes.len()
    }

    fn has_quorum(&self) -> bool {
        let active_nodes = self.get_active_nodes().len();
        let total_nodes = self.cluster_size();

        if total_nodes < self.config.min_cluster_size {
            return false;
        }

        active_nodes > (total_nodes / 2)
    }

    fn process_proposals(&mut self) -> ConsensusResult<Vec<ProposalId>> {
        let timed_out = self.check_proposal_timeouts();

        // Update status of all voting proposals
        let proposal_ids: Vec<ProposalId> = self.proposals.keys().copied().collect();
        for proposal_id in proposal_ids {
            self.update_proposal_status(proposal_id);
        }

        Ok(timed_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn create_test_node(id: u64, address: &str) -> ConsensusNode {
        ConsensusNode::new(NodeId::new(id), address.to_string())
    }

    fn create_test_proposal(id: u64, proposer_id: u64) -> Proposal {
        Proposal {
            id: ProposalId::new(id),
            proposer: NodeId::new(proposer_id),
            proposal_type: ProposalType::UpdateCoordination,
            data: vec![1, 2, 3],
            timestamp: LogicalTimestamp::new(0),
            round: RoundId::new(1),
            required_votes: 2,
            timeout: Duration::from_secs(30),
        }
    }

    #[test]
    fn test_consensus_node_creation() {
        let node_id = NodeId::new(1);
        let node = ConsensusNode::new(node_id, "localhost:8080".to_string());

        assert_eq!(node.id, node_id);
        assert_eq!(node.status, NodeStatus::Active);
        assert_eq!(node.address, "localhost:8080");
        assert!(node.is_active());
        assert!(!node.is_failed());
    }

    #[test]
    fn test_consensus_node_state_transitions() {
        let mut node = create_test_node(1, "localhost:8080");

        // Test heartbeat update
        node.mark_suspected();
        assert_eq!(node.status, NodeStatus::Suspected);
        
        node.update_heartbeat();
        assert_eq!(node.status, NodeStatus::Active);

        // Test failure marking
        node.mark_failed();
        assert_eq!(node.status, NodeStatus::Failed);
        assert!(node.is_failed());
        assert!(!node.is_active());

        // Test rejoining
        node.mark_rejoining();
        assert_eq!(node.status, NodeStatus::Rejoining);
        assert!(node.is_active());

        // Test marking active
        node.mark_active();
        assert_eq!(node.status, NodeStatus::Active);
    }

    #[test]
    fn test_proposal_info_voting() {
        let proposal = create_test_proposal(1, 1);
        let mut proposal_info = ProposalInfo::new(proposal);

        // Add accept vote
        let vote1 = VoteRecord {
            voter: NodeId::new(1),
            vote: Vote::Accept,
            timestamp: LogicalTimestamp::new(1),
            version_vector: VersionVector::new(),
        };
        proposal_info.add_vote(vote1);

        assert_eq!(proposal_info.accept_count, 1);
        assert!(!proposal_info.has_majority(3));

        // Add another accept vote
        let vote2 = VoteRecord {
            voter: NodeId::new(2),
            vote: Vote::Accept,
            timestamp: LogicalTimestamp::new(2),
            version_vector: VersionVector::new(),
        };
        proposal_info.add_vote(vote2);

        assert_eq!(proposal_info.accept_count, 2);
        assert!(proposal_info.has_majority(3));
    }

    #[test]
    fn test_proposal_info_vote_replacement() {
        let proposal = create_test_proposal(1, 1);
        let mut proposal_info = ProposalInfo::new(proposal);

        // Add initial reject vote
        let vote1 = VoteRecord {
            voter: NodeId::new(1),
            vote: Vote::Reject("initial rejection".to_string()),
            timestamp: LogicalTimestamp::new(1),
            version_vector: VersionVector::new(),
        };
        proposal_info.add_vote(vote1);
        assert_eq!(proposal_info.reject_count, 1);
        assert_eq!(proposal_info.accept_count, 0);

        // Replace with accept vote
        let vote2 = VoteRecord {
            voter: NodeId::new(1),
            vote: Vote::Accept,
            timestamp: LogicalTimestamp::new(2),
            version_vector: VersionVector::new(),
        };
        proposal_info.add_vote(vote2);
        assert_eq!(proposal_info.reject_count, 0);
        assert_eq!(proposal_info.accept_count, 1);
    }

    #[test]
    fn test_proposal_info_rejection() {
        let proposal = create_test_proposal(1, 1);
        let mut proposal_info = ProposalInfo::new(proposal);

        // Add reject votes
        for i in 1..=3 {
            let vote = VoteRecord {
                voter: NodeId::new(i),
                vote: Vote::Reject(format!("rejection {i}")),
                timestamp: LogicalTimestamp::new(i),
                version_vector: VersionVector::new(),
            };
            proposal_info.add_vote(vote);
        }

        assert_eq!(proposal_info.reject_count, 3);
        assert!(proposal_info.is_rejected(5)); // 3 out of 5 nodes rejected
    }

    #[test]
    fn test_proposal_info_abstain_votes() {
        let proposal = create_test_proposal(1, 1);
        let mut proposal_info = ProposalInfo::new(proposal);

        let vote = VoteRecord {
            voter: NodeId::new(1),
            vote: Vote::Abstain,
            timestamp: LogicalTimestamp::new(1),
            version_vector: VersionVector::new(),
        };
        proposal_info.add_vote(vote);

        assert_eq!(proposal_info.abstain_count, 1);
        assert_eq!(proposal_info.accept_count, 0);
        assert_eq!(proposal_info.reject_count, 0);
    }

    #[test]
    fn test_proposal_info_quorum() {
        let proposal = create_test_proposal(1, 1);
        let mut proposal_info = ProposalInfo::new(proposal);

        // Add votes from 3 nodes out of 5
        for i in 1..=3 {
            let vote = VoteRecord {
                voter: NodeId::new(i),
                vote: Vote::Accept,
                timestamp: LogicalTimestamp::new(i),
                version_vector: VersionVector::new(),
            };
            proposal_info.add_vote(vote);
        }

        assert!(proposal_info.has_quorum(5));
        assert!(!proposal_info.has_quorum(7)); // Need 4 out of 7 for quorum
    }

    #[test]
    fn test_basic_consensus_protocol_creation() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let consensus = BasicConsensusProtocol::new(node_id, config);

        assert_eq!(consensus.cluster_size(), 1);
        assert!(consensus.has_quorum());
        assert_eq!(consensus.get_active_nodes(), vec![node_id]);
    }

    #[test]
    fn test_consensus_add_remove_nodes() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add another node
        let node2 = create_test_node(2, "localhost:8081");
        consensus.add_node(node2).unwrap();

        assert_eq!(consensus.cluster_size(), 2);
        assert!(consensus.has_quorum());

        // Try to add duplicate node
        let node2_dup = create_test_node(2, "localhost:8082");
        assert!(matches!(
            consensus.add_node(node2_dup),
            Err(ConsensusError::NodeAlreadyExists(_))
        ));

        // Remove node
        consensus.remove_node(NodeId::new(2)).unwrap();
        assert_eq!(consensus.cluster_size(), 1);

        // Try to remove non-existent node
        assert!(matches!(
            consensus.remove_node(NodeId::new(3)),
            Err(ConsensusError::NodeNotFound(_))
        ));

        // Try to remove self
        assert!(matches!(
            consensus.remove_node(node_id),
            Err(ConsensusError::InvalidNodeState(_, _))
        ));
    }

    #[test]
    fn test_consensus_proposal_lifecycle() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add nodes to have a proper cluster
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();
        consensus.add_node(create_test_node(3, "localhost:8082")).unwrap();

        // Propose something
        let proposal_id = consensus
            .propose(ProposalType::UpdateCoordination, vec![1, 2, 3])
            .unwrap();

        assert_eq!(
            consensus.get_proposal_status(proposal_id),
            Some(ProposalStatus::Voting)
        );

        // Vote on the proposal
        consensus.vote(proposal_id, Vote::Accept).unwrap();

        // Should still be voting since we need majority (2 out of 3)
        assert_eq!(
            consensus.get_proposal_status(proposal_id),
            Some(ProposalStatus::Voting)
        );

        // Get proposal info
        let proposal_info = consensus.get_proposal_info(proposal_id).unwrap();
        assert_eq!(proposal_info.accept_count, 1);
        assert!(!proposal_info.has_majority(3));
    }

    #[test]
    fn test_consensus_proposal_without_quorum() {
        let node_id = NodeId::new(1);
        let mut config = ConsensusConfig::default();
        config.min_cluster_size = 3;
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Only one node, no quorum
        assert!(!consensus.has_quorum());

        // Should fail to propose
        let result = consensus.propose(ProposalType::UpdateCoordination, vec![1, 2, 3]);
        assert!(matches!(result, Err(ConsensusError::InsufficientNodes(_))));
    }

    #[test]
    fn test_consensus_max_concurrent_proposals() {
        let node_id = NodeId::new(1);
        let mut config = ConsensusConfig::default();
        config.max_concurrent_proposals = 2;
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add nodes for quorum
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();

        // Create max concurrent proposals
        let _proposal1 = consensus
            .propose(ProposalType::UpdateCoordination, vec![1])
            .unwrap();
        let _proposal2 = consensus
            .propose(ProposalType::NodeMembership, vec![2])
            .unwrap();

        // Third proposal should fail
        let result = consensus.propose(ProposalType::Configuration, vec![3]);
        assert!(matches!(result, Err(ConsensusError::InvalidProposal(_))));
    }

    #[test]
    fn test_consensus_vote_on_nonexistent_proposal() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        let result = consensus.vote(ProposalId::new(999), Vote::Accept);
        assert!(matches!(result, Err(ConsensusError::ProposalNotFound(_))));
    }

    #[test]
    fn test_consensus_vote_on_decided_proposal() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Create a proposal and manually set it as accepted
        let proposal_id = consensus
            .propose(ProposalType::UpdateCoordination, vec![1, 2, 3])
            .unwrap();

        // Manually mark as accepted
        if let Some(proposal_info) = consensus.proposals.get_mut(&proposal_id) {
            proposal_info.status = ProposalStatus::Accepted;
        }

        // Should fail to vote on decided proposal
        let result = consensus.vote(proposal_id, Vote::Accept);
        assert!(matches!(result, Err(ConsensusError::InvalidProposal(_))));
    }

    #[test]
    fn test_consensus_node_failure_handling() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add nodes
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();
        consensus.add_node(create_test_node(3, "localhost:8082")).unwrap();

        assert_eq!(consensus.get_active_nodes().len(), 3);

        // Handle node failure
        consensus.handle_node_failure(NodeId::new(2)).unwrap();

        // Check node is marked as failed
        let nodes = consensus.get_all_nodes();
        let failed_node = nodes.iter().find(|n| n.id == NodeId::new(2)).unwrap();
        assert_eq!(failed_node.status, NodeStatus::Failed);

        // Should still have quorum with 2 active nodes
        assert!(consensus.has_quorum());
    }

    #[test]
    fn test_consensus_node_failure_loses_quorum() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add one more node (total 2)
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();

        // Fail the other node - should lose quorum
        let result = consensus.handle_node_failure(NodeId::new(2));
        assert!(matches!(result, Err(ConsensusError::InsufficientNodes(_))));
    }

    #[test]
    fn test_consensus_node_rejoin() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add two more nodes to maintain quorum after failure
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();
        consensus.add_node(create_test_node(3, "localhost:8082")).unwrap();

        // Fail one node - should still have quorum
        consensus.handle_node_failure(NodeId::new(2)).unwrap();

        // Rejoin the node
        consensus.handle_node_rejoin(NodeId::new(2)).unwrap();

        // Check node is active again
        let nodes = consensus.get_all_nodes();
        let rejoined_node = nodes.iter().find(|n| n.id == NodeId::new(2)).unwrap();
        assert_eq!(rejoined_node.status, NodeStatus::Active);
    }

    #[test]
    fn test_consensus_node_rejoin_invalid_state() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add active node
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();

        // Try to rejoin active node
        let result = consensus.handle_node_rejoin(NodeId::new(2));
        assert!(matches!(result, Err(ConsensusError::InvalidNodeState(_, _))));
    }

    #[test]
    fn test_consensus_split_brain_detection() {
        let node_id = NodeId::new(1);
        let mut config = ConsensusConfig::default();
        config.split_brain_detection_enabled = true;
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add nodes to create a 5-node cluster
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();
        consensus.add_node(create_test_node(3, "localhost:8082")).unwrap();
        consensus.add_node(create_test_node(4, "localhost:8083")).unwrap();
        consensus.add_node(create_test_node(5, "localhost:8084")).unwrap();

        // Manually mark nodes as failed to simulate partition (fail 3 nodes, leaving 2 active)
        consensus.handle_node_failure(NodeId::new(3)).unwrap();
        consensus.handle_node_failure(NodeId::new(4)).unwrap();
        let _ = consensus.handle_node_failure(NodeId::new(5)); // This might fail due to quorum loss

        // Detect partition
        let partition = consensus.detect_network_partition().unwrap();
        assert!(partition.is_some());
    }

    #[test]
    fn test_consensus_split_brain_disabled() {
        let node_id = NodeId::new(1);
        let mut config = ConsensusConfig::default();
        config.split_brain_detection_enabled = false;
        let consensus = BasicConsensusProtocol::new(node_id, config);

        // Should not detect partition when disabled
        let partition = consensus.detect_network_partition().unwrap();
        assert!(partition.is_none());
    }

    #[test]
    fn test_consensus_handle_split_brain() {
        let node_id = NodeId::new(1);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Add nodes
        consensus.add_node(create_test_node(2, "localhost:8081")).unwrap();
        consensus.add_node(create_test_node(3, "localhost:8082")).unwrap();

        let partition_info = vec![NodeId::new(2), NodeId::new(3)];
        let result = consensus.handle_split_brain(partition_info);

        // Should return split-brain error
        assert!(matches!(result, Err(ConsensusError::SplitBrainDetected(_))));

        // Check nodes are marked as partitioned
        let nodes = consensus.get_all_nodes();
        for node in nodes {
            if node.id != node_id {
                assert_eq!(node.status, NodeStatus::Partitioned);
            }
        }
    }

    #[test]
    fn test_consensus_process_proposals_timeout() {
        let node_id = NodeId::new(1);
        let mut config = ConsensusConfig::default();
        config.proposal_timeout = Duration::from_millis(10); // Very short timeout
        let mut consensus = BasicConsensusProtocol::new(node_id, config);

        // Create proposal
        let proposal_id = consensus
            .propose(ProposalType::UpdateCoordination, vec![1, 2, 3])
            .unwrap();

        // Wait for timeout
        thread::sleep(Duration::from_millis(20));

        // Process proposals should detect timeout
        let timed_out = consensus.process_proposals().unwrap();
        assert!(timed_out.contains(&proposal_id));

        // Check proposal status
        assert_eq!(
            consensus.get_proposal_status(proposal_id),
            Some(ProposalStatus::Timeout)
        );
    }

    #[test]
    fn test_consensus_error_display() {
        let errors = vec![
            ConsensusError::NodeNotFound(NodeId::new(1)),
            ConsensusError::ProposalNotFound(ProposalId::new(1)),
            ConsensusError::InsufficientNodes("test".to_string()),
            ConsensusError::NetworkPartition("test".to_string()),
            ConsensusError::TimeoutError("test".to_string()),
            ConsensusError::InvalidProposal("test".to_string()),
            ConsensusError::ConsensusNotReached("test".to_string()),
            ConsensusError::NodeAlreadyExists(NodeId::new(1)),
            ConsensusError::InvalidNodeState(NodeId::new(1), "test".to_string()),
            ConsensusError::SplitBrainDetected("test".to_string()),
            ConsensusError::SerializationError("test".to_string()),
        ];

        for error in errors {
            let display_str = format!("{error}");
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_proposal_id_and_round_id() {
        let proposal_id = ProposalId::new(42);
        assert_eq!(proposal_id.0, 42);

        let round_id = RoundId::new(24);
        assert_eq!(round_id.0, 24);
    }

    #[test]
    fn test_consensus_config_default() {
        let config = ConsensusConfig::default();
        assert_eq!(config.heartbeat_interval, Duration::from_secs(1));
        assert_eq!(config.failure_detector_timeout, Duration::from_secs(5));
        assert_eq!(config.proposal_timeout, Duration::from_secs(30));
        assert_eq!(config.max_concurrent_proposals, 10);
        assert_eq!(config.min_cluster_size, 1);
        assert!(config.split_brain_detection_enabled);
        assert!(config.auto_rejoin_enabled);
        assert_eq!(config.rejoin_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_proposal_types() {
        let types = vec![
            ProposalType::UpdateCoordination,
            ProposalType::NodeMembership,
            ProposalType::Configuration,
            ProposalType::StateSync,
            ProposalType::Custom("test".to_string()),
        ];

        for proposal_type in types {
            // Test serialization/deserialization
            let serialized = serde_json::to_string(&proposal_type).unwrap();
            let deserialized: ProposalType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(proposal_type, deserialized);
        }
    }

    #[test]
    fn test_vote_types() {
        let votes = vec![
            Vote::Accept,
            Vote::Reject("reason".to_string()),
            Vote::Abstain,
        ];

        for vote in votes {
            // Test serialization/deserialization
            let serialized = serde_json::to_string(&vote).unwrap();
            let deserialized: Vote = serde_json::from_str(&serialized).unwrap();
            assert_eq!(vote, deserialized);
        }
    }

    #[test]
    fn test_node_status_transitions() {
        let statuses = vec![
            NodeStatus::Active,
            NodeStatus::Inactive,
            NodeStatus::Failed,
            NodeStatus::Rejoining,
            NodeStatus::Suspected,
            NodeStatus::Partitioned,
        ];

        for status in statuses {
            // Test serialization/deserialization
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: NodeStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}
