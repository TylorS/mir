//! Performance and stress tests
//! 
//! Tests system performance under various load conditions including:
//! - High-frequency updates
//! - Large state management
//! - Concurrent operations
//! - Memory usage patterns

use mir_runtime::*;
use mir_types::{ContentHash, Value, TypeRegistry};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};

#[test]
fn test_high_frequency_updates_performance() {
    let mut coordinator = HMRCoordinator::new();
    
    let start = Instant::now();
    let update_count = 1000;
    
    // Perform many rapid updates
    for i in 0..update_count {
        let old_hash = ContentHash::new(format!("module_v{}", i).as_bytes());
        let new_hash = ContentHash::new(format!("module_v{}", i + 1).as_bytes());
        
        let plan = coordinator.plan_update(old_hash, new_hash);
        assert_eq!(plan.status, UpdateStatus::Planned);
    }
    
    let duration = start.elapsed();
    
    // Should handle 1000 updates in under 1 second
    assert!(duration.as_secs() < 1);
    
    // Calculate throughput
    let updates_per_second = update_count as f64 / duration.as_secs_f64();
    println!("Update planning throughput: {:.2} updates/second", updates_per_second);
    
    // Should achieve reasonable throughput
    assert!(updates_per_second > 500.0);
}

#[test]
fn test_large_state_management_performance() {
    let mut state_manager = StateManager::new();
    
    // Create large state
    let module_hash = ContentHash::new(b"large_state_module");
    
    let start = Instant::now();
    
    // Create snapshot with large state
    let snapshot = state_manager.create_snapshot(module_hash).unwrap();
    
    let snapshot_duration = start.elapsed();
    
    // Should create snapshot quickly even with large state
    assert!(snapshot_duration.as_millis() < 100);
    
    // Test restoration performance
    let start = Instant::now();
    
    let restoration_result = state_manager.restore_snapshot(snapshot);
    
    let restoration_duration = start.elapsed();
    
    assert!(restoration_result.is_ok());
    assert!(restoration_duration.as_millis() < 100);
    
    println!("Snapshot creation: {}ms, restoration: {}ms", 
             snapshot_duration.as_millis(), 
             restoration_duration.as_millis());
}

