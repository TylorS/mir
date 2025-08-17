//! Built-in migration functions for common schema changes
//!
//! This module provides concrete implementations of migration functions
//! for common schema evolution patterns.

use crate::state_migration::{
    MigrationFunction, AutoMigrationGenerator, MigrationContext, MigrationError,
    SchemaTransition,
};
use crate::state::ComponentState;
use crate::change_analysis::{SchemaChange, SchemaChangeType};

/// Type alias for field transformation function
type FieldTransformFn = Box<dyn Fn(&Value) -> Result<Value, String> + Send + Sync>;

/// Type alias for component migration function
type ComponentMigrationFn = Box<dyn Fn(&ComponentState, TypeHash, TypeHash, &MigrationContext) -> Result<ComponentState, MigrationError> + Send + Sync>;

use mir_types::{TypeHash, Value};
use std::time::Duration;

/// Migration function for adding new fields with default values
pub struct AddFieldMigration {
    /// Name of the field being added
    field_name: String,
    /// Default value for the new field
    default_value: Value,
}

impl AddFieldMigration {
    /// Create a new add field migration
    pub fn new(field_name: String, default_value: Value) -> Self {
        AddFieldMigration {
            field_name,
            default_value,
        }
    }
}

impl MigrationFunction for AddFieldMigration {
    fn name(&self) -> &str {
        "add_field"
    }

    fn migrate(
        &self,
        old_state: &ComponentState,
        _old_schema: TypeHash,
        new_schema: TypeHash,
        _context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError> {
        let mut new_state = old_state.clone();
        new_state.schema_hash = new_schema;
        
        // Add the new field with default value
        new_state.values.insert(self.field_name.clone(), self.default_value.clone());
        
        Ok(new_state)
    }

    fn can_migrate(&self, transition: &SchemaTransition) -> bool {
        // This migration can handle any transition where a field is being added
        // In a real implementation, we would analyze the schema difference
        transition.from_schema != transition.to_schema
    }

    fn estimated_duration(&self, _state_size: usize) -> Duration {
        Duration::from_millis(10) // Very fast operation
    }

    fn is_reversible(&self) -> bool {
        true // Can be reversed by removing the field
    }
}

/// Migration function for removing fields
pub struct RemoveFieldMigration {
    /// Name of the field being removed
    field_name: String,
}

impl RemoveFieldMigration {
    /// Create a new remove field migration
    pub fn new(field_name: String) -> Self {
        RemoveFieldMigration { field_name }
    }
}

impl MigrationFunction for RemoveFieldMigration {
    fn name(&self) -> &str {
        "remove_field"
    }

    fn migrate(
        &self,
        old_state: &ComponentState,
        _old_schema: TypeHash,
        new_schema: TypeHash,
        _context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError> {
        let mut new_state = old_state.clone();
        new_state.schema_hash = new_schema;
        
        // Remove the field
        new_state.values.remove(&self.field_name);
        
        Ok(new_state)
    }

    fn can_migrate(&self, transition: &SchemaTransition) -> bool {
        transition.from_schema != transition.to_schema
    }

    fn estimated_duration(&self, _state_size: usize) -> Duration {
        Duration::from_millis(5)
    }

    fn is_reversible(&self) -> bool {
        false // Cannot restore removed data without backup
    }
}

/// Migration function for renaming fields
pub struct RenameFieldMigration {
    /// Old field name
    old_name: String,
    /// New field name
    new_name: String,
}

impl RenameFieldMigration {
    /// Create a new rename field migration
    pub fn new(old_name: String, new_name: String) -> Self {
        RenameFieldMigration { old_name, new_name }
    }
}

impl MigrationFunction for RenameFieldMigration {
    fn name(&self) -> &str {
        "rename_field"
    }

