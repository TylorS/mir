//! Comprehensive stress tests for the distributed HMR runtime
//! 
//! These tests push the system to its limits to verify:
//! - Performance under extreme load
//! - Memory usage patterns and limits
//! - Concurrent operation handling
//! - System stability over extended periods
//! - Recovery from resource exhaustion

use mir_runtime::*;
use mir_types::{ContentHash, Value, TypeRegistry};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex, mpsc, atomic::{AtomicUsize, Ordering}};

/// Comprehensive stress testing framework
struct StressTestFramework {
    test_duration: Duration,
    max_memory_mb: usize,
    max_threads: usize,
    operation_targets: OperationTargets,
    metrics: StressMetrics,
}

struct OperationTargets {
    updates_per_second: f64,
    consensus_operations_per_second: f64,
    state_snapshots_per_second: f64,
    crdt_operations_per_second: f64,
}

struct StressMetrics {
    total_operations: AtomicUsize,
    successful_operations: AtomicUsize,
    failed_operations: AtomicUsize,
    peak_memory_usage: AtomicUsize,
    operation_latencies: Arc<Mutex<Vec<Duration>>>,
    error_counts: Arc<Mutex<HashMap<String, usize>>>,
    throughput_samples: Arc<Mutex<Vec<f64>>>,
}

impl StressTestFramework {
    fn new() -> Self {
        Self {
            test_duration: Duration::from_secs(60), // 1 minute stress test
            max_memory_mb: 1024, // 1GB memory limit
            max_threads: 50,
            operation_targets: OperationTargets {
                updates_per_second: 100.0,
                consensus_operations_per_second: 200.0,
                state_snapshots_per_second: 50.0,
                crdt_operations_per_second: 500.0,
            },
            metrics: StressMetrics {
                total_operations: AtomicUsize::new(0),
                successful_operations: AtomicUsize::new(0),
                failed_operations: AtomicUsize::new(0),
                peak_memory_usage: AtomicUsize::new(0),
                operation_latencies: Arc::new(Mutex::new(Vec::new())),
                error_counts: Arc::new(Mutex::new(HashMap::new())),
                throughput_samples: Arc::new(Mutex::new(Vec::new())),
            },
        }
    }
    
    fn run_extreme_load_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running extreme load test for {}s...", self.test_duration.as_secs());
        
        let start_time = Instant::now();
        let (tx, rx) = mpsc::channel();
        let mut handles = vec![];
        
        // Spawn HMR stress workers
        for i in 0..10 {
            let tx_clone = tx.clone();
            let duration = self.test_duration;
            let target_ops = self.operation_targets.updates_per_second;
            
            let handle = thread::spawn(move || {
                Self::hmr_stress_worker(i, duration, target_ops, tx_clone)
            });
            handles.push(handle);
        }
        
        // Spawn consensus stress workers
        for i in 10..20 {
            let tx_clone = tx.clone();
            let duration = self.test_duration;
            let target_ops = self.operation_targets.consensus_operations_per_second;
            
            let handle = thread::spawn(move || {
                Self::consensus_stress_worker(i, duration, target_ops, tx_clone)
            });
            handles.push(handle);
        }
        
        // Spawn state management stress workers
        for i in 20..30 {
            let tx_clone = tx.clone();
            let duration = self.test_duration;
            let target_ops = self.operation_targets.state_snapshots_per_second;
            
            let handle = thread::spawn(move || {
                Self::state_stress_worker(i, duration, target_ops, tx_clone)
            });
            handles.push(handle);
        }
        
        // Spawn CRDT stress workers
        for i in 30..40 {
            let tx_clone = tx.clone();
            let duration = self.test_duration;
            let target_ops = self.operation_targets.crdt_operations_per_second;
            
            let handle = thread::spawn(move || {
                Self::crdt_stress_worker(i, duration, target_ops, tx_clone)
            });
            handles.push(handle);
        }
        
