//! Unit tests for distributed coordination
//! 
//! Tests distributed coordination functionality including:
//! - CRDT operations and merge semantics
//! - Consensus protocol behavior
//! - Distributed HMR coordination
//! - Node failure and recovery scenarios

use mir_runtime::*;
use std::time::Duration;
use std::thread;

#[test]
fn test_gcounter_operations() {
    let node1 = NodeId::new(1);
    let node2 = NodeId::new(2);
    
    let mut counter1 = GCounter::new(node1);
    let mut counter2 = GCounter::new(node2);
    
    // Test increment operations
    counter1.increment(5).unwrap();
    counter2.increment(3).unwrap();
    
    assert_eq!(counter1.value(), 5);
    assert_eq!(counter2.value(), 3);
    
    // Test merge operation
    counter1.merge(&counter2).unwrap();
    assert_eq!(counter1.value(), 8); // 5 + 3
    
    // Test idempotent merge
    counter1.merge(&counter2).unwrap();
    assert_eq!(counter1.value(), 8); // Should remain 8
}

#[test]
fn test_pncounter_operations() {
    let node1 = NodeId::new(1);
    let mut counter = PNCounter::new(node1);
    
    // Test increment and decrement
    counter.increment(10).unwrap();
    assert_eq!(counter.value(), 10);
    
    counter.decrement(3).unwrap();
    assert_eq!(counter.value(), 7);
    
    counter.decrement(15).unwrap();
    assert_eq!(counter.value(), -8);
}

#[test]
fn test_gset_operations() {
    let mut set1: GSet<String> = GSet::new();
    let mut set2: GSet<String> = GSet::new();
    
    // Add elements to both sets
    set1.add("apple".to_string()).unwrap();
    set1.add("banana".to_string()).unwrap();
    
    set2.add("banana".to_string()).unwrap();
    set2.add("cherry".to_string()).unwrap();
    
    // Test contains
    assert!(set1.contains(&"apple".to_string()));
    assert!(set1.contains(&"banana".to_string()));
    assert!(!set1.contains(&"cherry".to_string()));
    
    // Test merge
    set1.merge(&set2).unwrap();
    assert_eq!(set1.size(), 3);
    assert!(set1.contains(&"apple".to_string()));
    assert!(set1.contains(&"banana".to_string()));
    assert!(set1.contains(&"cherry".to_string()));
}

#[test]
fn test_orset_operations() {
    let mut set1: ORSet<String> = ORSet::new();
    let mut set2: ORSet<String> = ORSet::new();
    
    // Add and remove elements
    let tag1 = set1.add("item1".to_string()).unwrap();
    let tag2 = set1.add("item2".to_string()).unwrap();
    
    assert!(set1.contains(&"item1".to_string()));
    assert!(set1.contains(&"item2".to_string()));
    
    // Remove by tag
    set1.remove_tag(tag1).unwrap();
    assert!(!set1.contains(&"item1".to_string()));
    assert!(set1.contains(&"item2".to_string()));
    
    // Test merge with concurrent operations
    set2.add("item3".to_string()).unwrap();
    set1.merge(&set2).unwrap();
    
    assert!(set1.contains(&"item3".to_string()));
    assert_eq!(set1.elements().len(), 2); // item2 and item3
}

#[test]
fn test_lww_register_operations() {
    let node1 = NodeId::new(1);
    let node2 = NodeId::new(2);
    
    let mut register1 = LWWRegister::new("initial".to_string(), LogicalTimestamp::new(1), node1);
    let mut register2 = LWWRegister::new("concurrent".to_string(), LogicalTimestamp::new(2), node2);
    
    // Test basic operations
    assert_eq!(register1.get(), &"initial".to_string());
    assert_eq!(register2.get(), &"concurrent".to_string());
    
    // Test merge - higher timestamp wins
    register1.merge(&register2).unwrap();
    assert_eq!(register1.get(), &"concurrent".to_string());
    assert_eq!(register1.get_timestamp(), LogicalTimestamp::new(2));
}

#[test]
fn test_mv_register_operations() {
    let node1 = NodeId::new(1);
    let node2 = NodeId::new(2);
    
    let mut register1 = MVRegister::new(node1);
    let mut register2 = MVRegister::new(node2);
    
    // Set concurrent values
    register1.set("value1".to_string()).unwrap();
    register2.set("value2".to_string()).unwrap();
    
    // Before merge, each has one value
    assert_eq!(register1.get_values().len(), 1);
    assert_eq!(register2.get_values().len(), 1);
    
    // After merge, should have concurrent values
    register1.merge(&register2).unwrap();
    let concurrent_values = register1.get_concurrent_values();
    assert_eq!(concurrent_values.len(), 2);
    
    // Test conflict resolution
    register1.resolve_conflicts(|values| {
        // Choose lexicographically first value
        values.iter().min().unwrap().clone()
    }).unwrap();
    
    assert_eq!(register1.get_values().len(), 1);
}