    fn migrate(
        &self,
        old_state: &ComponentState,
        _old_schema: TypeHash,
        new_schema: TypeHash,
        _context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError> {
        let mut new_state = old_state.clone();
        new_state.schema_hash = new_schema;
        
        // Rename the field
        if let Some(value) = new_state.values.remove(&self.old_name) {
            new_state.values.insert(self.new_name.clone(), value);
        }
        
        Ok(new_state)
    }

    fn can_migrate(&self, transition: &SchemaTransition) -> bool {
        transition.from_schema != transition.to_schema
    }

    fn estimated_duration(&self, _state_size: usize) -> Duration {
        Duration::from_millis(10)
    }

    fn is_reversible(&self) -> bool {
        true // Can be reversed by renaming back
    }
}

/// Migration function for transforming field values
pub struct TransformFieldMigration {
    /// Name of the field to transform
    field_name: String,
    /// Transformation function
    transform_fn: FieldTransformFn,
}

impl TransformFieldMigration {
    /// Create a new transform field migration
    pub fn new<F>(field_name: String, transform_fn: F) -> Self
    where
        F: Fn(&Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        TransformFieldMigration {
            field_name,
            transform_fn: Box::new(transform_fn),
        }
    }
}

impl MigrationFunction for TransformFieldMigration {
    fn name(&self) -> &str {
        "transform_field"
    }

    fn migrate(
        &self,
        old_state: &ComponentState,
        _old_schema: TypeHash,
        new_schema: TypeHash,
        _context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError> {
        let mut new_state = old_state.clone();
        new_state.schema_hash = new_schema;
        
        // Transform the field value
        if let Some(old_value) = new_state.values.get(&self.field_name) {
            match (self.transform_fn)(old_value) {
                Ok(new_value) => {
                    new_state.values.insert(self.field_name.clone(), new_value);
                }
                Err(e) => {
                    return Err(MigrationError::ExecutionFailed(format!(
                        "Failed to transform field '{}': {}",
                        self.field_name, e
                    )));
                }
            }
        }
        
        Ok(new_state)
    }

    fn can_migrate(&self, transition: &SchemaTransition) -> bool {
        transition.from_schema != transition.to_schema
    }

    fn estimated_duration(&self, state_size: usize) -> Duration {
        // Transformation time depends on state size
        Duration::from_millis((state_size / 1000).max(10) as u64)
    }

    fn is_reversible(&self) -> bool {
        false // Transformations are generally not reversible
    }
}

/// Migration function for copying state without changes (identity migration)
pub struct IdentityMigration;

impl MigrationFunction for IdentityMigration {
    fn name(&self) -> &str {
        "identity"
    }

    fn migrate(
        &self,
        old_state: &ComponentState,
        _old_schema: TypeHash,
        new_schema: TypeHash,
        _context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError> {
        let mut new_state = old_state.clone();
        new_state.schema_hash = new_schema;
        Ok(new_state)
    }

    fn can_migrate(&self, _transition: &SchemaTransition) -> bool {
        true // Can handle any transition
    }

    fn estimated_duration(&self, _state_size: usize) -> Duration {
        Duration::from_millis(1) // Fastest possible migration
    }

    fn is_reversible(&self) -> bool {
        true // Identity is always reversible
    }
}

/// Automatic migration generator for backward compatible changes
pub struct BackwardCompatibleGenerator;

impl AutoMigrationGenerator for BackwardCompatibleGenerator {
    fn name(&self) -> &str {
        "backward_compatible"
    }

    fn generate_migration(
        &self,
        _transition: &SchemaTransition,
        schema_change: &SchemaChange,
    ) -> Result<Box<dyn MigrationFunction>, MigrationError> {
        match schema_change.change_type {
            SchemaChangeType::Compatible | SchemaChangeType::BackwardCompatible => {
                // For backward compatible changes, use identity migration
                Ok(Box::new(IdentityMigration))
            }
            _ => Err(MigrationError::NoAutoMigrationAvailable(_transition.clone())),
        }
    }

    fn can_generate(&self, schema_change: &SchemaChange) -> bool {
        matches!(
            schema_change.change_type,
            SchemaChangeType::Compatible | SchemaChangeType::BackwardCompatible
        )
    }
}

/// Automatic migration generator for field additions
pub struct AddFieldGenerator;

impl AutoMigrationGenerator for AddFieldGenerator {
    fn name(&self) -> &str {
        "add_field_generator"
    }