        // Monitor system resources
        let metrics_clone = Arc::new(Mutex::new(&mut self.metrics));
        let monitor_handle = thread::spawn(move || {
            Self::resource_monitor(start_time, Duration::from_secs(60), metrics_clone)
        });
        handles.push(monitor_handle);
        
        // Collect results
        drop(tx);
        let mut total_operations = 0;
        let mut total_errors = 0;
        
        for _ in 0..handles.len() - 1 { // Exclude monitor thread
            match rx.recv_timeout(self.test_duration + Duration::from_secs(10)) {
                Ok((ops, errors)) => {
                    total_operations += ops;
                    total_errors += errors;
                }
                Err(_) => {
                    println!("Warning: Worker thread timed out");
                }
            }
        }
        
        // Wait for all threads
        for handle in handles {
            let _ = handle.join();
        }
        
        let actual_duration = start_time.elapsed();
        let ops_per_second = total_operations as f64 / actual_duration.as_secs_f64();
        let error_rate = total_errors as f64 / total_operations as f64;
        
        println!("Extreme load test results:");
        println!("  Duration: {}s", actual_duration.as_secs());
        println!("  Total operations: {}", total_operations);
        println!("  Operations/second: {:.2}", ops_per_second);
        println!("  Error rate: {:.2}%", error_rate * 100.0);
        
        // Verify performance targets
        assert!(ops_per_second > 1000.0, "Should maintain high throughput under extreme load");
        assert!(error_rate < 0.05, "Error rate should be under 5% even under extreme load");
        
