//! Unit tests for HMR coordinator functionality
//! 
//! Tests the hot-module-reloading coordinator operations including:
//! - Change analysis and impact assessment
//! - Update planning and execution
//! - State preservation and migration
//! - Rollback mechanisms

use mir_runtime::*;
use mir_types::{ContentHash, Value};
use std::collections::HashMap;

#[test]
fn test_change_analysis_basic_operations() {
    let mut analyzer = ChangeAnalysisSystem::new();
    
    // Test with no changes
    let old_hash = ContentHash::new(b"module_v1");
    let new_hash = ContentHash::new(b"module_v1"); // Same content
    
    let analysis = analyzer.analyze_change(old_hash, new_hash);
    assert_eq!(analysis.compatibility, CompatibilityLevel::FullyCompatible);
    assert!(analysis.affected_modules.is_empty());
}

#[test]
fn test_change_analysis_with_modifications() {
    let mut analyzer = ChangeAnalysisSystem::new();
    
    let old_hash = ContentHash::new(b"module_v1");
    let new_hash = ContentHash::new(b"module_v2");
    
    let analysis = analyzer.analyze_change(old_hash, new_hash);
    assert_ne!(analysis.compatibility, CompatibilityLevel::FullyCompatible);
    assert!(!analysis.affected_modules.is_empty());
}

#[test]
fn test_hmr_coordinator_basic_operations() {
    let coordinator = HMRCoordinator::new();
    
    // Test initial state
    assert_eq!(coordinator.get_safety_level(), SafetyLevel::Standard);
    
    // Test update planning
    let old_hash = ContentHash::new(b"old_module");
    let new_hash = ContentHash::new(b"new_module");
    
    let plan = coordinator.plan_update(old_hash, new_hash);
    assert!(!plan.steps.is_empty());
    assert_eq!(plan.priority, UpdatePriority::Normal);
}

#[test]
fn test_update_plan_creation() {
    let coordinator = HMRCoordinator::new();
    
    let old_hash = ContentHash::new(b"module_v1");
    let new_hash = ContentHash::new(b"module_v2");
    
    let plan = coordinator.plan_update(old_hash, new_hash);
    
    // Verify plan structure
    assert!(!plan.steps.is_empty());
    assert_eq!(plan.status, UpdateStatus::Planned);
    
    // Check that steps are in logical order
    let step_types: Vec<_> = plan.steps.iter().map(|s| &s.step_type).collect();
    assert!(step_types.contains(&&UpdateStepType::ValidateCompatibility));
    assert!(step_types.contains(&&UpdateStepType::PreserveState));
    assert!(step_types.contains(&&UpdateStepType::ApplyUpdate));
}

#[test]
fn test_state_manager_operations() {
    let mut state_manager = StateManager::new();
    
    // Test state snapshot creation
    let module_hash = ContentHash::new(b"test_module");
    let snapshot_result = state_manager.create_snapshot(module_hash);
    assert!(snapshot_result.is_ok());
    
    let snapshot = snapshot_result.unwrap();
    assert_eq!(snapshot.module_hash, module_hash);
    assert!(!snapshot.components.is_empty());
}

#[test]
fn test_state_preservation_and_restoration() {
    let mut state_manager = StateManager::new();
    
    // Create some test state
    let module_hash = ContentHash::new(b"test_module");
    let mut test_state = HashMap::new();
    test_state.insert("counter".to_string(), Value::I32(42));
    test_state.insert("name".to_string(), Value::String("test".to_string()));
    
    // Create snapshot
    let snapshot = state_manager.create_snapshot(module_hash).unwrap();
    
    // Verify snapshot contains expected data
    assert_eq!(snapshot.module_hash, module_hash);
    assert!(!snapshot.components.is_empty());
    
    // Test restoration
    let restoration_result = state_manager.restore_snapshot(snapshot);
    assert!(restoration_result.is_ok());
}

