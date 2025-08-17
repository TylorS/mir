//! Integration test suite for the distributed HMR runtime
//! 
//! Tests the integration of core runtime components:
//! - Content-addressable storage
//! - State management
//! - Distributed coordination
//! - CRDT operations
//! - Consensus protocols

use mir_runtime::*;
use mir_types::{ContentHash, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex, mpsc};

/// Integration test framework for runtime components
struct RuntimeIntegrationTest {
    storage: InMemoryStore,
    state_manager: StateManager,
    runtime: DistributedRuntime,
    consensus_nodes: Vec<BasicConsensusProtocol>,
}

impl RuntimeIntegrationTest {
    fn new() -> Self {
        Self {
            storage: InMemoryStore::new(),
            state_manager: StateManager::new(),
            runtime: DistributedRuntime::new(),
            consensus_nodes: Vec::new(),
        }
    }
    
    fn setup_consensus_cluster(&mut self, node_count: usize) {
        for i in 1..=node_count {
            let node_id = NodeId::new(i as u64);
            let config = ConsensusConfig::default();
            let mut consensus = BasicConsensusProtocol::new(node_id, config);
            
            // Connect to other nodes
            for j in 1..=node_count {
                if i != j {
                    let peer_id = NodeId::new(j as u64);
                    let peer = ConsensusNode::new(peer_id, format!("localhost:808{}", j));
                    let _ = consensus.add_node(peer);
                }
            }
            
            self.consensus_nodes.push(consensus);
        }
    }
    
    fn test_storage_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing storage integration...");
        
        // Test basic storage operations
        let test_data = b"integration test data";
        let hash = self.storage.store(test_data)?;
        
        let retrieved = self.storage.retrieve(hash).unwrap();
        assert_eq!(retrieved, test_data);
        
        // Test storage with state manager
        let module_hash = ContentHash::new(b"test_module");
        let snapshot = self.state_manager.create_snapshot(module_hash)?;
        
        assert_eq!(snapshot.module_hash, module_hash);
        