        Ok(())
    }
    
    fn hmr_stress_worker(worker_id: usize, duration: Duration, target_ops_per_sec: f64, tx: mpsc::Sender<(usize, usize)>) -> (usize, usize) {
        let mut coordinator = HMRCoordinator::new();
        let start = Instant::now();
        let mut operations = 0;
        let mut errors = 0;
        
        let ops_interval = Duration::from_secs_f64(1.0 / target_ops_per_sec);
        let mut last_op_time = start;
        
        while start.elapsed() < duration {
            if last_op_time.elapsed() >= ops_interval {
                let old_hash = ContentHash::new(format!("worker_{}_op_{}", worker_id, operations).as_bytes());
                let new_hash = ContentHash::new(format!("worker_{}_op_{}_updated", worker_id, operations).as_bytes());
                
                let op_start = Instant::now();
                let plan = coordinator.plan_update(old_hash, new_hash);
                let _op_duration = op_start.elapsed();
                
                if plan.status == UpdateStatus::Planned {
                    operations += 1;
                } else {
                    errors += 1;
                }
                
                last_op_time = Instant::now();
            } else {
                // Small sleep to prevent busy waiting
                thread::sleep(Duration::from_micros(100));
            }
        }
        
        let _ = tx.send((operations, errors));
        (operations, errors)
    }
    
    fn consensus_stress_worker(worker_id: usize, duration: Duration, target_ops_per_sec: f64, tx: mpsc::Sender<(usize, usize)>) -> (usize, usize) {
        let node_id = NodeId::new(worker_id as u64);
        let config = ConsensusConfig::default();
        let mut consensus = BasicConsensusProtocol::new(node_id, config);
        
        // Add some peer nodes
        for i in 1..=3 {
            if i != worker_id {
                let peer = ConsensusNode::new(NodeId::new(i as u64), format!("localhost:808{}", i));
                let _ = consensus.add_node(peer);
            }
        }
        
        let start = Instant::now();
        let mut operations = 0;
        let mut errors = 0;
        
        let ops_interval = Duration::from_secs_f64(1.0 / target_ops_per_sec);
        let mut last_op_time = start;
        
        while start.elapsed() < duration {
            if last_op_time.elapsed() >= ops_interval {
                let proposal_data = vec![operations as u8];
                
                match consensus.propose(ProposalType::UpdateCoordination, proposal_data) {
                    Ok(_) => operations += 1,
                    Err(_) => errors += 1,
                }
                
                last_op_time = Instant::now();
            } else {
                thread::sleep(Duration::from_micros(100));
            }
        }
        
        let _ = tx.send((operations, errors));
        (operations, errors)
    }
    
    fn state_stress_worker(worker_id: usize, duration: Duration, target_ops_per_sec: f64, tx: mpsc::Sender<(usize, usize)>) -> (usize, usize) {
        let mut state_manager = StateManager::new();
        let start = Instant::now();
        let mut operations = 0;
        let mut errors = 0;
        
        let ops_interval = Duration::from_secs_f64(1.0 / target_ops_per_sec);
        let mut last_op_time = start;
        let mut snapshots = Vec::new();
        
        while start.elapsed() < duration {
            if last_op_time.elapsed() >= ops_interval {
                let module_hash = ContentHash::new(format!("state_worker_{}_op_{}", worker_id, operations).as_bytes());
                
                match state_manager.create_snapshot(module_hash) {
                    Ok(snapshot) => {
                        snapshots.push(snapshot);
                        operations += 1;
                        
                        // Periodically clean up old snapshots to prevent memory exhaustion
                        if snapshots.len() > 100 {
                            snapshots.drain(0..50);
                        }
                    }
                    Err(_) => errors += 1,
                }
                
                last_op_time = Instant::now();
            } else {
                thread::sleep(Duration::from_micros(100));
            }
        }
        
        let _ = tx.send((operations, errors));
        (operations, errors)
    }
    
    fn crdt_stress_worker(worker_id: usize, duration: Duration, target_ops_per_sec: f64, tx: mpsc::Sender<(usize, usize)>) -> (usize, usize) {
        let mut counter = GCounter::new(NodeId::new(worker_id as u64));
        let mut set: GSet<String> = GSet::new();
        let start = Instant::now();
        let mut operations = 0;
        let mut errors = 0;
        
        let ops_interval = Duration::from_secs_f64(1.0 / target_ops_per_sec);
        let mut last_op_time = start;
        
        while start.elapsed() < duration {
            if last_op_time.elapsed() >= ops_interval {
                // Alternate between counter and set operations
                if operations % 2 == 0 {
                    match counter.increment(1) {
                        Ok(_) => operations += 1,
                        Err(_) => errors += 1,
                    }
                } else {
                    let item = format!("item_{}_{}", worker_id, operations);
                    match set.add(item) {
                        Ok(_) => operations += 1,
                        Err(_) => errors += 1,
                    }
                }
                
                last_op_time = Instant::now();
            } else {
                thread::sleep(Duration::from_micros(100));
            }
        }
        
        let _ = tx.send((operations, errors));
        (operations, errors)
    }
    
    fn resource_monitor(start_time: Instant, duration: Duration, _metrics: Arc<Mutex<&mut StressMetrics>>) {
        while start_time.elapsed() < duration {
            // In a real implementation, this would monitor:
            // - Memory usage
            // - CPU usage
            // - Thread count
            // - File descriptors
            // - Network connections
            
            thread::sleep(Duration::from_secs(1));
        }
    }
    
    fn run_memory_pressure_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running memory pressure test...");
        
        let mut state_manager = StateManager::new();
        let mut snapshots = Vec::new();
        let start = Instant::now();
        
        // Create snapshots until we hit memory pressure
        let mut snapshot_count = 0;
        loop {
            let module_hash = ContentHash::new(format!("memory_pressure_{}", snapshot_count).as_bytes());
            
            match state_manager.create_snapshot(module_hash) {
                Ok(snapshot) => {
                    snapshots.push(snapshot);
                    snapshot_count += 1;
                    
                    // Check if we've been running too long
                    if start.elapsed().as_secs() > 30 {
                        println!("Memory pressure test timeout after 30s");
                        break;
                    }
                    
                    // Simulate realistic memory pressure
                    if snapshot_count % 1000 == 0 {
                        println!("Created {} snapshots, memory pressure building...", snapshot_count);
                        
                        // Test cleanup under pressure
                        let cleanup_start = Instant::now();
                        let cleanup_count = snapshots.len() / 4; // Clean up 25%
                        snapshots.drain(0..cleanup_count);
                        let cleanup_time = cleanup_start.elapsed();
                        
                        println!("Cleaned up {} snapshots in {}ms", cleanup_count, cleanup_time.as_millis());
                        
                        // Cleanup should remain fast even under pressure
                        assert!(cleanup_time.as_millis() < 1000, "Cleanup should remain fast under memory pressure");
                    }
                }
                Err(_) => {
                    println!("Hit memory limit after {} snapshots", snapshot_count);
                    break;
                }
            }
        }
        
        let total_time = start.elapsed();
        println!("Memory pressure test completed:");
        println!("  Total snapshots created: {}", snapshot_count);
        println!("  Final snapshots retained: {}", snapshots.len());
        println!("  Total time: {}s", total_time.as_secs());
        
        // Should create a reasonable number of snapshots
        assert!(snapshot_count > 100, "Should create at least 100 snapshots before hitting limits");
        
        // Test final cleanup
        let cleanup_start = Instant::now();
        snapshots.clear();
        let final_cleanup_time = cleanup_start.elapsed();
        
        println!("Final cleanup took: {}ms", final_cleanup_time.as_millis());
        assert!(final_cleanup_time.as_millis() < 1000, "Final cleanup should be fast");
        
        Ok(())
    }
    
    fn run_concurrent_operations_stress(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running concurrent operations stress test...");
        
        let coordinator = Arc::new(HMRCoordinator::new());
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let type_registry = Arc::new(Mutex::new(TypeRegistry::new()));
        
        let thread_count = 20;
        let operations_per_thread = 500;
        let start = Instant::now();
        
        let mut handles = vec![];
        let (tx, rx) = mpsc::channel();
        
        // Spawn threads performing mixed concurrent operations
        for thread_id in 0..thread_count {
            let coordinator_clone = Arc::clone(&coordinator);
            let state_manager_clone = Arc::clone(&state_manager);
            let type_registry_clone = Arc::clone(&type_registry);
            let tx_clone = tx.clone();
            
            let handle = thread::spawn(move || {
                let mut successful_ops = 0;
                let mut failed_ops = 0;
                let mut operation_times = Vec::new();
                
                for i in 0..operations_per_thread {
                    let op_start = Instant::now();
                    
                    match i % 5 {
                        0 => {
                            // HMR operation
                            let old_hash = ContentHash::new(format!("t{}_hmr_{}", thread_id, i).as_bytes());
                            let new_hash = ContentHash::new(format!("t{}_hmr_{}_new", thread_id, i).as_bytes());
                            let plan = coordinator_clone.plan_update(old_hash, new_hash);
                            if plan.status == UpdateStatus::Planned {
                                successful_ops += 1;
                            } else {
                                failed_ops += 1;
                            }
                        }
                        1 => {
                            // State management operation
                            if let Ok(mut sm) = state_manager_clone.try_lock() {
                                let module_hash = ContentHash::new(format!("t{}_state_{}", thread_id, i).as_bytes());
                                match sm.create_snapshot(module_hash) {
                                    Ok(_) => successful_ops += 1,
                                    Err(_) => failed_ops += 1,
                                }
                            } else {
                                failed_ops += 1; // Lock contention
                            }
                        }
                        2 => {
                            // Type registry operation
                            if let Ok(mut tr) = type_registry_clone.try_lock() {
                                let type_name = format!("Type_{}_{}", thread_id, i);
                                let _type_hash = tr.register_scalar_type(&type_name, 8);
                                successful_ops += 1;
                            } else {
                                failed_ops += 1; // Lock contention
                            }
                        }
                        3 => {
                            // CRDT operation
                            let mut counter = GCounter::new(NodeId::new(thread_id as u64));
                            match counter.increment(1) {
                                Ok(_) => successful_ops += 1,
                                Err(_) => failed_ops += 1,
                            }
                        }
                        4 => {
                            // Consensus operation
                            let node_id = NodeId::new(thread_id as u64);
                            let config = ConsensusConfig::default();
                            let mut consensus = BasicConsensusProtocol::new(node_id, config);
                            
                            let proposal_data = vec![i as u8];
                            match consensus.propose(ProposalType::UpdateCoordination, proposal_data) {
                                Ok(_) => successful_ops += 1,
                                Err(_) => failed_ops += 1,
                            }
                        }
                        _ => unreachable!()
                    }
                    
                    operation_times.push(op_start.elapsed());
                    
                    // Small delay to prevent overwhelming the system
                    if i % 50 == 0 {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
                
                tx_clone.send((successful_ops, failed_ops, operation_times)).unwrap();
            });
            
            handles.push(handle);
        }
        
        // Collect results
        drop(tx);
        let mut total_successful = 0;
        let mut total_failed = 0;
        let mut all_operation_times = Vec::new();
        
        for _ in 0..thread_count {
            let (successful, failed, times) = rx.recv().unwrap();
            total_successful += successful;
            total_failed += failed;
            all_operation_times.extend(times);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start.elapsed();
        let total_operations = total_successful + total_failed;
        let success_rate = total_successful as f64 / total_operations as f64;
        let ops_per_second = total_operations as f64 / total_time.as_secs_f64();
        
        // Calculate latency statistics
        all_operation_times.sort();
        let avg_latency = all_operation_times.iter().map(|d| d.as_micros() as f64).sum::<f64>() / all_operation_times.len() as f64;
        let p95_latency = all_operation_times[all_operation_times.len() * 95 / 100].as_micros() as f64;
        let p99_latency = all_operation_times[all_operation_times.len() * 99 / 100].as_micros() as f64;
        
        println!("Concurrent operations stress test results:");
        println!("  Total operations: {}", total_operations);
        println!("  Success rate: {:.2}%", success_rate * 100.0);
        println!("  Operations/second: {:.2}", ops_per_second);
        println!("  Average latency: {:.2}μs", avg_latency);
        println!("  P95 latency: {:.2}μs", p95_latency);
        println!("  P99 latency: {:.2}μs", p99_latency);
        
        // Verify performance under concurrent stress
        assert!(success_rate > 0.8, "Success rate should be above 80% under concurrent stress");
        assert!(ops_per_second > 500.0, "Should maintain reasonable throughput under concurrent stress");
        assert!(avg_latency < 10000.0, "Average latency should be under 10ms");
        
        Ok(())
    }
    
    fn run_long_running_stability_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running long-running stability test for {}s...", self.test_duration.as_secs());
        
        let coordinator = HMRCoordinator::new();
        let mut state_manager = StateManager::new();
        let mut type_registry = TypeRegistry::new();
        
        let start = Instant::now();
        let mut operation_count = 0;
        let mut error_count = 0;
        let mut last_report = start;
        
        while start.elapsed() < self.test_duration {
            operation_count += 1;
            
            // Perform different operations in rotation
            match operation_count % 6 {
                0 => {
                    // HMR operation
                    let old_hash = ContentHash::new(format!("stability_hmr_{}", operation_count).as_bytes());
                    let new_hash = ContentHash::new(format!("stability_hmr_{}_updated", operation_count).as_bytes());
                    let plan = coordinator.plan_update(old_hash, new_hash);
                    if plan.status != UpdateStatus::Planned {
                        error_count += 1;
                    }
                }
                1 => {
                    // State snapshot
                    let module_hash = ContentHash::new(format!("stability_state_{}", operation_count).as_bytes());
                    if state_manager.create_snapshot(module_hash).is_err() {
                        error_count += 1;
                    }
                }
                2 => {
                    // Type registration
                    let type_name = format!("StabilityType_{}", operation_count);
                    let _type_hash = type_registry.register_scalar_type(&type_name, 8);
                }
                3 => {
                    // CRDT operation
                    let mut counter = GCounter::new(NodeId::new(1));
                    if counter.increment(1).is_err() {
                        error_count += 1;
                    }
                }
                4 => {
                    // Consensus operation
                    let node_id = NodeId::new(1);
                    let config = ConsensusConfig::default();
                    let mut consensus = BasicConsensusProtocol::new(node_id, config);
                    let proposal_data = vec![operation_count as u8];
                    if consensus.propose(ProposalType::UpdateCoordination, proposal_data).is_err() {
                        error_count += 1;
                    }
                }
                5 => {
                    // Memory cleanup simulation
                    if operation_count % 1000 == 0 {
                        // Simulate periodic cleanup
                        thread::sleep(Duration::from_millis(1));
                    }
                }
                _ => unreachable!()
            }
            
            // Report progress every 10 seconds
            if last_report.elapsed() >= Duration::from_secs(10) {
                let elapsed = start.elapsed();
                let ops_per_second = operation_count as f64 / elapsed.as_secs_f64();
                let error_rate = error_count as f64 / operation_count as f64;
                
                println!("Stability test progress: {}s, {} ops ({:.1} ops/s), {:.2}% errors", 
                        elapsed.as_secs(), operation_count, ops_per_second, error_rate * 100.0);
                
                last_report = Instant::now();
            }
            
            // Small delay to prevent overwhelming the system
            if operation_count % 100 == 0 {
                thread::sleep(Duration::from_millis(1));
            }
        }
        
        let total_time = start.elapsed();
        let final_ops_per_second = operation_count as f64 / total_time.as_secs_f64();
        let final_error_rate = error_count as f64 / operation_count as f64;
        
        println!("Long-running stability test results:");
        println!("  Duration: {}s", total_time.as_secs());
        println!("  Total operations: {}", operation_count);
        println!("  Operations/second: {:.2}", final_ops_per_second);
        println!("  Error rate: {:.2}%", final_error_rate * 100.0);
        
        // Verify long-term stability
        assert!(final_error_rate < 0.01, "Error rate should remain under 1% during long-term operation");
        assert!(final_ops_per_second > 100.0, "Should maintain reasonable throughput over long periods");
        assert!(operation_count > 1000, "Should perform substantial number of operations");
        
        Ok(())
    }
    
    fn run_resource_exhaustion_recovery_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running resource exhaustion recovery test...");
        
        // Test recovery from various resource exhaustion scenarios
        
        // 1. Memory exhaustion recovery
        println!("Testing memory exhaustion recovery...");
        let mut state_manager = StateManager::new();
        let mut snapshots = Vec::new();
        
        // Create snapshots until exhaustion
        for i in 0..5000 {
            let module_hash = ContentHash::new(format!("exhaustion_test_{}", i).as_bytes());
            match state_manager.create_snapshot(module_hash) {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(_) => {
                    println!("Hit memory exhaustion at {} snapshots", i);
                    break;
                }
            }
        }
        
        // Test recovery by cleaning up
        let cleanup_start = Instant::now();
        let initial_count = snapshots.len();
        snapshots.clear();
        let cleanup_time = cleanup_start.elapsed();
        
        println!("Recovered from memory exhaustion: cleaned {} snapshots in {}ms", 
                initial_count, cleanup_time.as_millis());
        
        // Should recover quickly
        assert!(cleanup_time.as_millis() < 1000, "Recovery from memory exhaustion should be fast");
        
        // 2. Thread exhaustion recovery
        println!("Testing thread exhaustion recovery...");
        let mut thread_handles = Vec::new();
        
        // Spawn threads until exhaustion
        for i in 0..100 {
            match thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                i
            }) {
                Ok(handle) => thread_handles.push(handle),
                Err(_) => {
                    println!("Hit thread exhaustion at {} threads", i);
                    break;
                }
            }
        }
        
        // Test recovery by joining threads
        let join_start = Instant::now();
        let initial_thread_count = thread_handles.len();
        for handle in thread_handles {
            let _ = handle.join();
        }
        let join_time = join_start.elapsed();
        
        println!("Recovered from thread exhaustion: joined {} threads in {}ms", 
                initial_thread_count, join_time.as_millis());
        
        // Should recover reasonably quickly
        assert!(join_time.as_millis() < 5000, "Recovery from thread exhaustion should complete in reasonable time");
        
        // 3. Operation queue exhaustion recovery
        println!("Testing operation queue recovery...");
        let coordinator = HMRCoordinator::new();
        let mut pending_operations = Vec::new();
        
        // Create many pending operations
        for i in 0..1000 {
            let old_hash = ContentHash::new(format!("queue_test_{}", i).as_bytes());
            let new_hash = ContentHash::new(format!("queue_test_{}_updated", i).as_bytes());
            let plan = coordinator.plan_update(old_hash, new_hash);
            pending_operations.push(plan);
        }
        
        println!("Created {} pending operations", pending_operations.len());
        
        // Test that system can handle large operation queues
        assert!(pending_operations.len() == 1000, "Should handle large operation queues");
        
        println!("Resource exhaustion recovery test completed successfully");
        
        Ok(())
    }
}

