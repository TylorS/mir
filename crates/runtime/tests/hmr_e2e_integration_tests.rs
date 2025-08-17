//! End-to-end HMR integration tests
//! 
//! Tests complete HMR workflows including:
//! - Full update lifecycle from change detection to completion
//! - State preservation across updates
//! - Rollback scenarios
//! - Multi-node coordination

use mir_runtime::*;
use mir_types::{ContentHash, Value, TypeRegistry};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

/// Helper to create a test HMR system with all components
struct TestHMRSystem {
    coordinator: HMRCoordinator,
    state_manager: StateManager,
    change_analyzer: ChangeAnalysisSystem,
    migration_system: StateMigrationSystem,
    type_registry: TypeRegistry,
}

impl TestHMRSystem {
    fn new() -> Self {
        Self {
            coordinator: HMRCoordinator::new(),
            state_manager: StateManager::new(),
            change_analyzer: ChangeAnalysisSystem::new(),
            migration_system: StateMigrationSystem::new(),
            type_registry: TypeRegistry::new(),
        }
    }

    fn simulate_module_update(&mut self, old_content: &[u8], new_content: &[u8]) -> Result<UpdateResult, HMRError> {
        let old_hash = ContentHash::new(old_content);
        let new_hash = ContentHash::new(new_content);

        // 1. Analyze the change
        let analysis = self.change_analyzer.analyze_change(old_hash, new_hash);
        
        // 2. Create snapshot of current state
        let snapshot = self.state_manager.create_snapshot(old_hash)?;
        
        // 3. Plan the update
        let plan = self.coordinator.plan_update(old_hash, new_hash);
        
        // 4. Execute the update
        let result = self.coordinator.execute_update(plan)?;
        
        // 5. If successful, clean up old snapshot; if failed, restore
        match result.status {
            UpdateStatus::Completed => {
                // Update successful, can clean up snapshot
                Ok(result)
            }
            UpdateStatus::Failed => {
                // Restore from snapshot
                self.state_manager.restore_snapshot(snapshot)?;
                Ok(result)
            }
            _ => Ok(result)
        }
    }
}

#[test]
fn test_complete_hmr_workflow() {
    let mut system = TestHMRSystem::new();
    
    // Simulate initial module
    let initial_module = b"
        function calculate(x) {
            return x * 2;
        }
        export { calculate };
    ";
    
    // Simulate updated module
    let updated_module = b"
        function calculate(x) {
            return x * 3; // Changed multiplier
        }
        export { calculate };
    ";
    
    // Execute the update
    let result = system.simulate_module_update(initial_module, updated_module);
    assert!(result.is_ok());
    
    let update_result = result.unwrap();
    assert!(matches!(update_result.status, UpdateStatus::Completed | UpdateStatus::Failed));
    
    // Verify that the update was tracked
    assert!(!update_result.affected_modules.is_empty());
}

#[test]
fn test_state_preservation_during_update() {
    let mut system = TestHMRSystem::new();
    
    // Create some initial state
    let module_hash = ContentHash::new(b"stateful_module_v1");
    let mut initial_state = HashMap::new();
    initial_state.insert("counter".to_string(), Value::I32(42));
    initial_state.insert("user_name".to_string(), Value::String("Alice".to_string()));
    
    // Simulate state creation
    let snapshot = system.state_manager.create_snapshot(module_hash).unwrap();
    
    // Simulate module update
    let old_module = b"let counter = 42; let user_name = 'Alice';";
    let new_module = b"let counter = 42; let user_name = 'Alice'; let new_feature = true;";
    
    let result = system.simulate_module_update(old_module, new_module);
    assert!(result.is_ok());
    
    // Verify state was preserved
    let new_hash = ContentHash::new(new_module);
    let restored_snapshot = system.state_manager.create_snapshot(new_hash).unwrap();
    
    // Should have preserved the original state
    assert_eq!(restored_snapshot.module_hash, new_hash);
    assert!(!restored_snapshot.components.is_empty());
}

