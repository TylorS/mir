//! Type descriptor definitions and operations
//! 
//! Provides the core type system with RTTI support and schema-driven operations.

use crate::ContentHash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A hash that uniquely identifies a type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeHash(ContentHash);

impl TypeHash {
    pub fn new(hash: ContentHash) -> Self {
        Self(hash)
    }

    pub fn content_hash(&self) -> ContentHash {
        self.0
    }

    pub fn zero() -> Self {
        Self(ContentHash::zero())
    }
}

impl fmt::Display for TypeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeHash({})", self.0)
    }
}

/// Core type descriptor trait with RTTI support
pub trait TypeDescriptor: Send + Sync {
    /// Get the schema hash for this type
    fn schema_hash(&self) -> ContentHash;
    
    /// Get the type name for debugging
    fn type_name(&self) -> &str;
    
    /// Check if this type is compatible with another
    fn is_compatible_with(&self, other: &dyn TypeDescriptor) -> bool;
    
    /// Get metadata about this type
    fn metadata(&self) -> TypeMetadata;
}

/// Metadata about a type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMetadata {
    pub name: String,
    pub size_hint: Option<usize>,
    pub alignment: Option<usize>,
    pub is_copy: bool,
    pub is_send: bool,
    pub is_sync: bool,
    pub fields: Vec<FieldMetadata>,
}

/// Metadata about a field in a composite type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub name: String,
    pub type_hash: TypeHash,
    pub offset: Option<usize>,
    pub is_optional: bool,
}

/// Basic type descriptor implementation
#[derive(Debug, Clone)]
pub struct BasicTypeDescriptor {
    name: String,
    schema_hash: ContentHash,
    metadata: TypeMetadata,
}

impl BasicTypeDescriptor {
    pub fn new(name: String, data: &[u8]) -> Self {
        let schema_hash = ContentHash::new(data);
        let metadata = TypeMetadata {
            name: name.clone(),
            size_hint: None,
            alignment: None,
            is_copy: false,
            is_send: true,
            is_sync: true,
            fields: Vec::new(),
        };
        
        Self {
            name,
            schema_hash,
            metadata,
        }
    }

    pub fn with_metadata(mut self, metadata: TypeMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

impl TypeDescriptor for BasicTypeDescriptor {
    fn schema_hash(&self) -> ContentHash {
        self.schema_hash
    }

    fn type_name(&self) -> &str {
        &self.name
    }

    fn is_compatible_with(&self, other: &dyn TypeDescriptor) -> bool {
        self.schema_hash() == other.schema_hash()
    }

    fn metadata(&self) -> TypeMetadata {
        self.metadata.clone()
    }
}

/// Registry for type descriptors with RTTI support
pub struct TypeDescriptorRegistry {
    types: HashMap<TypeHash, Box<dyn TypeDescriptor>>,
    name_to_hash: HashMap<String, TypeHash>,
}

impl TypeDescriptorRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            name_to_hash: HashMap::new(),
        }
    }

    pub fn register(&mut self, descriptor: Box<dyn TypeDescriptor>) -> TypeHash {
        let hash = TypeHash::new(descriptor.schema_hash());
        let name = descriptor.type_name().to_string();
        
        self.name_to_hash.insert(name, hash);
        self.types.insert(hash, descriptor);
        
        hash
    }

    pub fn get(&self, hash: TypeHash) -> Option<&dyn TypeDescriptor> {
        self.types.get(&hash).map(|d| d.as_ref())
    }

    pub fn get_by_name(&self, name: &str) -> Option<&dyn TypeDescriptor> {
        self.name_to_hash.get(name)
            .and_then(|hash| self.get(*hash))
    }

    pub fn contains(&self, hash: TypeHash) -> bool {
        self.types.contains_key(&hash)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn type_names(&self) -> Vec<String> {
        self.name_to_hash.keys().cloned().collect()
    }
}

impl Default for TypeDescriptorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_hash_creation() {
        let content_hash = ContentHash::new(b"test type");
        let type_hash = TypeHash::new(content_hash);
        