        println!("Storage integration test passed");
        Ok(())
    }
    
    fn test_distributed_runtime_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing distributed runtime integration...");
        
        // Test event loop creation
        let loop_id = "test_loop".to_string();
        self.runtime.create_event_loop(loop_id.clone())?;
        
        // Test queue creation
        let queue_id = "test_queue".to_string();
        self.runtime.create_queue(queue_id.clone())?;
        
        // Test task scheduling
        let function_id = FunctionId::new(1);
        let task_id = self.runtime.schedule_task(
            loop_id,
            function_id,
            Duration::from_millis(100)
        )?;
        
        assert!(task_id.value() > 0);
        
        // Test queue operations
        let test_value = Value::String("test message".to_string());
        self.runtime.enqueue_item(queue_id.clone(), test_value.clone())?;
        
        let dequeued = self.runtime.dequeue_item(queue_id)?;
        assert_eq!(dequeued, Some(test_value));
        
        println!("Distributed runtime integration test passed");
        Ok(())
    }
    
    fn test_crdt_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing CRDT integration...");
        
        // Test GCounter
        let mut counter1 = GCounter::new(NodeId::new(1));
        let mut counter2 = GCounter::new(NodeId::new(2));
        
        counter1.increment(10)?;
        counter2.increment(20)?;
        
        counter1.merge(&counter2)?;
        assert_eq!(counter1.value(), 30);
        
        // Test GSet
        let mut set1: GSet<String> = GSet::new();
        let mut set2: GSet<String> = GSet::new();
        
        set1.add("item1".to_string())?;
        set2.add("item2".to_string())?;
        
        set1.merge(&set2)?;
        assert_eq!(set1.size(), 2);
        assert!(set1.contains(&"item1".to_string()));
        assert!(set1.contains(&"item2".to_string()));
        
        // Test PNCounter
        let mut pn_counter1 = PNCounter::new(NodeId::new(1));
        let mut pn_counter2 = PNCounter::new(NodeId::new(2));
        
        pn_counter1.increment(15)?;
        pn_counter2.decrement(5)?;
        
        pn_counter1.merge(&pn_counter2)?;
        assert_eq!(pn_counter1.value(), 10);
        
        println!("CRDT integration test passed");
        Ok(())
    }
    
    fn test_consensus_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing consensus integration...");
        
        if self.consensus_nodes.is_empty() {
            self.setup_consensus_cluster(3);
        }
        
        // Test proposal creation
        let proposer = &mut self.consensus_nodes[0];
        let proposal_data = vec![1, 2, 3, 4, 5];
        
        let proposal_id = proposer.propose(ProposalType::UpdateCoordination, proposal_data)?;
        
        // Test voting
        proposer.vote(proposal_id, Vote::Accept)?;
        
        // Verify proposal status
        let status = proposer.get_proposal_status(proposal_id);
        assert!(status.is_some());
        
        // Test cluster health
        assert!(proposer.has_quorum());
        assert_eq!(proposer.cluster_size(), 3);
        
        println!("Consensus integration test passed");
        Ok(())
    }
    
    fn test_state_management_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing state management integration...");
        
        // Create multiple snapshots
        let mut snapshots = Vec::new();
        for i in 0..10 {
            let module_hash = ContentHash::new(format!("module_{}", i).as_bytes());
            let snapshot = self.state_manager.create_snapshot(module_hash)?;
            snapshots.push(snapshot);
        }
        
        assert_eq!(snapshots.len(), 10);
        
        // Test snapshot restoration
        let snapshot_to_restore = snapshots[5].clone();
        self.state_manager.restore_snapshot(snapshot_to_restore)?;
        
        println!("State management integration test passed");
        Ok(())
    }
    
    fn test_performance_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing performance integration...");
        
        let start = Instant::now();
        
        // Perform mixed operations
        for i in 0..100 {
            // Storage operations
            let data = format!("performance_test_{}", i);
            let _hash = self.storage.store(data.as_bytes())?;
            
            // State operations
            let module_hash = ContentHash::new(format!("perf_module_{}", i).as_bytes());
            let _snapshot = self.state_manager.create_snapshot(module_hash)?;
            
            // CRDT operations
            let mut counter = GCounter::new(NodeId::new(1));
            counter.increment(1)?;
            
            // Runtime operations
            if i % 10 == 0 {
                let queue_id = format!("perf_queue_{}", i);
                self.runtime.create_queue(queue_id)?;
            }
        }
        
        let duration = start.elapsed();
        let ops_per_second = 100.0 / duration.as_secs_f64();
        
        println!("Performance integration test: {:.2} ops/sec", ops_per_second);
        
        // Should maintain reasonable performance
        assert!(ops_per_second > 50.0, "Performance should be above 50 ops/sec");
        
        Ok(())
    }
    
    fn test_concurrent_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing concurrent integration...");
        
        let storage = Arc::new(Mutex::new(InMemoryStore::new()));
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let runtime = Arc::new(Mutex::new(DistributedRuntime::new()));
        
        let thread_count = 5;
        let operations_per_thread = 20;
        let mut handles = vec![];
        let (tx, rx) = mpsc::channel();
        
        for thread_id in 0..thread_count {
            let storage_clone = Arc::clone(&storage);
            let state_manager_clone = Arc::clone(&state_manager);
            let runtime_clone = Arc::clone(&runtime);
            let tx_clone = tx.clone();
            
            let handle = thread::spawn(move || {
                let mut successful_ops = 0;
                
                for i in 0..operations_per_thread {
                    // Storage operation
                    if let Ok(mut store) = storage_clone.try_lock() {
                        let data = format!("thread_{}_{}", thread_id, i);
                        if store.store(data.as_bytes()).is_ok() {
                            successful_ops += 1;
                        }
                    }
                    
                    // State operation
                    if let Ok(mut sm) = state_manager_clone.try_lock() {
                        let module_hash = ContentHash::new(format!("t{}_m{}", thread_id, i).as_bytes());
                        if sm.create_snapshot(module_hash).is_ok() {
                            successful_ops += 1;
                        }
                    }
                    
                    // Runtime operation
                    if let Ok(mut rt) = runtime_clone.try_lock() {
                        let queue_id = format!("t{}_q{}", thread_id, i);
                        if rt.create_queue(queue_id).is_ok() {
                            successful_ops += 1;
                        }
                    }
                    
                    // CRDT operation (no locking needed)
                    let mut counter = GCounter::new(NodeId::new(thread_id as u64));
                    if counter.increment(1).is_ok() {
                        successful_ops += 1;
                    }
                }
                
                tx_clone.send(successful_ops).unwrap();
            });
            
            handles.push(handle);
        }
        
        // Collect results
        drop(tx);
        let mut total_successful = 0;
        for _ in 0..thread_count {
            total_successful += rx.recv().unwrap();
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let expected_ops = thread_count * operations_per_thread * 4; // 4 operations per iteration
        let success_rate = total_successful as f64 / expected_ops as f64;
        
        println!("Concurrent integration test: {}/{} operations successful ({:.1}%)", 
                total_successful, expected_ops, success_rate * 100.0);
        
        // Should have reasonable success rate under concurrent load
        assert!(success_rate > 0.7, "Success rate should be above 70% under concurrent load");
        
        Ok(())
    }
    
    fn test_error_handling_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing error handling integration...");
        
        // Test storage error handling
        let invalid_hash = ContentHash::new(b"nonexistent");
        let result = self.storage.retrieve(invalid_hash);
        assert!(result.is_none(), "Should return None for nonexistent hash");
        
        // Test consensus error handling
        if !self.consensus_nodes.is_empty() {
            let consensus = &mut self.consensus_nodes[0];
            
            // Test invalid proposal
            let invalid_proposal_id = ProposalId::new(999999);
            let vote_result = consensus.vote(invalid_proposal_id, Vote::Accept);
            assert!(vote_result.is_err(), "Should error on invalid proposal ID");
        }
        
        // Test CRDT error handling
        let mut counter = GCounter::new(NodeId::new(1));
        // GCounter increment should not fail under normal circumstances
        assert!(counter.increment(1).is_ok());
        
        println!("Error handling integration test passed");
        Ok(())
    }
    
    fn run_comprehensive_test_suite(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running comprehensive integration test suite...");
        
        self.test_storage_integration()?;
        self.test_distributed_runtime_integration()?;
        self.test_crdt_integration()?;
        self.test_consensus_integration()?;
        self.test_state_management_integration()?;
        self.test_performance_integration()?;
        self.test_concurrent_integration()?;
        self.test_error_handling_integration()?;
        
        println!("All integration tests passed successfully!");
        Ok(())
    }
}