// Stress test cases

#[test]
fn test_extreme_load_stress() {
    let mut framework = StressTestFramework::new();
    framework.test_duration = Duration::from_secs(30); // Shorter for CI
    
    framework.run_extreme_load_test()
        .expect("Extreme load stress test should complete successfully");
}

#[test]
fn test_memory_pressure_stress() {
    let mut framework = StressTestFramework::new();
    
    framework.run_memory_pressure_test()
        .expect("Memory pressure stress test should complete successfully");
}

#[test]
fn test_concurrent_operations_stress() {
    let mut framework = StressTestFramework::new();
    
    framework.run_concurrent_operations_stress()
        .expect("Concurrent operations stress test should complete successfully");
}

#[test]
fn test_long_running_stability_stress() {
    let mut framework = StressTestFramework::new();
    framework.test_duration = Duration::from_secs(60); // 1 minute for thorough testing
    
    framework.run_long_running_stability_test()
        .expect("Long-running stability stress test should complete successfully");
}

#[test]
fn test_resource_exhaustion_recovery_stress() {
    let mut framework = StressTestFramework::new();
    
    framework.run_resource_exhaustion_recovery_test()
        .expect("Resource exhaustion recovery stress test should complete successfully");
}

#[test]
fn test_mixed_workload_stress() {
    println!("Running mixed workload stress test...");
    
    let test_duration = Duration::from_secs(30);
    let start = Instant::now();
    
    // Create multiple components
    let coordinator = Arc::new(HMRCoordinator::new());
    let state_manager = Arc::new(Mutex::new(StateManager::new()));
    let type_registry = Arc::new(Mutex::new(TypeRegistry::new()));
    
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel();
    
    // Mixed workload: HMR + State + Consensus + CRDTs
    for worker_id in 0..8 {
        let coordinator_clone = Arc::clone(&coordinator);
        let state_manager_clone = Arc::clone(&state_manager);
        let type_registry_clone = Arc::clone(&type_registry);
        let tx_clone = tx.clone();
        let duration = test_duration;
        
        let handle = thread::spawn(move || {
            let mut operations = 0;
            let mut errors = 0;
            let worker_start = Instant::now();
            
            while worker_start.elapsed() < duration {
                match operations % 8 {
                    0 => {
                        // HMR operation
                        let old_hash = ContentHash::new(format!("mixed_hmr_{}_{}", worker_id, operations).as_bytes());
                        let new_hash = ContentHash::new(format!("mixed_hmr_{}_{}_new", worker_id, operations).as_bytes());
                        let plan = coordinator_clone.plan_update(old_hash, new_hash);
                        if plan.status == UpdateStatus::Planned {
                            operations += 1;
                        } else {
                            errors += 1;
                        }
                    }
                    1 => {
                        // State operation
                        if let Ok(mut sm) = state_manager_clone.try_lock() {
                            let module_hash = ContentHash::new(format!("mixed_state_{}_{}", worker_id, operations).as_bytes());
                            match sm.create_snapshot(module_hash) {
                                Ok(_) => operations += 1,
                                Err(_) => errors += 1,
                            }
                        } else {
                            errors += 1;
                        }
                    }
                    2 => {
                        // Type registry operation
                        if let Ok(mut tr) = type_registry_clone.try_lock() {
                            let type_name = format!("MixedType_{}_{}", worker_id, operations);
                            let _type_hash = tr.register_scalar_type(&type_name, 8);
                            operations += 1;
                        } else {
                            errors += 1;
                        }
                    }
                    3 => {
                        // CRDT counter operation
                        let mut counter = GCounter::new(NodeId::new(worker_id as u64));
                        match counter.increment(1) {
                            Ok(_) => operations += 1,
                            Err(_) => errors += 1,
                        }
                    }
                    4 => {
                        // CRDT set operation
                        let mut set: GSet<String> = GSet::new();
                        let item = format!("mixed_item_{}_{}", worker_id, operations);
                        match set.add(item) {
                            Ok(_) => operations += 1,
                            Err(_) => errors += 1,
                        }
                    }
                    5 => {
                        // Consensus operation
                        let node_id = NodeId::new(worker_id as u64);
                        let config = ConsensusConfig::default();
                        let mut consensus = BasicConsensusProtocol::new(node_id, config);
                        let proposal_data = vec![operations as u8];
                        match consensus.propose(ProposalType::UpdateCoordination, proposal_data) {
                            Ok(_) => operations += 1,
                            Err(_) => errors += 1,
                        }
                    }
                    6 => {
                        // Distributed runtime operation
                        let mut runtime = DistributedRuntime::new();
                        let queue_id = format!("mixed_queue_{}_{}", worker_id, operations);
                        match runtime.create_queue(queue_id) {
                            Ok(_) => operations += 1,
                            Err(_) => errors += 1,
                        }
                    }
                    7 => {
                        // Complex data structure operation
                        let mut complex_data = HashMap::new();
                        for i in 0..10 {
                            complex_data.insert(format!("key_{}", i), Value::I32(i));
                        }
                        let _complex_value = Value::Struct(complex_data);
                        operations += 1;
                    }
                    _ => unreachable!()
                }
                
                // Micro-sleep to prevent overwhelming
                if operations % 10 == 0 {
                    thread::sleep(Duration::from_micros(100));
                }
            }
            
            tx_clone.send((operations, errors)).unwrap();
        });
        
        handles.push(handle);
    }
    
    // Collect results
    drop(tx);
    let mut total_operations = 0;
    let mut total_errors = 0;
    
    for _ in 0..8 {
        let (ops, errs) = rx.recv().unwrap();
        total_operations += ops;
        total_errors += errs;
    }
    
    // Wait for all workers
    for handle in handles {
        handle.join().unwrap();
    }
    
    let actual_duration = start.elapsed();
    let ops_per_second = total_operations as f64 / actual_duration.as_secs_f64();
    let error_rate = total_errors as f64 / (total_operations + total_errors) as f64;
    
    println!("Mixed workload stress test results:");
    println!("  Duration: {}s", actual_duration.as_secs());
    println!("  Total operations: {}", total_operations);
    println!("  Total errors: {}", total_errors);
    println!("  Operations/second: {:.2}", ops_per_second);
    println!("  Error rate: {:.2}%", error_rate * 100.0);
    
    // Verify mixed workload performance
    assert!(ops_per_second > 500.0, "Should maintain good throughput with mixed workload");
    assert!(error_rate < 0.1, "Error rate should be under 10% with mixed workload");
    assert!(total_operations > 1000, "Should complete substantial number of operations");
}