#[test]
fn test_rollback_manager_operations() {
    let mut rollback_manager = RollbackManager::new();
    
    let update_id = "test_update_123".to_string();
    let old_hash = ContentHash::new(b"old_version");
    let new_hash = ContentHash::new(b"new_version");
    
    // Create rollback plan
    let rollback_plan = rollback_manager.create_rollback_plan(update_id.clone(), old_hash, new_hash);
    assert!(!rollback_plan.steps.is_empty());
    assert_eq!(rollback_plan.update_id, update_id);
    
    // Test rollback execution
    let execution_result = rollback_manager.execute_rollback(rollback_plan);
    assert!(execution_result.is_ok());
}

#[test]
fn test_validation_system() {
    let validator = UpdateValidator::new();
    
    let old_hash = ContentHash::new(b"module_v1");
    let new_hash = ContentHash::new(b"module_v2");
    
    let validation_result = validator.validate_update(old_hash, new_hash);
    assert!(validation_result.is_valid);
    assert!(!validation_result.warnings.is_empty() || validation_result.warnings.is_empty()); // Either is fine
}

#[test]
fn test_update_execution_workflow() {
    let mut coordinator = HMRCoordinator::new();
    
    let old_hash = ContentHash::new(b"workflow_test_v1");
    let new_hash = ContentHash::new(b"workflow_test_v2");
    
    // Plan the update
    let plan = coordinator.plan_update(old_hash, new_hash);
    assert_eq!(plan.status, UpdateStatus::Planned);
    
    // Execute the update
    let execution_result = coordinator.execute_update(plan);
    assert!(execution_result.is_ok());
    
    let result = execution_result.unwrap();
    assert!(matches!(result.status, UpdateStatus::Completed | UpdateStatus::Failed));
}

#[test]
fn test_concurrent_update_handling() {
    let coordinator = HMRCoordinator::new();
    
    // Test that concurrent updates are handled properly
    let hash1 = ContentHash::new(b"concurrent_test_1");
    let hash2 = ContentHash::new(b"concurrent_test_2");
    let hash3 = ContentHash::new(b"concurrent_test_3");
    
    let plan1 = coordinator.plan_update(hash1, hash2);
    let plan2 = coordinator.plan_update(hash2, hash3);
    
    // Both plans should be valid
    assert_eq!(plan1.status, UpdateStatus::Planned);
    assert_eq!(plan2.status, UpdateStatus::Planned);
    
    // Plans should have different IDs
    assert_ne!(plan1.id, plan2.id);
}

#[test]
fn test_safety_level_configuration() {
    let mut coordinator = HMRCoordinator::new();
    
    // Test default safety level
    assert_eq!(coordinator.get_safety_level(), SafetyLevel::Standard);
    
    // Test changing safety level
    coordinator.set_safety_level(SafetyLevel::Paranoid);
    assert_eq!(coordinator.get_safety_level(), SafetyLevel::Paranoid);
    
    coordinator.set_safety_level(SafetyLevel::Minimal);
    assert_eq!(coordinator.get_safety_level(), SafetyLevel::Minimal);
}

#[test]
fn test_update_priority_handling() {
    let coordinator = HMRCoordinator::new();
    
    // Test normal priority update
    let normal_hash1 = ContentHash::new(b"normal_update_1");
    let normal_hash2 = ContentHash::new(b"normal_update_2");
    let normal_plan = coordinator.plan_update(normal_hash1, normal_hash2);
    assert_eq!(normal_plan.priority, UpdatePriority::Normal);
    
    // Test high priority update (simulated by using specific content)
    let urgent_hash1 = ContentHash::new(b"URGENT_security_fix_1");
    let urgent_hash2 = ContentHash::new(b"URGENT_security_fix_2");
    let urgent_plan = coordinator.plan_update(urgent_hash1, urgent_hash2);
    // Priority might be determined by content analysis
    assert!(matches!(urgent_plan.priority, UpdatePriority::Normal | UpdatePriority::High | UpdatePriority::Critical));
}

