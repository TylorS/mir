//! End-to-end tests for distributed HMR coordination using consensus
//!
//! These tests verify that the consensus mechanism works correctly in the context
//! of the distributed hot-module-reloading system.

use mir_runtime::consensus::{
    BasicConsensusProtocol, ConsensusConfig, ConsensusNode, ConsensusProtocol, ProposalStatus,
    ProposalType, Vote,
};
use mir_runtime::crdt::NodeId;
use std::thread;
use std::time::Duration;

#[test]
fn test_consensus_based_hmr_coordination() {
    // Test a single node's consensus behavior for HMR coordination
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes to simulate a cluster
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(2),
            "localhost:8081".to_string(),
        ))
        .unwrap();
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(3),
            "localhost:8082".to_string(),
        ))
        .unwrap();

    // Verify cluster setup
    assert_eq!(consensus.cluster_size(), 3);
    assert!(consensus.has_quorum());

    // Propose an HMR update
    let update_data = b"module_update_v1.0.1".to_vec();
    let proposal_id = consensus
        .propose(ProposalType::UpdateCoordination, update_data)
        .unwrap();

    // Verify proposal is in voting state
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );

    // Vote on the proposal
    consensus.vote(proposal_id, Vote::Accept).unwrap();

    // Verify vote was recorded
    let proposal_info = consensus.get_proposal_info(proposal_id).unwrap();
    assert_eq!(proposal_info.accept_count, 1);
}

#[test]
fn test_hmr_consensus_with_node_failure() {
    // Test HMR consensus behavior when nodes fail
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes to create a 5-node cluster
    for i in 2..=5 {
        consensus
            .add_node(ConsensusNode::new(
                NodeId::new(i),
                format!("localhost:808{i}"),
            ))
            .unwrap();
    }

    // Verify initial cluster state
    assert_eq!(consensus.cluster_size(), 5);
    assert!(consensus.has_quorum());

    // Simulate node failures
    consensus.handle_node_failure(NodeId::new(4)).unwrap();
    consensus.handle_node_failure(NodeId::new(5)).unwrap();

    // Should still have quorum with 3 active nodes
    assert_eq!(consensus.get_active_nodes().len(), 3);
    assert!(consensus.has_quorum());

    // Should be able to propose HMR updates
    let proposal_id = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"emergency_update".to_vec(),
        )
        .unwrap();

    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );
}

#[test]
fn test_hmr_concurrent_proposals() {
    // Test concurrent HMR proposals on a single consensus node
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(2),
            "localhost:8081".to_string(),
        ))
        .unwrap();
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(3),
            "localhost:8082".to_string(),
        ))
        .unwrap();

    // Create multiple concurrent HMR proposals
    let proposal1 = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"module_a_update".to_vec(),
        )
        .unwrap();

    let proposal2 = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"module_b_update".to_vec(),
        )
        .unwrap();

    let proposal3 = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"module_c_update".to_vec(),
        )
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
fn test_hmr_update_rejection() {
    // Test HMR update rejection through consensus
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(2),
            "localhost:8081".to_string(),
        ))
        .unwrap();
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(3),
            "localhost:8082".to_string(),
        ))
        .unwrap();

    // Propose an HMR update
    let proposal_id = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"controversial_update".to_vec(),
        )
        .unwrap();

    // Vote to reject the update
    consensus
        .vote(proposal_id, Vote::Reject("Update not safe".to_string()))
        .unwrap();

    // Verify proposal is still in voting state (need majority to reject)
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );

    // Verify the rejection vote was recorded
    let proposal_info = consensus.get_proposal_info(proposal_id).unwrap();
    assert_eq!(proposal_info.reject_count, 1);
}

#[test]
fn test_hmr_rolling_update_coordination() {
    // Test rolling update coordination using consensus
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes to simulate a larger cluster
    for i in 2..=5 {
        consensus
            .add_node(ConsensusNode::new(
                NodeId::new(i),
                format!("localhost:808{i}"),
            ))
            .unwrap();
    }

    // Verify cluster setup
    assert_eq!(consensus.cluster_size(), 5);
    assert!(consensus.has_quorum());

    // Simulate multiple rolling update proposals
    let update_versions = vec![b"v1.0.1".to_vec(), b"v1.0.2".to_vec(), b"v1.0.3".to_vec()];

    let mut proposal_ids = Vec::new();
    for update_data in update_versions {
        let proposal_id = consensus
            .propose(ProposalType::UpdateCoordination, update_data)
            .unwrap();
        proposal_ids.push(proposal_id);
    }

    // All proposals should be in voting state
    for proposal_id in proposal_ids {
        assert_eq!(
            consensus.get_proposal_status(proposal_id),
            Some(ProposalStatus::Voting)
        );
    }
}

#[test]
fn test_hmr_network_partition_handling() {
    // Test HMR behavior during network partitions
    let node_id = NodeId::new(1);
    let mut config = ConsensusConfig::default();
    config.split_brain_detection_enabled = true;
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Create a 5-node cluster
    for i in 2..=5 {
        consensus
            .add_node(ConsensusNode::new(
                NodeId::new(i),
                format!("localhost:808{i}"),
            ))
            .unwrap();
    }

    // Simulate network partition by failing nodes
    consensus.handle_node_failure(NodeId::new(4)).unwrap();
    consensus.handle_node_failure(NodeId::new(5)).unwrap();

    // Should still have quorum with 3 active nodes
    assert_eq!(consensus.get_active_nodes().len(), 3);
    assert!(consensus.has_quorum());

    // Should be able to propose HMR updates
    let proposal_id = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"partition_update".to_vec(),
        )
        .unwrap();

    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Voting)
    );

    // Simulate partition recovery
    consensus.handle_node_rejoin(NodeId::new(4)).unwrap();
    consensus.handle_node_rejoin(NodeId::new(5)).unwrap();

    // All nodes should be active again
    assert_eq!(consensus.get_active_nodes().len(), 5);
    assert!(consensus.has_quorum());
}

#[test]
fn test_hmr_update_timeout_handling() {
    // Test HMR update timeout handling
    let node_id = NodeId::new(1);
    let mut config = ConsensusConfig::default();
    config.proposal_timeout = Duration::from_millis(50);
    let mut consensus = BasicConsensusProtocol::new(node_id, config);

    // Add peer nodes
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(2),
            "localhost:8081".to_string(),
        ))
        .unwrap();
    consensus
        .add_node(ConsensusNode::new(
            NodeId::new(3),
            "localhost:8082".to_string(),
        ))
        .unwrap();

    // Propose an HMR update
    let proposal_id = consensus
        .propose(
            ProposalType::UpdateCoordination,
            b"timeout_test_update".to_vec(),
        )
        .unwrap();

    // Don't vote, let it timeout
    thread::sleep(Duration::from_millis(100));

    // Process proposals - should detect timeout
    let timed_out = consensus.process_proposals().unwrap();
    assert!(timed_out.contains(&proposal_id));

    // Verify proposal timed out
    assert_eq!(
        consensus.get_proposal_status(proposal_id),
        Some(ProposalStatus::Timeout)
    );
}