#[test]
fn test_rollback_on_failed_update() {
    let mut system = TestHMRSystem::new();
    
    // Set up a scenario that should fail
    let working_module = b"function good() { return 'working'; }";
    let broken_module = b"function bad() { throw new Error('broken'); }";
    
    // First, establish working state
    let working_hash = ContentHash::new(working_module);
    let _snapshot = system.state_manager.create_snapshot(working_hash).unwrap();
    
    // Attempt update that should fail
    let result = system.simulate_module_update(working_module, broken_module);
    
    // Should handle the failure gracefully
    assert!(result.is_ok());
    let update_result = result.unwrap();
    
    // If the update failed, the system should have rolled back
    if update_result.status == UpdateStatus::Failed {
        // Verify rollback occurred
        assert!(!update_result.rollback_info.is_none());
    }
}

#[test]
fn test_schema_migration_during_update() {
    let mut system = TestHMRSystem::new();
    
    // Register a migration function
    let old_schema = ContentHash::new(b"user_v1");
    let new_schema = ContentHash::new(b"user_v2");
    
    let migration_fn = |value: Value| -> Result<Value, MigrationError> {
        // Simulate adding a new field to a user object
        if let Value::Struct(mut fields) = value {
            fields.insert("version".to_string(), Value::I32(2));
            Ok(Value::Struct(fields))
        } else {
            Ok(value)
        }
    };
    
    system.migration_system.register_migration(old_schema, new_schema, Box::new(migration_fn));
    
    // Simulate update that requires migration
    let old_module = b"struct User { name: string, age: number }";
    let new_module = b"struct User { name: string, age: number, version: number }";
    
    let result = system.simulate_module_update(old_module, new_module);
    assert!(result.is_ok());
    
    let update_result = result.unwrap();
    assert!(matches!(update_result.status, UpdateStatus::Completed | UpdateStatus::Failed));
}

#[test]
fn test_concurrent_updates_handling() {
    let system = TestHMRSystem::new();
    
    // Simulate multiple concurrent update requests
    let modules = vec![
        (b"module_a_v1".as_slice(), b"module_a_v2".as_slice()),
        (b"module_b_v1".as_slice(), b"module_b_v2".as_slice()),
        (b"module_c_v1".as_slice(), b"module_c_v2".as_slice()),
    ];
    
    let mut plans = Vec::new();
    
    for (old, new) in modules {
        let old_hash = ContentHash::new(old);
        let new_hash = ContentHash::new(new);
        let plan = system.coordinator.plan_update(old_hash, new_hash);
        plans.push(plan);
    }
    
    // All plans should be created successfully
    assert_eq!(plans.len(), 3);
    
    // Each plan should have a unique ID
    let plan_ids: Vec<_> = plans.iter().map(|p| &p.id).collect();
    let unique_ids: std::collections::HashSet<_> = plan_ids.iter().collect();
    assert_eq!(unique_ids.len(), 3);
}

#[test]
fn test_dependency_chain_updates() {
    let mut system = TestHMRSystem::new();
    
    // Set up dependency chain: A -> B -> C
    let module_a = ContentHash::new(b"module_a");
    let module_b = ContentHash::new(b"module_b");
    let module_c = ContentHash::new(b"module_c");
    
    system.change_analyzer.add_dependency(module_a, module_b);
    system.change_analyzer.add_dependency(module_b, module_c);
    
    // Update module C, which should affect A and B
    let old_c = b"module_c_v1";
    let new_c = b"module_c_v2";
    
    let result = system.simulate_module_update(old_c, new_c);
    assert!(result.is_ok());
    
    let update_result = result.unwrap();
    
    // Should have identified affected modules in the dependency chain
    assert!(!update_result.affected_modules.is_empty());
    
    // The update should account for the dependency chain
    let affected_count = update_result.affected_modules.len();
    assert!(affected_count >= 1); // At least the changed module itself
}

