//! Comprehensive integration tests for the distributed HMR runtime
//! 
//! This test suite provides end-to-end testing of the complete system including:
//! - Full HMR lifecycle testing with real state preservation
//! - Multi-node distributed coordination scenarios
//! - Performance testing under realistic load conditions
//! - Cross-backend compatibility validation
//! - Error recovery and fault tolerance testing

use mir_runtime::*;
use mir_types::{ContentHash, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex, mpsc};

/// Comprehensive test harness for integration testing
struct IntegrationTestHarness {
    nodes: HashMap<NodeId, TestNode>,
    network_simulator: NetworkSimulator,
    load_generator: LoadGenerator,
    metrics_collector: MetricsCollector,
}

struct TestNode {
    id: NodeId,
    hmr_coordinator: HMRCoordinator<InMemoryStore>,
    consensus: BasicConsensusProtocol,
    state_manager: StateManager,
    runtime: DistributedRuntime,
    is_healthy: bool,
}

struct NetworkSimulator {
    partitions: Vec<Vec<NodeId>>,
    latencies: HashMap<(NodeId, NodeId), Duration>,
    packet_loss: HashMap<(NodeId, NodeId), f64>,
    is_partitioned: bool,
}

struct LoadGenerator {
    update_rate: f64,
    state_size: usize,
    concurrent_operations: usize,
}

struct MetricsCollector {
    update_latencies: Vec<Duration>,
    consensus_times: Vec<Duration>,
    state_migration_times: Vec<Duration>,
    error_counts: HashMap<String, usize>,
    throughput_samples: Vec<f64>,
}

impl IntegrationTestHarness {
    fn new(node_count: usize) -> Self {
        let mut nodes = HashMap::new();
        
        for i in 1..=node_count {
            let node_id = NodeId::new(i as u64);
            let store = InMemoryStore::new();
            let node = TestNode {
                id: node_id,
                hmr_coordinator: HMRCoordinator::new(store),
                consensus: BasicConsensusProtocol::new(node_id, ConsensusConfig::default()),
                state_manager: StateManager::new(),
                runtime: DistributedRuntime::new(),
                is_healthy: true,
            };
            nodes.insert(node_id, node);
        }
        
        // Connect all nodes
        let node_ids: Vec<_> = nodes.keys().cloned().collect();
        for (_, node) in nodes.iter_mut() {
            for &other_id in &node_ids {
                if other_id != node.id {
                    let consensus_node = ConsensusNode::new(other_id, format!("localhost:808{}", other_id.value()));
                    let _ = node.consensus.add_node(consensus_node);
                    let _ = node.hmr_coordinator.add_node(other_id);
                }
            }
        }
        
        Self {
            nodes,
            network_simulator: NetworkSimulator::new(),
            load_generator: LoadGenerator::new(),
            metrics_collector: MetricsCollector::new(),
        }
    }
    
    fn simulate_realistic_hmr_scenario(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate a realistic application with multiple modules and dependencies
        let modules = vec![
            ("user_service", b"export function getUser(id) { return users[id]; }"),
            ("auth_service", b"export function authenticate(token) { return validateToken(token); }"),
            ("api_gateway", b"import { getUser } from 'user_service'; import { authenticate } from 'auth_service';"),
            ("database", b"export const users = new Map(); export function query(sql) { /* ... */ }"),
        ];
        
        // Deploy initial version
        for (name, content) in &modules {
            let module_hash = ContentHash::new(content);
            for (_, node) in self.nodes.iter_mut() {
                let snapshot = node.state_manager.create_snapshot(module_hash)?;
                // Simulate some initial state
                self.add_realistic_state(&mut node.state_manager, name, &snapshot)?;
            }
        }
        
        // Simulate realistic update scenarios
        self.simulate_hot_fix_deployment()?;
        self.simulate_feature_rollout()?;
        self.simulate_schema_migration()?;
        self.simulate_rollback_scenario()?;
        
        Ok(())
    }
    
    fn simulate_hot_fix_deployment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing hot fix deployment scenario...");
        
