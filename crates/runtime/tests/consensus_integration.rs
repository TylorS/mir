//! Integration tests for the consensus module
//!
//! These tests verify the consensus protocol behavior in more complex scenarios
//! including multi-node coordination, failure recovery, and distributed scenarios.

use mir_runtime::consensus::{
    BasicConsensusProtocol, ConsensusConfig, ConsensusError, ConsensusNode, ConsensusProtocol,
    ProposalStatus, ProposalType, Vote,
};
use mir_runtime::crdt::NodeId;
use std::thread;
use std::time::Duration;

/// Helper to create a single consensus node with peers
fn create_consensus_with_peers(node_id: NodeId, peer_count: usize) -> BasicConsensusProtocol {
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes
    for i in 1..=peer_count {
        if NodeId::new(i as u64) != node_id {
            let peer_node = ConsensusNode::new(
                NodeId::new(i as u64),
                format!("localhost:808{i}"),
            );
            consensus.add_node(peer_node).unwrap();
        }
    }

    consensus
}

#[test]
fn test_single_node_consensus_proposal() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 3);

    // Node 1 proposes
    let proposal_id = consensus
        .propose(ProposalType::UpdateCoordination, vec![1, 2, 3])
        .unwrap();

    // Node votes on its own proposal
    consensus.vote(proposal_id, Vote::Accept).unwrap();

    // Process proposals to update status
    let _ = consensus.process_proposals();

    // With 3 nodes total, need 2 votes for majority, but only 1 vote so far
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );
}

#[test]
fn test_consensus_node_management() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 3);

    // Verify initial cluster state
    assert_eq!(consensus.cluster_size(), 3);
    assert_eq!(consensus.get_active_nodes().len(), 3);
    assert!(consensus.has_quorum());

    // Add another node
    let new_node = ConsensusNode::new(NodeId::new(4), "localhost:8084".to_string());
    consensus.add_node(new_node).unwrap();

    assert_eq!(consensus.cluster_size(), 4);
    assert_eq!(consensus.get_active_nodes().len(), 4);

    // Remove a node
    consensus.remove_node(NodeId::new(4)).unwrap();
    assert_eq!(consensus.cluster_size(), 3);
}

#[test]
fn test_consensus_proposal_with_multiple_votes() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 5);

    // Node 1 proposes
    let proposal_id = consensus
        .propose(ProposalType::NodeMembership, vec![4, 5, 6])
        .unwrap();

    // Vote on the proposal (simulating distributed voting)
    consensus.vote(proposal_id, Vote::Accept).unwrap();

    // Process proposals to update status
    let _ = consensus.process_proposals();

    // With 5 nodes total, need 3 votes for majority, but only 1 vote so far
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );

    // Verify the vote was recorded
    let proposal_info = consensus.get_proposal_info(proposal_id).unwrap();
    assert_eq!(proposal_info.accept_count, 1);
}

#[test]
fn test_consensus_node_failure_handling() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 5);

    // Verify initial state
    assert_eq!(consensus.get_active_nodes().len(), 5);
    assert!(consensus.has_quorum());

    // Handle node failure
    consensus.handle_node_failure(NodeId::new(2)).unwrap();
    
    // Should still have quorum with 4 active nodes
    assert_eq!(consensus.get_active_nodes().len(), 4);
    assert!(consensus.has_quorum());

    // Handle another failure
    consensus.handle_node_failure(NodeId::new(3)).unwrap();
    
    // Should still have quorum with 3 active nodes
    assert_eq!(consensus.get_active_nodes().len(), 3);
    assert!(consensus.has_quorum());
}

#[test]
fn test_consensus_quorum_loss_prevents_proposals() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 3);

    // Fail two nodes, leaving only one active
    let _ = consensus.handle_node_failure(NodeId::new(2)); // This will fail due to quorum loss
    let _ = consensus.handle_node_failure(NodeId::new(3)); // This will also fail
    
    // Should not have quorum
    assert!(!consensus.has_quorum());
    
    // Should not be able to propose without quorum
    let result = consensus.propose(ProposalType::UpdateCoordination, vec![1, 2, 3]);
    assert!(matches!(result, Err(ConsensusError::InsufficientNodes(_))));
}

#[test]
fn test_consensus_concurrent_proposals() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 3);

    // Create multiple concurrent proposals
    let proposal1 = consensus
        .propose(ProposalType::UpdateCoordination, vec![1])
        .unwrap();
    let proposal2 = consensus
        .propose(ProposalType::NodeMembership, vec![2])
        .unwrap();
    let proposal3 = consensus
        .propose(ProposalType::Configuration, vec![3])
        .unwrap();

    // All proposals should be in voting state
    assert_eq!(
        consensus.get_proposal_status(proposal1),
        Some(ProposalStatus::Voting)
    );
    assert_eq!(
        consensus.get_proposal_status(proposal2),
        Some(ProposalStatus::Voting)
    );
    assert_eq!(
        consensus.get_proposal_status(proposal3),
        Some(ProposalStatus::Voting)
    );
}

#[test]
fn test_consensus_node_rejoin_scenario() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 5);

    // Fail a node
    let failed_node = NodeId::new(2);
    consensus.handle_node_failure(failed_node).unwrap();

    // Verify cluster still has quorum
    assert!(consensus.has_quorum());
    assert_eq!(consensus.get_active_nodes().len(), 4);

    // Node rejoins
    consensus.handle_node_rejoin(failed_node).unwrap();

    // Verify all nodes are active again
    assert_eq!(consensus.get_active_nodes().len(), 5);
    assert!(consensus.has_quorum());
}