#[test]
fn test_consensus_basic_operations() {
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Add nodes to form a cluster
    consensus.add_node(ConsensusNode::new(NodeId::new(2), "localhost:8081".to_string())).unwrap();
    consensus.add_node(ConsensusNode::new(NodeId::new(3), "localhost:8082".to_string())).unwrap();
    
    // Test cluster properties
    assert_eq!(consensus.cluster_size(), 3);
    assert!(consensus.has_quorum());
    
    // Test proposal
    let proposal_id = consensus.propose(ProposalType::UpdateCoordination, vec![1, 2, 3]).unwrap();
    assert_eq!(consensus.get_proposal_status(proposal_id), Some(ProposalStatus::Voting));
    
    // Test voting
    consensus.vote(proposal_id, Vote::Accept).unwrap();
    let proposal_info = consensus.get_proposal_info(proposal_id).unwrap();
    assert_eq!(proposal_info.accept_count, 1);
}

#[test]
fn test_consensus_node_failure_handling() {
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Create 5-node cluster
    for i in 2..=5 {
        consensus.add_node(ConsensusNode::new(NodeId::new(i), format!("localhost:808{}", i))).unwrap();
    }
    
    assert_eq!(consensus.cluster_size(), 5);
    assert!(consensus.has_quorum());
    
    // Simulate node failures
    consensus.handle_node_failure(NodeId::new(4)).unwrap();
    consensus.handle_node_failure(NodeId::new(5)).unwrap();
    
    // Should still have quorum with 3 nodes
    assert_eq!(consensus.get_active_nodes().len(), 3);
    assert!(consensus.has_quorum());
    
    // Test node rejoin
    consensus.handle_node_rejoin(NodeId::new(4)).unwrap();
    assert_eq!(consensus.get_active_nodes().len(), 4);
}

#[test]
fn test_consensus_proposal_timeout() {
    let node_id = NodeId::new(1);
    let mut config = ConsensusConfig::default();
    config.proposal_timeout = Duration::from_millis(50);
    
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Add nodes for quorum
    consensus.add_node(ConsensusNode::new(NodeId::new(2), "localhost:8081".to_string())).unwrap();
    consensus.add_node(ConsensusNode::new(NodeId::new(3), "localhost:8082".to_string())).unwrap();
    
    // Create proposal
    let proposal_id = consensus.propose(ProposalType::UpdateCoordination, vec![1, 2, 3]).unwrap();
    
    // Don't vote, let it timeout
    thread::sleep(Duration::from_millis(100));
    
    // Process proposals should detect timeout
    let timed_out = consensus.process_proposals().unwrap();
    assert!(timed_out.contains(&proposal_id));
    assert_eq!(consensus.get_proposal_status(proposal_id), Some(ProposalStatus::Timeout));
}

#[test]
fn test_distributed_hmr_coordinator() {
    let node_id = NodeId::new(1);
    let mut coordinator = BasicDistributedHMRCoordinator::new(node_id);
    
    // Add peer nodes
    coordinator.add_node(NodeId::new(2)).unwrap();
    coordinator.add_node(NodeId::new(3)).unwrap();
    
    // Test distributed update
    let update_data = b"test_update_v1.0.1".to_vec();
    let update_id = coordinator.coordinate_update(update_data).unwrap();
    
    // Verify update is tracked
    let status = coordinator.get_update_status(update_id).unwrap();
    assert!(matches!(status, DistributedUpdateStatus::InProgress | DistributedUpdateStatus::Completed));
}

#[test]
fn test_distributed_runtime_primitives() {
    let mut runtime = DistributedRuntime::new();
    
    // Test event loop
    let event_loop_id = "test_loop".to_string();
    runtime.create_event_loop(event_loop_id.clone()).unwrap();
    
    let task_id = runtime.schedule_task(
        event_loop_id.clone(),
        FunctionId::new(1),
        Duration::from_millis(100)
    ).unwrap();
    
    assert!(runtime.get_task_info(task_id).is_some());
    
    // Test queue
    let queue_id = "test_queue".to_string();
    runtime.create_queue(queue_id.clone()).unwrap();
    
    let test_item = mir_types::Value::String("test_message".to_string());
    runtime.enqueue_item(queue_id.clone(), test_item.clone()).unwrap();
    
    let dequeued = runtime.dequeue_item(queue_id).unwrap();
    assert_eq!(dequeued, Some(test_item));
}