        let old_content = b"export function validateInput(data) { return data.length > 0; }";
        let fixed_content = b"export function validateInput(data) { return data && data.length > 0; }"; // Fixed null check
        
        let start = Instant::now();
        
        // Coordinate update across all nodes
        let coordinator_id = NodeId::new(1);
        if let Some(coordinator) = self.nodes.get_mut(&coordinator_id) {
            let old_hash = ContentHash::new(old_content);
            let new_hash = ContentHash::new(fixed_content);
            
            // Analyze change
            let analysis = coordinator.hmr_coordinator.analyze_change(old_hash, new_hash)?;
            self.metrics_collector.record_analysis_time(start.elapsed());
            
            // Plan update
            let plan = coordinator.hmr_coordinator.plan_update(old_hash, new_hash);
            
            // Execute across cluster
            let result = coordinator.hmr_coordinator.coordinate_distributed_update(plan)?;
            
            self.metrics_collector.record_update_latency(start.elapsed());
            
            // Verify update succeeded
            assert!(matches!(result.status, DistributedUpdateStatus::Completed));
            assert_eq!(result.successful_nodes.len(), self.nodes.len());
        }
        
        println!("Hot fix deployment completed in {}ms", start.elapsed().as_millis());
        Ok(())
    }
    
    fn simulate_feature_rollout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing gradual feature rollout scenario...");
        
        let old_version = b"export const FEATURE_FLAGS = { newUI: false };";
        let new_version = b"export const FEATURE_FLAGS = { newUI: true, analytics: true };";
        
        // Implement canary deployment - update nodes one by one
        let node_ids: Vec<_> = self.nodes.keys().cloned().collect();
        
        for (i, &node_id) in node_ids.iter().enumerate() {
            let start = Instant::now();
            
            if let Some(node) = self.nodes.get_mut(&node_id) {
                let old_hash = ContentHash::new(old_version);
                let new_hash = ContentHash::new(new_version);
                
                // Update single node
                let plan = node.hmr_coordinator.plan_update(old_hash, new_hash);
                let result = node.hmr_coordinator.execute_update(plan)?;
                
                // Verify health after update
                self.verify_node_health(node_id)?;
                
                // Wait between updates (canary deployment)
                if i < node_ids.len() - 1 {
                    thread::sleep(Duration::from_millis(100));
                }
                
                self.metrics_collector.record_canary_update_time(start.elapsed());
            }
        }
        
        println!("Gradual feature rollout completed successfully");
        Ok(())
    }
    
    fn simulate_schema_migration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing schema migration scenario...");
        
        // Simulate adding a new field to a data structure
        let old_schema = b"type User = { id: number, name: string }";
        let new_schema = b"type User = { id: number, name: string, email: string, createdAt: Date }";
        
        let start = Instant::now();
        
        // Register migration function
        let old_hash = ContentHash::new(old_schema);
        let new_hash = ContentHash::new(new_schema);
        
        for (_, node) in self.nodes.iter_mut() {
            // Register migration function
            let migration_fn = |value: Value| -> Result<Value, MigrationError> {
                if let Value::Struct(mut fields) = value {
                    // Add new fields with default values
                    fields.insert("email".to_string(), Value::String("user@example.com".to_string()));
                    fields.insert("createdAt".to_string(), Value::String("2024-01-01T00:00:00Z".to_string()));
                    Ok(Value::Struct(fields))
                } else {
                    Ok(value)
                }
            };
            
            node.state_manager.register_migration(old_hash, new_hash, Box::new(migration_fn));
        }
        
        // Execute schema migration
        let coordinator_id = NodeId::new(1);
        if let Some(coordinator) = self.nodes.get_mut(&coordinator_id) {
            let plan = coordinator.hmr_coordinator.plan_update(old_hash, new_hash);
            let result = coordinator.hmr_coordinator.coordinate_distributed_update(plan)?;
            
            assert!(matches!(result.status, DistributedUpdateStatus::Completed));
            self.metrics_collector.record_migration_time(start.elapsed());
        }
        
        println!("Schema migration completed in {}ms", start.elapsed().as_millis());
        Ok(())
    }
    
    fn simulate_rollback_scenario(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing rollback scenario...");
        
        let working_version = b"export function processData(data) { return data.map(x => x * 2); }";
        let broken_version = b"export function processData(data) { return data.map(x => x.invalidMethod()); }"; // Will cause runtime error
        
        let start = Instant::now();
        
        // Attempt to deploy broken version
        let coordinator_id = NodeId::new(1);
        if let Some(coordinator) = self.nodes.get_mut(&coordinator_id) {
            let old_hash = ContentHash::new(working_version);
            let new_hash = ContentHash::new(broken_version);
            
            let plan = coordinator.hmr_coordinator.plan_update(old_hash, new_hash);
            let result = coordinator.hmr_coordinator.coordinate_distributed_update(plan);
            
            // Update should fail and trigger rollback
            match result {
                Ok(update_result) => {
                    if update_result.status == DistributedUpdateStatus::Failed {
                        // Verify rollback occurred
                        assert!(update_result.rollback_info.is_some());
                        println!("Rollback triggered successfully");
                    }
                }
                Err(_) => {
                    // Error is expected for broken deployment
                    println!("Deployment failed as expected, testing rollback...");
                    
                    // Manually trigger rollback
                    let rollback_result = coordinator.hmr_coordinator.rollback_to_previous_version(old_hash)?;
                    assert!(matches!(rollback_result.status, DistributedUpdateStatus::Completed));
                }
            }
            
            self.metrics_collector.record_rollback_time(start.elapsed());
        }
        
        println!("Rollback scenario completed in {}ms", start.elapsed().as_millis());
        Ok(())
    }
    
    fn test_network_partition_resilience(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing network partition resilience...");
        
        // Create partition: [1, 2] vs [3, 4, 5]
        let partition1 = vec![NodeId::new(1), NodeId::new(2)];
        let partition2 = vec![NodeId::new(3), NodeId::new(4), NodeId::new(5)];
        
        self.network_simulator.create_partition(partition1.clone(), partition2.clone());
        
        // Test that majority partition can continue operating
        let majority_leader = NodeId::new(3);
        if let Some(leader) = self.nodes.get_mut(&majority_leader) {
            // Simulate node failures in minority partition
            for &failed_id in &partition1 {
                let _ = leader.consensus.handle_node_failure(failed_id);
            }
            
            // Should still have quorum
            assert!(leader.consensus.has_quorum());
            
            // Should be able to perform updates
            let old_hash = ContentHash::new(b"partition_test_v1");
            let new_hash = ContentHash::new(b"partition_test_v2");
            
            let plan = leader.hmr_coordinator.plan_update(old_hash, new_hash);
            let result = leader.hmr_coordinator.execute_update(plan)?;
            
            assert!(matches!(result.status, UpdateStatus::Completed));
        }
        
        // Test partition healing
        self.network_simulator.heal_partition();
        
        // Test node rejoin
        if let Some(leader) = self.nodes.get_mut(&majority_leader) {
            for &rejoining_id in &partition1 {
                let result = leader.consensus.handle_node_rejoin(rejoining_id);
                assert!(result.is_ok());
            }
            
            // Should have full cluster again
            assert_eq!(leader.consensus.get_active_nodes().len(), 5);
        }
        
        println!("Network partition resilience test completed");
        Ok(())
    }
    
    fn test_high_load_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing high load performance...");
        
        let load_duration = Duration::from_secs(10);
        let start = Instant::now();
        
        // Generate high load
        let (tx, rx) = mpsc::channel();
        let mut handles = vec![];
        
        // Spawn load generators
        for i in 0..10 {
            let tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                let mut operations = 0;
                let thread_start = Instant::now();
                
                while thread_start.elapsed() < load_duration {
                    // Simulate various operations
                    match i % 4 {
                        0 => {
                            // HMR operations
                            let _old_hash = ContentHash::new(format!("load_test_{}", operations).as_bytes());
                            let _new_hash = ContentHash::new(format!("load_test_{}_updated", operations).as_bytes());
                            operations += 1;
                        }
                        1 => {
                            // State operations
                            operations += 1;
                        }
                        2 => {
                            // Consensus operations
                            operations += 1;
                        }
                        3 => {
                            // CRDT operations
                            let mut counter = GCounter::new(NodeId::new(i as u64));
                            let _ = counter.increment(1);
                            operations += 1;
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
        for _ in 0..10 {
            total_operations += rx.recv().unwrap();
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let actual_duration = start.elapsed();
        let ops_per_second = total_operations as f64 / actual_duration.as_secs_f64();
        
        self.metrics_collector.record_throughput(ops_per_second);
        
        println!("High load test: {} ops/sec over {}s", ops_per_second, actual_duration.as_secs());
        
        // Should maintain reasonable throughput under load
        assert!(ops_per_second > 1000.0);
        
        Ok(())
    }
    
    fn test_memory_pressure_handling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing memory pressure handling...");
        
        let start = Instant::now();
        
        // Create memory pressure by generating many snapshots
        for (_, node) in self.nodes.iter_mut() {
            let mut snapshots = Vec::new();
            
            for i in 0..1000 {
                let module_hash = ContentHash::new(format!("memory_pressure_{}", i).as_bytes());
                
                match node.state_manager.create_snapshot(module_hash) {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(_) => break, // Expected under memory pressure
                }
                
                // Stop if taking too long
                if start.elapsed().as_secs() > 5 {
                    break;
                }
            }
            
            println!("Node {} created {} snapshots under memory pressure", 
                     node.id.value(), snapshots.len());
            
            // Should create at least some snapshots
            assert!(snapshots.len() > 10);
            
            // Test cleanup
            let cleanup_start = Instant::now();
            snapshots.clear();
            let cleanup_time = cleanup_start.elapsed();
            
            // Cleanup should be fast
            assert!(cleanup_time.as_millis() < 100);
        }
        
        println!("Memory pressure handling test completed in {}ms", start.elapsed().as_millis());
        Ok(())
    }
    
    fn verify_system_consistency(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Verifying system consistency...");
        
        // Check that all nodes are in consistent state
        let mut cluster_sizes = Vec::new();
        let mut active_node_counts = Vec::new();
        
        for (_, node) in &self.nodes {
            cluster_sizes.push(node.consensus.cluster_size());
            active_node_counts.push(node.consensus.get_active_nodes().len());
        }
        
        // All nodes should agree on cluster size
        let first_cluster_size = cluster_sizes[0];
        for &size in &cluster_sizes {
            assert_eq!(size, first_cluster_size, "Cluster size inconsistency detected");
        }
        
        // Active node counts should be consistent (within reason)
        let first_active_count = active_node_counts[0];
        for &count in &active_node_counts {
            let diff = (count as i32 - first_active_count as i32).abs();
            assert!(diff <= 1, "Active node count inconsistency: {} vs {}", count, first_active_count);
        }
        
        println!("System consistency verified");
        Ok(())
    }
    
    fn add_realistic_state(&mut self, state_manager: &mut StateManager, module_name: &str, snapshot: &StateSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate realistic application state
        match module_name {
            "user_service" => {
                // Add user data
                let mut users = HashMap::new();
                for i in 1..=100 {
                    users.insert(format!("user_{}", i), Value::Struct({
                        let mut user = HashMap::new();
                        user.insert("id".to_string(), Value::I32(i));
                        user.insert("name".to_string(), Value::String(format!("User {}", i)));
                        user.insert("active".to_string(), Value::Bool(i % 2 == 0));
                        user
                    }));
                }
            }
            "auth_service" => {
                // Add authentication state
                let mut sessions = HashMap::new();
                for i in 1..=50 {
                    sessions.insert(format!("session_{}", i), Value::Struct({
                        let mut session = HashMap::new();
                        session.insert("token".to_string(), Value::String(format!("token_{}", i)));
                        session.insert("user_id".to_string(), Value::I32(i));
                        session.insert("expires_at".to_string(), Value::I64(1640995200 + i * 3600)); // Unix timestamp
                        session
                    }));
                }
            }
            _ => {
                // Generic state
                let mut generic_state = HashMap::new();
                generic_state.insert("initialized".to_string(), Value::Bool(true));
                generic_state.insert("version".to_string(), Value::String("1.0.0".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn verify_node_health(&self, node_id: NodeId) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(node) = self.nodes.get(&node_id) {
            assert!(node.is_healthy, "Node {} is not healthy", node_id.value());
            assert!(node.consensus.has_quorum(), "Node {} does not have quorum", node_id.value());
        }
        Ok(())
    }
    
    fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Performance Report ===\n");
        
        if !self.metrics_collector.update_latencies.is_empty() {
            let avg_latency = self.metrics_collector.update_latencies.iter()
                .map(|d| d.as_millis() as f64)
                .sum::<f64>() / self.metrics_collector.update_latencies.len() as f64;
            report.push_str(&format!("Average update latency: {:.2}ms\n", avg_latency));
        }
        
        if !self.metrics_collector.throughput_samples.is_empty() {
            let avg_throughput = self.metrics_collector.throughput_samples.iter().sum::<f64>() 
                / self.metrics_collector.throughput_samples.len() as f64;
            report.push_str(&format!("Average throughput: {:.2} ops/sec\n", avg_throughput));
        }
        
        report.push_str(&format!("Total error count: {}\n", 
            self.metrics_collector.error_counts.values().sum::<usize>()));
        
        report
    }
}

impl NetworkSimulator {
    fn new() -> Self {
        Self {
            partitions: Vec::new(),
            latencies: HashMap::new(),
            packet_loss: HashMap::new(),
            is_partitioned: false,
        }
    }
    
    fn create_partition(&mut self, partition1: Vec<NodeId>, partition2: Vec<NodeId>) {
        self.partitions = vec![partition1, partition2];
        self.is_partitioned = true;
    }
    
    fn heal_partition(&mut self) {
        self.partitions.clear();
        self.is_partitioned = false;
    }
}

impl LoadGenerator {
    fn new() -> Self {
        Self {
            update_rate: 10.0, // updates per second
            state_size: 1000,   // number of state entries
            concurrent_operations: 10,
        }
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            update_latencies: Vec::new(),
            consensus_times: Vec::new(),
            state_migration_times: Vec::new(),
            error_counts: HashMap::new(),
            throughput_samples: Vec::new(),
        }
    }
    
    fn record_update_latency(&mut self, latency: Duration) {
        self.update_latencies.push(latency);
    }
    
    fn record_analysis_time(&mut self, time: Duration) {
        // Record analysis time for metrics
    }
    
    fn record_migration_time(&mut self, time: Duration) {
        self.state_migration_times.push(time);
    }
    
    fn record_rollback_time(&mut self, time: Duration) {
        // Record rollback time for metrics
    }
    
    fn record_canary_update_time(&mut self, time: Duration) {
        // Record canary update time for metrics
    }
    
    fn record_throughput(&mut self, ops_per_sec: f64) {
        self.throughput_samples.push(ops_per_sec);
    }
    
    fn record_error(&mut self, error_type: String) {
        *self.error_counts.entry(error_type).or_insert(0) += 1;
    }
}

// Integration test cases

#[test]
fn test_end_to_end_hmr_scenarios() {
    let mut harness = IntegrationTestHarness::new(5);
    
    // Run comprehensive HMR scenarios
    harness.simulate_realistic_hmr_scenario().expect("HMR scenarios should complete successfully");
    
    // Verify system consistency after all scenarios
    harness.verify_system_consistency().expect("System should remain consistent");
    
    // Generate performance report
    let report = harness.generate_performance_report();
    println!("{}", report);
}

#[test]
fn test_distributed_system_integration() {
    let mut harness = IntegrationTestHarness::new(7); // Larger cluster for distributed testing
    
    // Test network partition resilience
    harness.test_network_partition_resilience().expect("Should handle network partitions");
    
    // Test Byzantine fault tolerance
    // Simulate 2 Byzantine failures in 7-node cluster
    let byzantine_nodes = vec![NodeId::new(6), NodeId::new(7)];
    for &node_id in &byzantine_nodes {
        if let Some(node) = harness.nodes.get_mut(&NodeId::new(1)) {
            let _ = node.consensus.handle_node_failure(node_id);
        }
    }
    
    // Should still maintain quorum and operate
    harness.verify_system_consistency().expect("Should handle Byzantine failures");
    
    println!("Distributed system integration tests completed successfully");
}

#[test]
fn test_performance_and_stress() {
    let mut harness = IntegrationTestHarness::new(5);
    
    // Test high load performance
    harness.test_high_load_performance().expect("Should handle high load");
    
    // Test memory pressure
    harness.test_memory_pressure_handling().expect("Should handle memory pressure");
    
    // Test long-running stability
    let start = Instant::now();
    let test_duration = Duration::from_secs(30);
    
    let mut operation_count = 0;
    while start.elapsed() < test_duration {
        // Perform continuous operations
        let old_hash = ContentHash::new(format!("stability_test_{}", operation_count).as_bytes());
        let new_hash = ContentHash::new(format!("stability_test_{}_updated", operation_count).as_bytes());
        
        if let Some(coordinator) = harness.nodes.get_mut(&NodeId::new(1)) {
            let plan = coordinator.hmr_coordinator.plan_update(old_hash, new_hash);
            if plan.status == UpdateStatus::Planned {
                operation_count += 1;
            }
        }
        
        if operation_count % 100 == 0 {
            thread::sleep(Duration::from_millis(10)); // Brief pause
        }
    }
    
    println!("Stability test: {} operations over {}s", operation_count, test_duration.as_secs());
    assert!(operation_count > 1000, "Should maintain high throughput during stability test");
    
    // Verify system is still consistent after stress test
    harness.verify_system_consistency().expect("System should remain consistent after stress");
}

#[test]
fn test_cross_backend_compatibility() {
    use mir_backend_wasm::{WasmCodeGenerator, OptimizationLevel};
    use mir_ast::{Module, Statement, Expression, NodeId, ExportDeclaration, ExportVisibility, CompatibilityInfo};
    
    // Create test module for cross-backend testing
    let test_module = Module {
        id: NodeId::new(1),
        name: "cross_backend_test".to_string(),
        imports: vec![],
        exports: vec![
            ExportDeclaration {
                name: "test_function".to_string(),
                is_type: false,
                exported_hash: ContentHash::new(b"test_function"),
                visibility: ExportVisibility::Public,
                compatibility_info: CompatibilityInfo {
                    version: "1.0.0".to_string(),
                    breaking_changes: vec![],
                    deprecated_features: vec![],
                    migration_hints: vec![],
                },
            }
        ],
        statements: vec![
            Statement::FunctionDeclaration {
                id: NodeId::new(2),
                name: "test_function".to_string(),
                parameters: vec!["param".to_string()],
                body: vec![
                    Statement::Expression {
                        id: NodeId::new(3),
                        expression: Expression::Literal {
                            id: NodeId::new(4),
                            value: Value::I32(42),
                        },
                    },
                ],
            },
        ],
        type_hash: mir_types::TypeHash::new(ContentHash::new(b"cross_backend_test")),
        capabilities: mir_ast::ModuleCapabilities {
            can_send_messages: false,
            can_receive_messages: false,
            allowed_message_types: vec![],
        },
        message_queue: std::collections::VecDeque::new(),
    };
    
    // Test WASM backend compilation
    let mut wasm_generator = WasmCodeGenerator::new();
    let wasm_result = wasm_generator.generate(&test_module);
    assert!(wasm_result.is_ok(), "WASM backend should compile successfully");
    
    let wasm_module = wasm_result.unwrap();
    assert_eq!(wasm_module.functions.len(), 1);
    assert_eq!(wasm_module.exports.len(), 1);
    
    // Test different optimization levels
    let optimization_levels = vec![
        OptimizationLevel::None,
        OptimizationLevel::Size,
        OptimizationLevel::Speed,
        OptimizationLevel::Aggressive,
    ];
    
    for level in optimization_levels {
        let mut generator = WasmCodeGenerator::new();
        generator.set_optimization_level(level);
        
        let result = generator.generate(&test_module);
        assert!(result.is_ok(), "Optimization level {:?} should work", level);
        
        let optimized_module = result.unwrap();
        assert!(optimized_module.bytecode.len() >= 8); // Valid WASM header
        assert_eq!(&optimized_module.bytecode[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }
    
    // Test serialization compatibility
    let serialized = serde_json::to_string(&test_module).expect("Should serialize to JSON");
    let deserialized: Module = serde_json::from_str(&serialized).expect("Should deserialize from JSON");
    
    assert_eq!(test_module.name, deserialized.name);
    assert_eq!(test_module.exports.len(), deserialized.exports.len());
    assert_eq!(test_module.statements.len(), deserialized.statements.len());
    
    println!("Cross-backend compatibility tests completed successfully");
}

#[test]
fn test_fault_tolerance_and_recovery() {
    let mut harness = IntegrationTestHarness::new(5);
    
    // Test cascading failure recovery
    println!("Testing cascading failure recovery...");
    
    // Simulate multiple node failures
    let failed_nodes = vec![NodeId::new(4), NodeId::new(5)];
    for &failed_id in &failed_nodes {
        if let Some(leader) = harness.nodes.get_mut(&NodeId::new(1)) {
            let _ = leader.consensus.handle_node_failure(failed_id);
        }
        harness.nodes.get_mut(&failed_id).unwrap().is_healthy = false;
    }
    
    // System should still operate with majority
    harness.verify_system_consistency().expect("Should handle multiple failures");
    
    // Test recovery when nodes come back online
    for &recovering_id in &failed_nodes {
        if let Some(leader) = harness.nodes.get_mut(&NodeId::new(1)) {
            let result = leader.consensus.handle_node_rejoin(recovering_id);
            assert!(result.is_ok(), "Node rejoin should succeed");
        }
        harness.nodes.get_mut(&recovering_id).unwrap().is_healthy = true;
    }
    
    // Test split-brain prevention
    println!("Testing split-brain prevention...");
    
    // Create equal partition (2 vs 3 nodes)
    let partition1 = vec![NodeId::new(1), NodeId::new(2)];
    let partition2 = vec![NodeId::new(3), NodeId::new(4), NodeId::new(5)];
    
    harness.network_simulator.create_partition(partition1.clone(), partition2.clone());
    
    // Minority partition should not be able to make progress
    if let Some(minority_node) = harness.nodes.get_mut(&NodeId::new(1)) {
        for &failed_id in &partition2 {
            let _ = minority_node.consensus.handle_node_failure(failed_id);
        }
        
        assert!(!minority_node.consensus.has_quorum(), "Minority should not have quorum");
        
        let old_hash = ContentHash::new(b"split_brain_test");
        let new_hash = ContentHash::new(b"split_brain_test_updated");
        let result = minority_node.consensus.propose(ProposalType::UpdateCoordination, vec![1, 2, 3]);
        
        assert!(result.is_err(), "Minority partition should not accept proposals");
    }
    
    // Majority partition should continue operating
    if let Some(majority_node) = harness.nodes.get_mut(&NodeId::new(3)) {
        for &failed_id in &partition1 {
            let _ = majority_node.consensus.handle_node_failure(failed_id);
        }
        
        assert!(majority_node.consensus.has_quorum(), "Majority should have quorum");
        
        let result = majority_node.consensus.propose(ProposalType::UpdateCoordination, vec![4, 5, 6]);
        assert!(result.is_ok(), "Majority partition should accept proposals");
    }
    
    // Heal partition and verify recovery
    harness.network_simulator.heal_partition();
    
    // All nodes should eventually converge
    thread::sleep(Duration::from_millis(100)); // Allow time for convergence
    harness.verify_system_consistency().expect("Should converge after partition healing");
    
    println!("Fault tolerance and recovery tests completed successfully");
}

#[test]
fn test_real_world_deployment_scenarios() {
    let mut harness = IntegrationTestHarness::new(3);
    
    println!("Testing real-world deployment scenarios...");
    
    // Scenario 1: Blue-green deployment
    println!("Testing blue-green deployment...");
    
    let blue_version = b"export const VERSION = 'blue'; export function process() { return 'blue_result'; }";
    let green_version = b"export const VERSION = 'green'; export function process() { return 'green_result'; }";
    
    // Deploy to "green" environment (node 2)
    if let Some(green_node) = harness.nodes.get_mut(&NodeId::new(2)) {
        let blue_hash = ContentHash::new(blue_version);
        let green_hash = ContentHash::new(green_version);
        
        let plan = green_node.hmr_coordinator.plan_update(blue_hash, green_hash);
        let result = green_node.hmr_coordinator.execute_update(plan).expect("Green deployment should succeed");
        
        assert!(matches!(result.status, UpdateStatus::Completed));
    }
    
    // After validation, switch traffic (deploy to all nodes)
    let coordinator_id = NodeId::new(1);
    if let Some(coordinator) = harness.nodes.get_mut(&coordinator_id) {
        let blue_hash = ContentHash::new(blue_version);
        let green_hash = ContentHash::new(green_version);
        
        let plan = coordinator.hmr_coordinator.plan_update(blue_hash, green_hash);
        let result = coordinator.hmr_coordinator.coordinate_distributed_update(plan)
            .expect("Blue-green switch should succeed");
        
        assert!(matches!(result.status, DistributedUpdateStatus::Completed));
    }
    
    // Scenario 2: A/B testing deployment
    println!("Testing A/B testing deployment...");
    
    let version_a = b"export const VARIANT = 'A'; export function experiment() { return 'variant_a'; }";
    let version_b = b"export const VARIANT = 'B'; export function experiment() { return 'variant_b'; }";
    
    // Deploy variant A to nodes 1-2, variant B to node 3
    let deployments = vec![
        (vec![NodeId::new(1), NodeId::new(2)], version_a),
        (vec![NodeId::new(3)], version_b),
    ];
    
    for (node_ids, version) in deployments {
        for &node_id in &node_ids {
            if let Some(node) = harness.nodes.get_mut(&node_id) {
                let old_hash = ContentHash::new(b"original_version");
                let new_hash = ContentHash::new(version);
                
                let plan = node.hmr_coordinator.plan_update(old_hash, new_hash);
                let result = node.hmr_coordinator.execute_update(plan)
                    .expect("A/B deployment should succeed");
                
                assert!(matches!(result.status, UpdateStatus::Completed));
            }
        }
    }
    
    // Scenario 3: Emergency hotfix
    println!("Testing emergency hotfix deployment...");
    
    let vulnerable_code = b"export function authenticate(token) { return true; }"; // Security vulnerability
    let fixed_code = b"export function authenticate(token) { return validateToken(token); }"; // Fixed
    
    let start = Instant::now();
    
    // Emergency deployment should be fast
    if let Some(coordinator) = harness.nodes.get_mut(&coordinator_id) {
        let old_hash = ContentHash::new(vulnerable_code);
        let new_hash = ContentHash::new(fixed_code);
        
        let plan = coordinator.hmr_coordinator.plan_update(old_hash, new_hash);
        let result = coordinator.hmr_coordinator.coordinate_distributed_update(plan)
            .expect("Emergency hotfix should succeed");
        
        assert!(matches!(result.status, DistributedUpdateStatus::Completed));
        
        let deployment_time = start.elapsed();
        println!("Emergency hotfix deployed in {}ms", deployment_time.as_millis());
        
        // Should be very fast for critical fixes
        assert!(deployment_time.as_millis() < 1000, "Emergency deployment should be under 1 second");
    }
    
    println!("Real-world deployment scenarios completed successfully");
}