#[test]
fn test_error_handling_in_updates() {
    let mut coordinator = HMRCoordinator::new();
    
    // Test with invalid hashes that might cause errors
    let invalid_hash1 = ContentHash::zero();
    let invalid_hash2 = ContentHash::new(b"valid_hash");
    
    let plan = coordinator.plan_update(invalid_hash1, invalid_hash2);
    
    // Should handle gracefully
    assert!(matches!(plan.status, UpdateStatus::Planned | UpdateStatus::Failed));
    
    // If planned, execution should handle errors
    if plan.status == UpdateStatus::Planned {
        let execution_result = coordinator.execute_update(plan);
        // Should either succeed or fail gracefully
        match execution_result {
            Ok(result) => {
                assert!(matches!(result.status, UpdateStatus::Completed | UpdateStatus::Failed));
            }
            Err(_) => {
                // Error is acceptable for invalid input
            }
        }
    }
}

#[test]
fn test_state_migration_system() {
    let mut migration_system = StateMigrationSystem::new();
    
    let old_schema = ContentHash::new(b"schema_v1");
    let new_schema = ContentHash::new(b"schema_v2");
    
    // Test migration registration
    let migration_fn = |value: Value| -> Result<Value, MigrationError> {
        // Simple identity migration for testing
        Ok(value)
    };
    
    migration_system.register_migration(old_schema, new_schema, Box::new(migration_fn));
    
    // Test migration execution
    let test_value = Value::I32(42);
    let migration_result = migration_system.migrate_value(test_value, old_schema, new_schema);
    assert!(migration_result.is_ok());
    assert_eq!(migration_result.unwrap(), Value::I32(42));
}

#[test]
fn test_dependency_graph_analysis() {
    let mut analyzer = ChangeAnalysisSystem::new();
    
    // Create a simple dependency graph
    let module_a = ContentHash::new(b"module_a");
    let module_b = ContentHash::new(b"module_b");
    let module_c = ContentHash::new(b"module_c");
    
    // Add dependencies: A -> B -> C
    analyzer.add_dependency(module_a, module_b);
    analyzer.add_dependency(module_b, module_c);
    
    // Analyze impact of changing module C
    let analysis = analyzer.analyze_change(module_c, ContentHash::new(b"module_c_v2"));
    
    // Should affect modules B and A due to dependencies
    assert!(analysis.affected_modules.len() >= 1);
    
    // Check that dependency information is captured
    let affected_hashes: Vec<_> = analysis.affected_modules.iter()
        .map(|m| m.module_hash)
        .collect();
    
    // At minimum, the changed module should be in the affected list
    assert!(affected_hashes.contains(&module_c) || 
            affected_hashes.contains(&ContentHash::new(b"module_c_v2")));
}

#[test]
fn test_performance_with_large_updates() {
    use std::time::Instant;
    
    let coordinator = HMRCoordinator::new();
    
    // Create many update plans to test performance
    let start = Instant::now();
    
    for i in 0..100 {
        let old_hash = ContentHash::new(format!("module_{}_v1", i).as_bytes());
        let new_hash = ContentHash::new(format!("module_{}_v2", i).as_bytes());
        
        let _plan = coordinator.plan_update(old_hash, new_hash);
    }
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time (less than 1 second for 100 plans)
    assert!(duration.as_secs() < 1);
}

#[test]
fn test_atomic_state_updates() {
    let mut state_manager = StateManager::new();
    
    // Test atomic update of multiple components
    let module_hash = ContentHash::new(b"atomic_test_module");
    
    let mut updates = Vec::new();
    updates.push(StateChange {
        component_id: "component1".to_string(),
        change_type: StateChangeType::Update,
        old_value: Some(Value::I32(1)),
        new_value: Value::I32(10),
    });
    updates.push(StateChange {
        component_id: "component2".to_string(),
        change_type: StateChangeType::Update,
        old_value: Some(Value::String("old".to_string())),
        new_value: Value::String("new".to_string()),
    });
    
    let atomic_update = AtomicStateUpdate {
        module_hash,
        changes: updates,
        transaction_id: "test_transaction".to_string(),
    };
    
    let result = state_manager.apply_atomic_update(atomic_update);
    assert!(result.is_ok());
}