// Test cases

#[test]
fn test_storage_and_state_integration() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_storage_integration()
        .expect("Storage integration test should pass");
    test_framework.test_state_management_integration()
        .expect("State management integration test should pass");
}

#[test]
fn test_distributed_runtime_integration() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_distributed_runtime_integration()
        .expect("Distributed runtime integration test should pass");
}

#[test]
fn test_crdt_operations_integration() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_crdt_integration()
        .expect("CRDT integration test should pass");
}

#[test]
fn test_consensus_protocol_integration() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_consensus_integration()
        .expect("Consensus integration test should pass");
}

#[test]
fn test_performance_under_load() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_performance_integration()
        .expect("Performance integration test should pass");
}

#[test]
fn test_concurrent_operations() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_concurrent_integration()
        .expect("Concurrent integration test should pass");
}

#[test]
fn test_error_handling() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.test_error_handling_integration()
        .expect("Error handling integration test should pass");
}

#[test]
fn test_comprehensive_integration_suite() {
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.run_comprehensive_test_suite()
        .expect("Comprehensive integration test suite should pass");
}

#[test]
fn test_distributed_coordination_scenario() {
    println!("Testing distributed coordination scenario...");
    
    // Create multiple nodes with consensus
    let mut nodes = Vec::new();
    for i in 1..=5 {
        let node_id = NodeId::new(i);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);
        
        // Connect to other nodes
        for j in 1..=5 {
            if i != j {
                let peer_id = NodeId::new(j);
                let peer = ConsensusNode::new(peer_id, format!("localhost:808{}", j));
                let _ = consensus.add_node(peer);
            }
        }
        
        nodes.push(consensus);
    }
    
    // Test distributed proposal and voting
    let proposal_data = b"distributed_test_proposal".to_vec();
    let proposal_id = nodes[0].propose(ProposalType::UpdateCoordination, proposal_data)
        .expect("Should create proposal");
    
    // Vote from multiple nodes
    for node in nodes.iter_mut().take(3) { // Majority vote
        node.vote(proposal_id, Vote::Accept)
            .expect("Should vote successfully");
    }
    
    // Verify cluster health
    for node in &nodes {
        assert!(node.has_quorum(), "All nodes should have quorum");
        assert_eq!(node.cluster_size(), 5, "All nodes should see full cluster");
    }
    
    println!("Distributed coordination scenario test passed");
}