        assert_eq!(type_hash.content_hash(), content_hash);
    }

    #[test]
    fn test_type_hash_zero() {
        let zero_hash = TypeHash::zero();
        assert_eq!(zero_hash.content_hash(), ContentHash::zero());
    }

    #[test]
    fn test_type_hash_display() {
        let content_hash = ContentHash::new(b"test");
        let type_hash = TypeHash::new(content_hash);
        let display = format!("{}", type_hash);
        
        assert!(display.starts_with("TypeHash("));
        assert!(display.contains(&content_hash.to_hex()));
    }

    #[test]
    fn test_basic_type_descriptor_creation() {
        let descriptor = BasicTypeDescriptor::new("TestType".to_string(), b"test data");
        
        assert_eq!(descriptor.type_name(), "TestType");
        assert_eq!(descriptor.schema_hash(), ContentHash::new(b"test data"));
    }

    #[test]
    fn test_basic_type_descriptor_with_metadata() {
        let metadata = TypeMetadata {
            name: "TestType".to_string(),
            size_hint: Some(64),
            alignment: Some(8),
            is_copy: true,
            is_send: true,
            is_sync: false,
            fields: vec![
                FieldMetadata {
                    name: "field1".to_string(),
                    type_hash: TypeHash::zero(),
                    offset: Some(0),
                    is_optional: false,
                }
            ],
        };

        let descriptor = BasicTypeDescriptor::new("TestType".to_string(), b"test data")
            .with_metadata(metadata.clone());

        let retrieved_metadata = descriptor.metadata();
        assert_eq!(retrieved_metadata.name, metadata.name);
        assert_eq!(retrieved_metadata.size_hint, metadata.size_hint);
        assert_eq!(retrieved_metadata.alignment, metadata.alignment);
        assert_eq!(retrieved_metadata.is_copy, metadata.is_copy);
        assert_eq!(retrieved_metadata.fields.len(), 1);
    }

    #[test]
    fn test_type_descriptor_compatibility() {
        let desc1 = BasicTypeDescriptor::new("Type1".to_string(), b"same data");
        let desc2 = BasicTypeDescriptor::new("Type2".to_string(), b"same data");
        let desc3 = BasicTypeDescriptor::new("Type3".to_string(), b"different data");

        assert!(desc1.is_compatible_with(&desc2));
        assert!(desc2.is_compatible_with(&desc1));
        assert!(!desc1.is_compatible_with(&desc3));
        assert!(!desc3.is_compatible_with(&desc1));
    }

    #[test]
    fn test_type_descriptor_registry_registration() {
        let mut registry = TypeDescriptorRegistry::new();
        let descriptor = Box::new(BasicTypeDescriptor::new("TestType".to_string(), b"test data"));
        
        let hash = registry.register(descriptor);
        
        assert!(registry.contains(hash));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_type_descriptor_registry_retrieval() {
        let mut registry = TypeDescriptorRegistry::new();
        let descriptor = Box::new(BasicTypeDescriptor::new("TestType".to_string(), b"test data"));
        let expected_hash = TypeHash::new(ContentHash::new(b"test data"));
        
        let hash = registry.register(descriptor);
        
        assert_eq!(hash, expected_hash);
        
        let retrieved = registry.get(hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().type_name(), "TestType");
    }

    #[test]
    fn test_type_descriptor_registry_by_name() {
        let mut registry = TypeDescriptorRegistry::new();
        let descriptor = Box::new(BasicTypeDescriptor::new("TestType".to_string(), b"test data"));
        
        registry.register(descriptor);
        
        let retrieved = registry.get_by_name("TestType");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().type_name(), "TestType");
        
        let not_found = registry.get_by_name("NonExistentType");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_type_descriptor_registry_type_names() {
        let mut registry = TypeDescriptorRegistry::new();
        
        registry.register(Box::new(BasicTypeDescriptor::new("Type1".to_string(), b"data1")));
        registry.register(Box::new(BasicTypeDescriptor::new("Type2".to_string(), b"data2")));
        registry.register(Box::new(BasicTypeDescriptor::new("Type3".to_string(), b"data3")));
        
        let names = registry.type_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Type1".to_string()));
        assert!(names.contains(&"Type2".to_string()));
        assert!(names.contains(&"Type3".to_string()));
    }

    #[test]
    fn test_type_descriptor_registry_empty() {
        let registry = TypeDescriptorRegistry::new();
        
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.type_names().is_empty());
    }

    #[test]
    fn test_type_descriptor_registry_multiple_registrations() {
        let mut registry = TypeDescriptorRegistry::new();
        
        let desc1 = Box::new(BasicTypeDescriptor::new("Type1".to_string(), b"data1"));
        let desc2 = Box::new(BasicTypeDescriptor::new("Type2".to_string(), b"data2"));
        
        let hash1 = registry.register(desc1);
        let hash2 = registry.register(desc2);
        
        assert_ne!(hash1, hash2);
        assert_eq!(registry.len(), 2);
        
        assert!(registry.get(hash1).is_some());
        assert!(registry.get(hash2).is_some());
        assert_eq!(registry.get(hash1).unwrap().type_name(), "Type1");
        assert_eq!(registry.get(hash2).unwrap().type_name(), "Type2");
    }

    #[test]
    fn test_field_metadata() {
        let field = FieldMetadata {
            name: "test_field".to_string(),
            type_hash: TypeHash::new(ContentHash::new(b"field_type")),
            offset: Some(16),
            is_optional: true,
        };

        assert_eq!(field.name, "test_field");
        assert_eq!(field.offset, Some(16));
        assert!(field.is_optional);
    }

    #[test]
    fn test_type_metadata_serialization() {
        let metadata = TypeMetadata {
            name: "TestType".to_string(),
            size_hint: Some(32),
            alignment: Some(4),
            is_copy: true,
            is_send: true,
            is_sync: false,
            fields: vec![
                FieldMetadata {
                    name: "field1".to_string(),
                    type_hash: TypeHash::zero(),
                    offset: Some(0),
                    is_optional: false,
                }
            ],
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: TypeMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.size_hint, deserialized.size_hint);
        assert_eq!(metadata.alignment, deserialized.alignment);
        assert_eq!(metadata.is_copy, deserialized.is_copy);
        assert_eq!(metadata.fields.len(), deserialized.fields.len());
    }
}