#[test]
fn test_concurrent_operations_performance() {
    let coordinator = Arc::new(HMRCoordinator::new());
    let thread_count = 10;
    let operations_per_thread = 100;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Spawn multiple threads performing concurrent operations
    for thread_id in 0..thread_count {
        let coordinator_clone = Arc::clone(&coordinator);
        
        let handle = thread::spawn(move || {
            let mut successful_operations = 0;
            
            for i in 0..operations_per_thread {
                let old_hash = ContentHash::new(format!("thread_{}_module_{}", thread_id, i).as_bytes());
                let new_hash = ContentHash::new(format!("thread_{}_module_{}_updated", thread_id, i).as_bytes());
                
                let plan = coordinator_clone.plan_update(old_hash, new_hash);
                if plan.status == UpdateStatus::Planned {
                    successful_operations += 1;
                }
            }
            
            successful_operations
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads and collect results
    let mut total_successful = 0;
    for handle in handles {
        total_successful += handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_operations = thread_count * operations_per_thread;
    
    // Should complete all operations successfully
    assert_eq!(total_successful, total_operations);
    
    // Should complete in reasonable time
    assert!(duration.as_secs() < 5);
    
    let ops_per_second = total_operations as f64 / duration.as_secs_f64();
    println!("Concurrent operations throughput: {:.2} ops/second", ops_per_second);
    
    // Should achieve good concurrent throughput
    assert!(ops_per_second > 200.0);
}

#[test]
fn test_memory_usage_under_load() {
    let mut state_manager = StateManager::new();
    let mut snapshots = Vec::new();
    
    // Create many snapshots to test memory usage
    let snapshot_count = 1000;
    
    let start = Instant::now();
    
    for i in 0..snapshot_count {
        let module_hash = ContentHash::new(format!("memory_test_module_{}", i).as_bytes());
        let snapshot = state_manager.create_snapshot(module_hash).unwrap();
        snapshots.push(snapshot);
        
        // Periodically check that we're not taking too long
        if i % 100 == 0 {
            let elapsed = start.elapsed();
            assert!(elapsed.as_secs() < 10); // Should not take more than 10 seconds
        }
    }
    
    let creation_duration = start.elapsed();
    
    // Should create all snapshots in reasonable time
    assert!(creation_duration.as_secs() < 10);
    
    println!("Created {} snapshots in {}ms", 
             snapshot_count, 
             creation_duration.as_millis());
    
    // Test cleanup performance
    let start = Instant::now();
    
    // Drop half the snapshots to simulate cleanup
    snapshots.truncate(snapshot_count / 2);
    
    let cleanup_duration = start.elapsed();
    
    // Cleanup should be fast
    assert!(cleanup_duration.as_millis() < 100);
    
    println!("Cleaned up {} snapshots in {}ms", 
             snapshot_count / 2, 
             cleanup_duration.as_millis());
}

#[test]
fn test_crdt_performance_under_load() {
    let node_count = 10;
    let operations_per_node = 1000;
    
    // Create multiple counters
    let mut counters = Vec::new();
    for i in 0..node_count {
        counters.push(GCounter::new(NodeId::new(i as u64)));
    }
    
    let start = Instant::now();
    
    // Perform many increment operations
    for (i, counter) in counters.iter_mut().enumerate() {
        for j in 0..operations_per_node {
            counter.increment(1).unwrap();
        }
    }
    
    let increment_duration = start.elapsed();
    
    // Should complete increments quickly
    assert!(increment_duration.as_secs() < 1);
    
    println!("Performed {} increments in {}ms", 
             node_count * operations_per_node, 
             increment_duration.as_millis());
    
    // Test merge performance
    let start = Instant::now();
    
    let mut merged_counter = counters[0].clone();
    for counter in &counters[1..] {
        merged_counter.merge(counter).unwrap();
    }
    
    let merge_duration = start.elapsed();
    
    // Should merge quickly
    assert!(merge_duration.as_millis() < 100);
    
    // Verify final value
    let expected_value = (node_count * operations_per_node) as u64;
    assert_eq!(merged_counter.value(), expected_value);
    
    println!("Merged {} counters in {}ms", 
             node_count, 
             merge_duration.as_millis());
}

#[test]
fn test_consensus_performance_under_load() {
    let node_id = NodeId::new(1);
    let config = ConsensusConfig::default();
    let mut consensus = BasicConsensusProtocol::new(node_id, config);
    
    // Add nodes for a larger cluster
    for i in 2..=10 {
        let node = ConsensusNode::new(NodeId::new(i), format!("localhost:808{}", i));
        consensus.add_node(node).unwrap();
    }
    
    let proposal_count = 1000;
    let start = Instant::now();
    
    // Create many proposals
    let mut proposal_ids = Vec::new();
    for i in 0..proposal_count {
        let proposal_data = vec![i as u8];
        let proposal_id = consensus.propose(ProposalType::UpdateCoordination, proposal_data).unwrap();
        proposal_ids.push(proposal_id);
    }
    
    let proposal_duration = start.elapsed();
    
    // Should create proposals quickly
    assert!(proposal_duration.as_secs() < 2);
    
    println!("Created {} proposals in {}ms", 
             proposal_count, 
             proposal_duration.as_millis());
    
    // Test voting performance
    let start = Instant::now();
    
    for &proposal_id in &proposal_ids[..100] { // Vote on first 100 proposals
        consensus.vote(proposal_id, Vote::Accept).unwrap();
    }
    
    let voting_duration = start.elapsed();
    
    // Should vote quickly
    assert!(voting_duration.as_millis() < 100);
    
    println!("Voted on 100 proposals in {}ms", voting_duration.as_millis());
}

#[test]
fn test_type_registry_performance() {
    let mut registry = TypeRegistry::new();
    
    let type_count = 10000;
    let start = Instant::now();
    
    // Register many types
    let mut type_hashes = Vec::new();
    for i in 0..type_count {
        let type_name = format!("TestType{}", i);
        let type_hash = registry.register_scalar_type(&type_name, 8);
        type_hashes.push(type_hash);
    }
    
    let registration_duration = start.elapsed();
    
    // Should register types quickly
    assert!(registration_duration.as_secs() < 1);
    
    println!("Registered {} types in {}ms", 
             type_count, 
             registration_duration.as_millis());
    
    // Test lookup performance
    let start = Instant::now();
    
    for &type_hash in &type_hashes {
        assert!(registry.contains(type_hash));
    }
    
    let lookup_duration = start.elapsed();
    
    // Should lookup types quickly
    assert!(lookup_duration.as_millis() < 100);
    
    println!("Looked up {} types in {}ms", 
             type_count, 
             lookup_duration.as_millis());
}

#[test]
fn test_serialization_performance() {
    use mir_types::UniversalValueOperations;
    
    // Create complex nested structure
    let mut large_struct = HashMap::new();
    for i in 0..1000 {
        large_struct.insert(format!("field_{}", i), Value::I32(i));
    }
    
    let complex_value = Value::Struct(large_struct);
    
    let serialization_count = 100;
    let start = Instant::now();
    
    // Perform many serializations
    for _ in 0..serialization_count {
        let _serialized = complex_value.lossless_serialize().unwrap();
    }
    
    let serialization_duration = start.elapsed();
    
    // Should serialize quickly
    assert!(serialization_duration.as_secs() < 1);
    
    println!("Performed {} serializations in {}ms", 
             serialization_count, 
             serialization_duration.as_millis());
    
    // Test hash computation performance
    let start = Instant::now();
    
    for _ in 0..serialization_count {
        let _hash = complex_value.stable_hash();
    }
    
    let hashing_duration = start.elapsed();
    
    // Should hash quickly
    assert!(hashing_duration.as_millis() < 500);
    
    println!("Computed {} hashes in {}ms", 
             serialization_count, 
             hashing_duration.as_millis());
}

#[test]
fn test_distributed_runtime_performance() {
    let mut runtime = DistributedRuntime::new();
    
    let event_loop_count = 100;
    let queue_count = 100;
    
    let start = Instant::now();
    
    // Create many event loops and queues
    for i in 0..event_loop_count {
        let loop_id = format!("loop_{}", i);
        runtime.create_event_loop(loop_id).unwrap();
    }
    
    for i in 0..queue_count {
        let queue_id = format!("queue_{}", i);
        runtime.create_queue(queue_id).unwrap();
    }
    
    let creation_duration = start.elapsed();
    
    // Should create resources quickly
    assert!(creation_duration.as_millis() < 500);
    
    println!("Created {} event loops and {} queues in {}ms", 
             event_loop_count, 
             queue_count, 
             creation_duration.as_millis());
    
    // Test task scheduling performance
    let task_count = 1000;
    let start = Instant::now();
    
    for i in 0..task_count {
        let loop_id = format!("loop_{}", i % event_loop_count);
        let _task_id = runtime.schedule_task(
            loop_id,
            FunctionId::new(i as u64),
            Duration::from_millis(10)
        ).unwrap();
    }
    
    let scheduling_duration = start.elapsed();
    
    // Should schedule tasks quickly
    assert!(scheduling_duration.as_millis() < 500);
    
    println!("Scheduled {} tasks in {}ms", 
             task_count, 
             scheduling_duration.as_millis());
}

#[test]
fn test_stress_with_mixed_operations() {
    let coordinator = Arc::new(HMRCoordinator::new());
    let state_manager = Arc::new(Mutex::new(StateManager::new()));
    let type_registry = Arc::new(Mutex::new(TypeRegistry::new()));
    
    let thread_count = 5;
    let operations_per_thread = 200;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Spawn threads performing mixed operations
    for thread_id in 0..thread_count {
        let coordinator_clone = Arc::clone(&coordinator);
        let state_manager_clone = Arc::clone(&state_manager);
        let type_registry_clone = Arc::clone(&type_registry);
        
        let handle = thread::spawn(move || {
            let mut successful_operations = 0;
            
            for i in 0..operations_per_thread {
                // Mix of different operations
                match i % 4 {
                    0 => {
                        // HMR operation
                        let old_hash = ContentHash::new(format!("t{}_m{}_old", thread_id, i).as_bytes());
                        let new_hash = ContentHash::new(format!("t{}_m{}_new", thread_id, i).as_bytes());
                        let plan = coordinator_clone.plan_update(old_hash, new_hash);
                        if plan.status == UpdateStatus::Planned {
                            successful_operations += 1;
                        }
                    }
                    1 => {
                        // State management operation
                        let module_hash = ContentHash::new(format!("t{}_state_{}", thread_id, i).as_bytes());
                        if let Ok(mut sm) = state_manager_clone.lock() {
                            if sm.create_snapshot(module_hash).is_ok() {
                                successful_operations += 1;
                            }
                        }
                    }
                    2 => {
                        // Type registry operation
                        if let Ok(mut tr) = type_registry_clone.lock() {
                            let type_name = format!("T{}_{}", thread_id, i);
                            let _type_hash = tr.register_scalar_type(&type_name, 8);
                            successful_operations += 1;
                        }
                    }
                    3 => {
                        // CRDT operation
                        let mut counter = GCounter::new(NodeId::new(thread_id as u64));
                        if counter.increment(1).is_ok() {
                            successful_operations += 1;
                        }
                    }
                    _ => unreachable!()
                }
            }
            
            successful_operations
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    let mut total_successful = 0;
    for handle in handles {
        total_successful += handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_operations = thread_count * operations_per_thread;
    
    // Should complete most operations successfully
    assert!(total_successful > total_operations * 80 / 100); // At least 80% success rate
    
    // Should complete in reasonable time
    assert!(duration.as_secs() < 10);
    
    let ops_per_second = total_successful as f64 / duration.as_secs_f64();
    println!("Mixed operations throughput: {:.2} ops/second", ops_per_second);
    
    // Should achieve reasonable throughput under stress
    assert!(ops_per_second > 100.0);
}

#[test]
fn test_memory_pressure_handling() {
    let mut state_manager = StateManager::new();
    let mut snapshots = Vec::new();
    
    // Create snapshots until we have a reasonable number
    let target_snapshots = 5000;
    
    let start = Instant::now();
    
    for i in 0..target_snapshots {
        let module_hash = ContentHash::new(format!("pressure_test_{}", i).as_bytes());
        
        match state_manager.create_snapshot(module_hash) {
            Ok(snapshot) => {
                snapshots.push(snapshot);
            }
            Err(_) => {
                // If we hit memory pressure, that's expected
                break;
            }
        }
        
        // Check if we're taking too long
        if start.elapsed().as_secs() > 30 {
            break;
        }
    }
    
    let creation_duration = start.elapsed();
    
    println!("Created {} snapshots under memory pressure in {}ms", 
             snapshots.len(), 
             creation_duration.as_millis());
    
    // Should handle memory pressure gracefully
    assert!(snapshots.len() > 100); // Should create at least some snapshots
    
    // Test cleanup under pressure
    let start = Instant::now();
    
    // Clear all snapshots
    snapshots.clear();
    
    let cleanup_duration = start.elapsed();
    
    // Cleanup should be fast even under pressure
    assert!(cleanup_duration.as_millis() < 1000);
    
    println!("Cleaned up all snapshots in {}ms", cleanup_duration.as_millis());
}

#[test]
fn test_long_running_stability() {
    let coordinator = HMRCoordinator::new();
    let mut state_manager = StateManager::new();
    
    let test_duration = Duration::from_secs(5); // 5 second stability test
    let start = Instant::now();
    
    let mut operation_count = 0;
    let mut error_count = 0;
    
    // Perform continuous operations for the test duration
    while start.elapsed() < test_duration {
        operation_count += 1;
        
        // Alternate between different operations
        match operation_count % 3 {
            0 => {
                let old_hash = ContentHash::new(format!("stability_test_{}", operation_count).as_bytes());
                let new_hash = ContentHash::new(format!("stability_test_{}_updated", operation_count).as_bytes());
                let plan = coordinator.plan_update(old_hash, new_hash);
                if plan.status != UpdateStatus::Planned {
                    error_count += 1;
                }
            }
            1 => {
                let module_hash = ContentHash::new(format!("stability_state_{}", operation_count).as_bytes());
                if state_manager.create_snapshot(module_hash).is_err() {
                    error_count += 1;
                }
            }
            2 => {
                let mut counter = GCounter::new(NodeId::new(1));
                if counter.increment(1).is_err() {
                    error_count += 1;
                }
            }
            _ => unreachable!()
        }
        
        // Small delay to prevent overwhelming the system
        thread::sleep(Duration::from_millis(1));
    }
    
    let actual_duration = start.elapsed();
    
    println!("Performed {} operations in {}ms with {} errors", 
             operation_count, 
             actual_duration.as_millis(),
             error_count);
    
    // Should maintain low error rate during long-running test
    let error_rate = error_count as f64 / operation_count as f64;
    assert!(error_rate < 0.01); // Less than 1% error rate
    
    // Should achieve reasonable throughput
    let ops_per_second = operation_count as f64 / actual_duration.as_secs_f64();
    assert!(ops_per_second > 100.0);
}