#[test]
fn test_pub_sub_channel() {
    let mut channel = BasicPubSubChannel::new();
    
    let topic = "test_topic".to_string();
    let message = Message {
        id: MessageId::new(1),
        topic: topic.clone(),
        payload: mir_types::Value::String("Hello, PubSub!".to_string()),
        timestamp: std::time::SystemTime::now(),
    };
    
    // Subscribe to topic
    let subscription_id = channel.subscribe(topic.clone(), FunctionId::new(1)).unwrap();
    
    // Publish message
    let message_id = channel.publish(message.clone()).unwrap();
    
    // Verify subscription received message
    let received_messages = channel.get_messages_for_subscription(subscription_id).unwrap();
    assert!(!received_messages.is_empty());
    
    // Unsubscribe
    channel.unsubscribe(subscription_id).unwrap();
}

#[test]
fn test_distributed_coordination_with_partitions() {
    let node_id = NodeId::new(1);
    let mut config = ConsensusConfig::default();
    config.split_brain_detection_enabled = true;
    
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Create 5-node cluster
    for i in 2..=5 {
        consensus.add_node(ConsensusNode::new(NodeId::new(i), format!("localhost:808{}", i))).unwrap();
    }
    
    // Simulate network partition
    for i in 3..=5 {
        let _ = consensus.handle_node_failure(NodeId::new(i));
    }
    
    // Should detect partition
    let partition = consensus.detect_network_partition().unwrap();
    assert!(partition.is_some());
    
    let partitioned_nodes = partition.unwrap();
    assert_eq!(partitioned_nodes.len(), 3);
}

#[test]
fn test_crdt_concurrent_operations() {
    use std::sync::Arc;
    use std::thread;
    
    let counter = Arc::new(std::sync::Mutex::new(GCounter::new(NodeId::new(1))));
    let mut handles = vec![];
    
    // Spawn multiple threads to increment concurrently
    for i in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut c = counter_clone.lock().unwrap();
            c.increment(i).unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final value
    let final_counter = counter.lock().unwrap();
    assert_eq!(final_counter.value(), (0..10).sum::<u64>());
}

#[test]
fn test_rolling_update_coordination() {
    let node_id = NodeId::new(1);
    let mut coordinator = BasicDistributedHMRCoordinator::new(node_id);
    
    // Add multiple nodes
    for i in 2..=5 {
        coordinator.add_node(NodeId::new(i)).unwrap();
    }
    
    // Test rolling update strategy
    let update_data = b"rolling_update_v2.0.0".to_vec();
    let update_id = coordinator.coordinate_rolling_update(
        update_data,
        RollingUpdateStrategy::OneByOne
    ).unwrap();
    
    // Verify update is in progress
    let status = coordinator.get_update_status(update_id).unwrap();
    assert!(matches!(status, DistributedUpdateStatus::InProgress | DistributedUpdateStatus::Completed));
    
    // Check cluster health during update
    let health = coordinator.get_cluster_health().unwrap();
    assert!(matches!(health.overall, OverallHealth::Healthy | OverallHealth::Degraded));
}

#[test]
fn test_consensus_different_proposal_types() {
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Add nodes
    consensus.add_node(ConsensusNode::new(NodeId::new(2), "localhost:8081".to_string())).unwrap();
    consensus.add_node(ConsensusNode::new(NodeId::new(3), "localhost:8082".to_string())).unwrap();
    
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
        let proposal_id = consensus.propose(proposal_type, vec![i as u8]).unwrap();
        proposal_ids.push(proposal_id);
    }
    
    // All proposals should be in voting state
    for proposal_id in proposal_ids {
        assert_eq!(consensus.get_proposal_status(proposal_id), Some(ProposalStatus::Voting));
    }
}

#[test]
fn test_distributed_system_performance() {
    use std::time::Instant;
    
    let mut runtime = DistributedRuntime::new();
    
    // Create multiple event loops and queues
    let start = Instant::now();
    
    for i in 0..100 {
        let loop_id = format!("loop_{}", i);
        let queue_id = format!("queue_{}", i);
        
        runtime.create_event_loop(loop_id.clone()).unwrap();
        runtime.create_queue(queue_id.clone()).unwrap();
        
        // Schedule a task and enqueue an item
        let _task_id = runtime.schedule_task(
            loop_id,
            FunctionId::new(i),
            Duration::from_millis(10)
        ).unwrap();
        
        let item = mir_types::Value::I32(i as i32);
        runtime.enqueue_item(queue_id, item).unwrap();
    }
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time
    assert!(duration.as_secs() < 1);
}