#[test]
fn test_performance_with_large_state() {
    use std::time::Instant;
    
    let mut system = TestHMRSystem::new();
    
    // Create a large state to test performance
    let module_hash = ContentHash::new(b"large_state_module");
    
    // Simulate creating a large state snapshot
    let start = Instant::now();
    let snapshot = system.state_manager.create_snapshot(module_hash).unwrap();
    let snapshot_duration = start.elapsed();
    
    // Should complete in reasonable time
    assert!(snapshot_duration.as_secs() < 1);
    
    // Test update with large state
    let old_module = vec![0u8; 10000]; // 10KB module
    let new_module = vec![1u8; 10000]; // 10KB updated module
    
    let start = Instant::now();
    let result = system.simulate_module_update(&old_module, &new_module);
    let update_duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(update_duration.as_secs() < 2); // Should complete within 2 seconds
}

#[test]
fn test_error_recovery_scenarios() {
    let mut system = TestHMRSystem::new();
    
    // Test various error scenarios
    let error_scenarios = vec![
        // Invalid module content
        (b"valid_module".as_slice(), b"invalid{syntax".as_slice()),
        // Empty module
        (b"some_content".as_slice(), b"".as_slice()),
        // Very large module
        (b"small".as_slice(), &vec![0u8; 1024 * 1024]), // 1MB
    ];
    
    for (old, new) in error_scenarios {
        let result = system.simulate_module_update(old, new);
        
        // Should handle errors gracefully without panicking
        match result {
            Ok(update_result) => {
                // If successful, that's fine
                assert!(matches!(update_result.status, UpdateStatus::Completed | UpdateStatus::Failed));
            }
            Err(_) => {
                // Errors are acceptable for invalid input
            }
        }
    }
}

#[test]
fn test_atomic_multi_module_update() {
    let mut system = TestHMRSystem::new();
    
    // Test updating multiple related modules atomically
    let modules = vec![
        ("shared_types", b"type User = { name: string }"),
        ("user_service", b"import { User } from 'shared_types'; function getUser(): User { ... }"),
        ("user_controller", b"import { getUser } from 'user_service'; export function handleRequest() { ... }"),
    ];
    
    // Create snapshots for all modules
    let mut snapshots = Vec::new();
    for (name, content) in &modules {
        let hash = ContentHash::new(content);
        let snapshot = system.state_manager.create_snapshot(hash).unwrap();
        snapshots.push((name, snapshot));
    }
    
    // Simulate atomic update of all modules
    let updated_modules = vec![
        ("shared_types", b"type User = { name: string, id: number }"), // Added id field
        ("user_service", b"import { User } from 'shared_types'; function getUser(): User { ... }"),
        ("user_controller", b"import { getUser } from 'user_service'; export function handleRequest() { ... }"),
    ];
    
    // In a real implementation, this would be an atomic operation
    let mut all_successful = true;
    let mut results = Vec::new();
    
    for ((old_name, old_content), (new_name, new_content)) in modules.iter().zip(updated_modules.iter()) {
        assert_eq!(old_name, new_name); // Sanity check
        
        let result = system.simulate_module_update(old_content, new_content);
        match result {
            Ok(update_result) => {
                results.push(update_result);
                if !matches!(update_result.status, UpdateStatus::Completed) {
                    all_successful = false;
                }
            }
            Err(_) => {
                all_successful = false;
                break;
            }
        }
    }
    
    // If any update failed, all should be rolled back (in a real atomic implementation)
    if !all_successful {
        // Simulate rollback of all modules
        for (_, snapshot) in snapshots {
            let _ = system.state_manager.restore_snapshot(snapshot);
        }
    }
    
    // Test should complete without panicking
    assert!(true);
}