    fn generate_migration(
        &self,
        _transition: &SchemaTransition,
        _schema_change: &SchemaChange,
    ) -> Result<Box<dyn MigrationFunction>, MigrationError> {
        // In a real implementation, we would analyze the schema change
        // to determine what field was added and what default value to use
        
        // For now, create a generic field addition with a default value
        let migration = AddFieldMigration::new(
            "new_field".to_string(),
            Value::I32(0), // Default value
        );
        
        Ok(Box::new(migration))
    }

    fn can_generate(&self, schema_change: &SchemaChange) -> bool {
        // This generator can handle compatible changes that likely involve field additions
        matches!(schema_change.change_type, SchemaChangeType::Compatible)
    }
}

/// User-defined migration function that executes custom logic
pub struct UserDefinedMigration {
    /// Name of the migration
    name: String,
    /// Custom migration logic
    migration_fn: ComponentMigrationFn,
    /// Whether this migration is reversible
    reversible: bool,
    /// Estimated duration function
    duration_fn: Box<dyn Fn(usize) -> Duration + Send + Sync>,
}

impl UserDefinedMigration {
    /// Create a new user-defined migration
    pub fn new<F, D>(
        name: String,
        migration_fn: F,
        reversible: bool,
        duration_fn: D,
    ) -> Self
    where
        F: Fn(&ComponentState, TypeHash, TypeHash, &MigrationContext) -> Result<ComponentState, MigrationError> + Send + Sync + 'static,
        D: Fn(usize) -> Duration + Send + Sync + 'static,
    {
        UserDefinedMigration {
            name,
            migration_fn: Box::new(migration_fn),
            reversible,
            duration_fn: Box::new(duration_fn),
        }
    }
}

impl MigrationFunction for UserDefinedMigration {
    fn name(&self) -> &str {
        &self.name
    }

    fn migrate(
        &self,
        old_state: &ComponentState,
        old_schema: TypeHash,
        new_schema: TypeHash,
        context: &MigrationContext,
    ) -> Result<ComponentState, MigrationError> {
        (self.migration_fn)(old_state, old_schema, new_schema, context)
    }

    fn can_migrate(&self, _transition: &SchemaTransition) -> bool {
        true // User-defined migrations can handle any transition they're registered for
    }

    fn estimated_duration(&self, state_size: usize) -> Duration {
        (self.duration_fn)(state_size)
    }

    fn is_reversible(&self) -> bool {
        self.reversible
    }
}

/// Collection of built-in migration functions
pub struct BuiltinMigrations;

impl BuiltinMigrations {
    /// Get all built-in migration functions
    pub fn get_all() -> Vec<Box<dyn MigrationFunction>> {
        vec![
            Box::new(IdentityMigration),
            Box::new(AddFieldMigration::new("default_field".to_string(), Value::I32(0))),
            Box::new(RemoveFieldMigration::new("removed_field".to_string())),
            Box::new(RenameFieldMigration::new("old_name".to_string(), "new_name".to_string())),
        ]
    }