#[test]
fn test_mixed_workload_scenario() {
    println!("Testing mixed workload scenario...");
    
    let storage = Arc::new(Mutex::new(InMemoryStore::new()));
    let state_manager = Arc::new(Mutex::new(StateManager::new()));
    let runtime = Arc::new(Mutex::new(DistributedRuntime::new()));
    
    let workload_duration = Duration::from_secs(5);
    let start = Instant::now();
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel();
    
    // Spawn workers with different workload patterns
    for worker_id in 0..3 {
        let storage_clone = Arc::clone(&storage);
        let state_manager_clone = Arc::clone(&state_manager);
        let runtime_clone = Arc::clone(&runtime);
        let tx_clone = tx.clone();
        let duration = workload_duration;
        
        let handle = thread::spawn(move || {
            let mut operations = 0;
            let worker_start = Instant::now();
            
            while worker_start.elapsed() < duration {
                match operations % 4 {
                    0 => {
                        // Storage workload
                        if let Ok(mut store) = storage_clone.try_lock() {
                            let data = format!("worker_{}_op_{}", worker_id, operations);
                            if store.store(data.as_bytes()).is_ok() {
                                operations += 1;
                            }
                        }
                    }
                    1 => {
                        // State management workload
                        if let Ok(mut sm) = state_manager_clone.try_lock() {
                            let module_hash = ContentHash::new(format!("w{}_m{}", worker_id, operations).as_bytes());
                            if sm.create_snapshot(module_hash).is_ok() {
                                operations += 1;
                            }
                        }
                    }
                    2 => {
                        // Runtime workload
                        if let Ok(mut rt) = runtime_clone.try_lock() {
                            let queue_id = format!("w{}_q{}", worker_id, operations);
                            if rt.create_queue(queue_id).is_ok() {
                                operations += 1;
                            }
                        }
                    }
                    3 => {
                        // CRDT workload
                        let mut counter = GCounter::new(NodeId::new(worker_id as u64));
                        if counter.increment(1).is_ok() {
                            operations += 1;
                        }
                    }
                    _ => unreachable!()
                }
                
                // Small delay to prevent overwhelming
                thread::sleep(Duration::from_micros(100));
            }
            
            tx_clone.send(operations).unwrap();
        });
        
        handles.push(handle);
    }
    
    // Collect results
    drop(tx);
    let mut total_operations = 0;
    for _ in 0..3 {
        total_operations += rx.recv().unwrap();
    }
    
    // Wait for all workers
    for handle in handles {
        handle.join().unwrap();
    }
    
    let actual_duration = start.elapsed();
    let ops_per_second = total_operations as f64 / actual_duration.as_secs_f64();
    
    println!("Mixed workload scenario: {} operations in {}s ({:.1} ops/sec)", 
            total_operations, actual_duration.as_secs(), ops_per_second);
    
    // Should maintain reasonable throughput with mixed workload
    assert!(ops_per_second > 100.0, "Should maintain good throughput with mixed workload");
    assert!(total_operations > 500, "Should complete substantial number of operations");
}

#[test]
fn test_system_stability_scenario() {
    println!("Testing system stability scenario...");
    
    let mut test_framework = RuntimeIntegrationTest::new();
    test_framework.setup_consensus_cluster(3);
    
    let stability_duration = Duration::from_secs(10);
    let start = Instant::now();
    let mut operation_count = 0;
    let mut error_count = 0;
    
    while start.elapsed() < stability_duration {
        operation_count += 1;
        
        // Perform various operations
        match operation_count % 5 {
            0 => {
                let data = format!("stability_test_{}", operation_count);
                if test_framework.storage.store(data.as_bytes()).is_err() {
                    error_count += 1;
                }
            }
            1 => {
                let module_hash = ContentHash::new(format!("stability_module_{}", operation_count).as_bytes());
                if test_framework.state_manager.create_snapshot(module_hash).is_err() {
                    error_count += 1;
                }
            }
            2 => {
                let queue_id = format!("stability_queue_{}", operation_count);
                if test_framework.runtime.create_queue(queue_id).is_err() {
                    error_count += 1;
                }
            }
            3 => {
                let mut counter = GCounter::new(NodeId::new(1));
                if counter.increment(1).is_err() {
                    error_count += 1;
                }
            }
            4 => {
                if !test_framework.consensus_nodes.is_empty() {
                    let proposal_data = vec![operation_count as u8];
                    if test_framework.consensus_nodes[0].propose(ProposalType::UpdateCoordination, proposal_data).is_err() {
                        error_count += 1;
                    }
                }
            }
            _ => unreachable!()
        }
        
        // Small delay
        if operation_count % 100 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }
    
    let actual_duration = start.elapsed();
    let ops_per_second = operation_count as f64 / actual_duration.as_secs_f64();
    let error_rate = error_count as f64 / operation_count as f64;
    
    println!("System stability scenario: {} operations in {}s ({:.1} ops/sec, {:.2}% errors)", 
            operation_count, actual_duration.as_secs(), ops_per_second, error_rate * 100.0);
    
    // System should remain stable over time
    assert!(error_rate < 0.05, "Error rate should be under 5% during stability test");
    assert!(ops_per_second > 200.0, "Should maintain good throughput during stability test");
    assert!(operation_count > 1000, "Should complete substantial number of operations");
}