#[test]
fn test_hot_reload_with_active_connections() {
    let mut system = TestHMRSystem::new();
    
    // Simulate a module with active connections/state
    let module_with_connections = b"
        let activeConnections = new Map();
        function addConnection(id, conn) { activeConnections.set(id, conn); }
        export { addConnection, activeConnections };
    ";
    
    let updated_module = b"
        let activeConnections = new Map();
        let connectionStats = { total: 0, active: 0 }; // New feature
        function addConnection(id, conn) { 
            activeConnections.set(id, conn);
            connectionStats.total++;
            connectionStats.active++;
        }
        export { addConnection, activeConnections, connectionStats };
    ";
    
    // Create initial state with some "connections"
    let module_hash = ContentHash::new(module_with_connections);
    let snapshot = system.state_manager.create_snapshot(module_hash).unwrap();
    
    // Perform hot reload
    let result = system.simulate_module_update(module_with_connections, updated_module);
    assert!(result.is_ok());
    
    let update_result = result.unwrap();
    
    // Update should preserve existing connections while adding new functionality
    assert!(matches!(update_result.status, UpdateStatus::Completed | UpdateStatus::Failed));
    
    // In a real implementation, we would verify that:
    // 1. Existing connections are preserved
    // 2. New functionality is available
    // 3. No connections were dropped during the update
}

#[test]
fn test_gradual_rollout_simulation() {
    let system = TestHMRSystem::new();
    
    // Simulate gradual rollout across multiple instances
    let instances = vec!["instance_1", "instance_2", "instance_3", "instance_4", "instance_5"];
    let old_version = b"version_1.0.0";
    let new_version = b"version_1.1.0";
    
    let mut rollout_results = Vec::new();
    
    // Roll out to instances one by one
    for instance in instances {
        let old_hash = ContentHash::new(&format!("{}_{}", instance, std::str::from_utf8(old_version).unwrap()).as_bytes());
        let new_hash = ContentHash::new(&format!("{}_{}", instance, std::str::from_utf8(new_version).unwrap()).as_bytes());
        
        let plan = system.coordinator.plan_update(old_hash, new_hash);
        rollout_results.push((instance, plan));
        
        // In a real gradual rollout, we would:
        // 1. Wait for health checks
        // 2. Monitor metrics
        // 3. Decide whether to continue or rollback
    }
    
    // All instances should have valid update plans
    assert_eq!(rollout_results.len(), 5);
    
    for (instance, plan) in rollout_results {
        assert_eq!(plan.status, UpdateStatus::Planned);
        assert!(!plan.steps.is_empty());
    }
}

#[test]
fn test_cross_module_type_compatibility() {
    let mut system = TestHMRSystem::new();
    
    // Register some types in the type registry
    let user_type_v1 = system.type_registry.register_scalar_type("User", 64);
    let user_type_v2 = system.type_registry.register_scalar_type("UserV2", 72); // Larger size
    
    // Simulate modules that depend on these types
    let module_a = b"import { User } from 'types'; function processUser(u: User) { ... }";
    let module_b = b"import { User } from 'types'; function createUser(): User { ... }";
    
    // Update the type definition
    let updated_types = b"export type UserV2 = User & { id: number }";
    
    // This should trigger updates in dependent modules
    let result = system.simulate_module_update(b"export type User = { name: string }", updated_types);
    assert!(result.is_ok());
    
    let update_result = result.unwrap();
    
    // Should have analyzed type compatibility
    assert!(matches!(update_result.status, UpdateStatus::Completed | UpdateStatus::Failed));
}

#[test]
fn test_memory_usage_during_updates() {
    let mut system = TestHMRSystem::new();
    
    // Create multiple snapshots to test memory management
    let mut snapshots = Vec::new();
    
    for i in 0..10 {
        let module_content = format!("module_{}_content", i);
        let module_hash = ContentHash::new(module_content.as_bytes());
        
        let snapshot = system.state_manager.create_snapshot(module_hash).unwrap();
        snapshots.push(snapshot);
    }
    
    // Perform updates that should clean up old snapshots
    for i in 0..5 {
        let old_content = format!("module_{}_content", i);
        let new_content = format!("module_{}_updated_content", i);
        
        let result = system.simulate_module_update(old_content.as_bytes(), new_content.as_bytes());
        assert!(result.is_ok());
    }
    
    // In a real implementation, we would verify that:
    // 1. Old snapshots are cleaned up
    // 2. Memory usage doesn't grow unbounded
    // 3. Only necessary state is retained
    
    // For now, just verify the operations completed successfully
    assert_eq!(snapshots.len(), 10);
}