    /// Get all built-in auto migration generators
    pub fn get_generators() -> Vec<Box<dyn AutoMigrationGenerator>> {
        vec![
            Box::new(BackwardCompatibleGenerator),
            Box::new(AddFieldGenerator),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ComponentType;
    use mir_types::ContentHash;
    use std::collections::HashMap;

    fn create_test_component_state() -> ComponentState {
        let mut values = HashMap::new();
        values.insert("test_field".to_string(), Value::I32(42));
        
        ComponentState {
            component_name: "test_component".to_string(),
            component_type: ComponentType::StatefulFunction,
            values,
            connections: Vec::new(),
            is_stateful: true,
            schema_hash: TypeHash::new(ContentHash::new(b"test_schema")),
        }
    }

    fn create_test_context() -> MigrationContext {
        MigrationContext {
            module_id: mir_ast::NodeId::new(1),
            migration_id: ContentHash::new(b"test_migration"),
            context_data: HashMap::new(),
            config: crate::state_migration::MigrationConfig::default(),
            started_at: 0,
        }
    }

    #[test]
    fn test_identity_migration() {
        let migration = IdentityMigration;
        let old_state = create_test_component_state();
        let old_schema = TypeHash::new(ContentHash::new(b"old"));
        let new_schema = TypeHash::new(ContentHash::new(b"new"));
        let context = create_test_context();

        let result = migration.migrate(&old_state, old_schema, new_schema, &context);
        assert!(result.is_ok());
        
        let new_state = result.unwrap();
        assert_eq!(new_state.schema_hash, new_schema);
        assert_eq!(new_state.values, old_state.values);
    }

    #[test]
    fn test_add_field_migration() {
        let migration = AddFieldMigration::new("new_field".to_string(), Value::I32(100));
        let old_state = create_test_component_state();
        let old_schema = TypeHash::new(ContentHash::new(b"old"));
        let new_schema = TypeHash::new(ContentHash::new(b"new"));
        let context = create_test_context();

        let result = migration.migrate(&old_state, old_schema, new_schema, &context);
        assert!(result.is_ok());
        
        let new_state = result.unwrap();
        assert_eq!(new_state.schema_hash, new_schema);
        assert!(new_state.values.contains_key("new_field"));
        assert_eq!(new_state.values.get("new_field"), Some(&Value::I32(100)));
    }

    #[test]
    fn test_remove_field_migration() {
        let migration = RemoveFieldMigration::new("test_field".to_string());
        let old_state = create_test_component_state();
        let old_schema = TypeHash::new(ContentHash::new(b"old"));
        let new_schema = TypeHash::new(ContentHash::new(b"new"));
        let context = create_test_context();

        let result = migration.migrate(&old_state, old_schema, new_schema, &context);
        assert!(result.is_ok());
        
        let new_state = result.unwrap();
        assert_eq!(new_state.schema_hash, new_schema);
        assert!(!new_state.values.contains_key("test_field"));
    }

    #[test]
    fn test_rename_field_migration() {
        let migration = RenameFieldMigration::new("test_field".to_string(), "renamed_field".to_string());
        let old_state = create_test_component_state();
        let old_schema = TypeHash::new(ContentHash::new(b"old"));
        let new_schema = TypeHash::new(ContentHash::new(b"new"));
        let context = create_test_context();

        let result = migration.migrate(&old_state, old_schema, new_schema, &context);
        assert!(result.is_ok());
        
        let new_state = result.unwrap();
        assert_eq!(new_state.schema_hash, new_schema);
        assert!(!new_state.values.contains_key("test_field"));
        assert!(new_state.values.contains_key("renamed_field"));
        assert_eq!(new_state.values.get("renamed_field"), Some(&Value::I32(42)));
    }

    #[test]
    fn test_transform_field_migration() {
        let migration = TransformFieldMigration::new(
            "test_field".to_string(),
            |value| match value {
                Value::I32(n) => Ok(Value::I32(n * 2)),
                _ => Err("Expected I32".to_string()),
            },
        );
        
        let old_state = create_test_component_state();
        let old_schema = TypeHash::new(ContentHash::new(b"old"));
        let new_schema = TypeHash::new(ContentHash::new(b"new"));
        let context = create_test_context();

        let result = migration.migrate(&old_state, old_schema, new_schema, &context);
        assert!(result.is_ok());
        
        let new_state = result.unwrap();
        assert_eq!(new_state.schema_hash, new_schema);
        assert_eq!(new_state.values.get("test_field"), Some(&Value::I32(84)));
    }

    #[test]
    fn test_backward_compatible_generator() {
        let generator = BackwardCompatibleGenerator;
        let transition = SchemaTransition {
            from_schema: TypeHash::new(ContentHash::new(b"old")),
            to_schema: TypeHash::new(ContentHash::new(b"new")),
            component_type: "test".to_string(),
        };
        
        let schema_change = SchemaChange {
            schema_hash: ContentHash::new(b"new"),
            old_version: Some(ContentHash::new(b"old")),
            new_version: ContentHash::new(b"new"),
            change_type: SchemaChangeType::BackwardCompatible,
            migration_path: None,
            affected_modules: Vec::new(),
        };

        assert!(generator.can_generate(&schema_change));
        let result = generator.generate_migration(&transition, &schema_change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_migrations() {
        let migrations = BuiltinMigrations::get_all();
        assert!(!migrations.is_empty());
        
        let generators = BuiltinMigrations::get_generators();
        assert!(!generators.is_empty());
    }
}