#[test]
fn test_consensus_proposal_timeout_handling() {
    let mut config = ConsensusConfig::default();
    config.proposal_timeout = Duration::from_millis(50);
    
    let node_id = NodeId::new(1);
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Add nodes for quorum
    consensus.add_node(ConsensusNode::new(NodeId::new(2), "localhost:8081".to_string())).unwrap();
    consensus.add_node(ConsensusNode::new(NodeId::new(3), "localhost:8082".to_string())).unwrap();

    // Create proposal
    let proposal_id = consensus
        .propose(ProposalType::UpdateCoordination, vec![1, 2, 3])
        .unwrap();

    // Don't vote, let it timeout
    thread::sleep(Duration::from_millis(100));

    // Process proposals should detect timeout
    let timed_out = consensus.process_proposals().unwrap();
    assert!(timed_out.contains(&proposal_id));

    // Verify status
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Timeout)
    );
}

#[test]
fn test_consensus_split_brain_detection_and_handling() {
    let mut config = ConsensusConfig::default();
    config.split_brain_detection_enabled = true;
    
    let node_id = NodeId::new(1);
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Create 5-node cluster
    for i in 2..=5 {
        consensus.add_node(ConsensusNode::new(
            NodeId::new(i),
            format!("localhost:808{i}"),
        )).unwrap();
    }

    // Simulate network partition - fail 3 nodes
    for i in 3..=5 {
        let _ = consensus.handle_node_failure(NodeId::new(i));
    }

    // Should detect partition
    let partition = consensus.detect_network_partition().unwrap();
    assert!(partition.is_some());

    let partitioned_nodes = partition.unwrap();
    assert_eq!(partitioned_nodes.len(), 3);

    // Handle split-brain should fail
    let result = consensus.handle_split_brain(partitioned_nodes);
    assert!(matches!(result, Err(ConsensusError::SplitBrainDetected(_))));
}

#[test]
fn test_consensus_different_proposal_types() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 3);

    let proposal_types = vec![
        ProposalType::UpdateCoordination,
        ProposalType::NodeMembership,
        ProposalType::Configuration,
        ProposalType::StateSync,
        ProposalType::Custom("test_custom".to_string()),
    ];

    let mut proposal_ids = Vec::new();

    // Create proposals of different types
    for (i, proposal_type) in proposal_types.into_iter().enumerate() {
        let proposal_id = consensus
            .propose(proposal_type, vec![i as u8])
            .unwrap();
        proposal_ids.push(proposal_id);
    }

    // All proposals should be in voting state
    for &proposal_id in &proposal_ids {
        assert_eq!(
            consensus.get_proposal_status(proposal_id),
            Some(ProposalStatus::Voting)
        );
    }
}

#[test]
fn test_consensus_large_cluster_scalability() {
    // Test with larger cluster to ensure scalability
    let cluster_size = 7;
    let mut consensus = create_consensus_with_peers(NodeId::new(1), cluster_size);

    // Verify cluster setup
    assert_eq!(consensus.cluster_size(), cluster_size);
    assert_eq!(consensus.get_active_nodes().len(), cluster_size);
    assert!(consensus.has_quorum());

    // Create multiple proposals
    let mut proposal_ids = Vec::new();
    for i in 0..5 {
        let proposal_id = consensus
            .propose(ProposalType::UpdateCoordination, vec![i as u8])
            .unwrap();
        proposal_ids.push(proposal_id);
    }

    // All proposals should be in voting state
    for &proposal_id in &proposal_ids {
        assert_eq!(
            consensus.get_proposal_status(proposal_id),
            Some(ProposalStatus::Voting)
        );
    }
}

#[test]
fn test_consensus_proposal_lifecycle() {
    let mut consensus = create_consensus_with_peers(NodeId::new(1), 3);

    let proposal_id = consensus
        .propose(ProposalType::UpdateCoordination, vec![1, 2, 3])
        .unwrap();

    // Initially in voting state
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );

    // Vote on the proposal
    consensus.vote(proposal_id, Vote::Accept).unwrap();

    // Still voting since we need majority (2 out of 3)
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );

    // Get proposal info to verify vote was recorded
    let proposal_info = consensus.get_proposal_info(proposal_id).unwrap();
    assert_eq!(proposal_info.accept_count, 1);
}

#[test]
fn test_consensus_heartbeat_processing() {
    let mut config = ConsensusConfig::default();
    config.heartbeat_interval = Duration::from_millis(10);
    config.failure_detector_timeout = Duration::from_millis(50);

    let node_id = NodeId::new(1);
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add nodes
    consensus.add_node(ConsensusNode::new(NodeId::new(2), "localhost:8081".to_string())).unwrap();
    consensus.add_node(ConsensusNode::new(NodeId::new(3), "localhost:8082".to_string())).unwrap();

    // Process heartbeats immediately - should be empty since not enough time has passed
    let suspected = consensus.process_heartbeats().unwrap();
    assert!(suspected.is_empty());

    // Verify initial state
    assert_eq!(consensus.get_active_nodes().len(), 3);
    assert!(consensus.has_quorum());
}