//! Composite data structures for the MIR type system

use crate::{ContentHash, TypeHash, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Garbage collection header for GC-backed structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GCHeader {
    /// Mark bit for garbage collection
    pub marked: bool,
    /// Reference count for reference counting GC
    pub ref_count: usize,
    /// Generation for generational GC
    pub generation: u8,
    /// Size in bytes for memory management
    pub size: usize,
}

impl GCHeader {
    pub fn new(size: usize) -> Self {
        GCHeader {
            marked: false,
            ref_count: 1,
            generation: 0,
            size,
        }
    }
    
    pub fn increment_ref(&mut self) {
        self.ref_count += 1;
    }
    
    pub fn decrement_ref(&mut self) -> bool {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count == 0
    }
    
    pub fn mark(&mut self) {
        self.marked = true;
    }
    
    pub fn unmark(&mut self) {
        self.marked = false;
    }
    
    pub fn is_marked(&self) -> bool {
        self.marked
    }
}

/// Struct type with named fields and GC header
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructType {
    /// Named fields with their values
    pub fields: HashMap<String, Value>,
    /// Schema hash for this struct type
    pub schema_hash: ContentHash,
    /// Garbage collection header
    pub gc_header: GCHeader,
}

impl StructType {
    pub fn new(schema_hash: ContentHash) -> Self {
        let fields = HashMap::new();
        let size = std::mem::size_of::<HashMap<String, Value>>();
        
        StructType {
            fields,
            schema_hash,
            gc_header: GCHeader::new(size),
        }
    }
    
    pub fn with_fields(fields: HashMap<String, Value>, schema_hash: ContentHash) -> Self {
        let size = std::mem::size_of::<HashMap<String, Value>>() + 
                   fields.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
        
        StructType {
            fields,
            schema_hash,
            gc_header: GCHeader::new(size),
        }
    }
}

/// Struct operations trait
pub trait StructOperations {
    fn get_field(&self, name: &str) -> Option<&Value>;
    fn set_field(&mut self, name: String, value: Value) -> Result<(), StructError>;
    fn has_field(&self, name: &str) -> bool;
    fn field_names(&self) -> Vec<&str>;
    fn field_count(&self) -> usize;
    fn clone_with_field(&self, name: String, value: Value) -> Self;
    fn remove_field(&mut self, name: &str) -> Option<Value>;
}

impl StructOperations for StructType {
    fn get_field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }
    
    fn set_field(&mut self, name: String, value: Value) -> Result<(), StructError> {
        let _old_size = self.gc_header.size;
        self.fields.insert(name, value);
        
        // Update size estimate
        let new_size = std::mem::size_of::<HashMap<String, Value>>() + 
                       self.fields.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
        self.gc_header.size = new_size;
        
        Ok(())
    }
    
    fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }
    
    fn field_names(&self) -> Vec<&str> {
        self.fields.keys().map(|s| s.as_str()).collect()
    }
    
    fn field_count(&self) -> usize {
        self.fields.len()
    }
    
    fn clone_with_field(&self, name: String, value: Value) -> Self {
        let mut new_fields = self.fields.clone();
        new_fields.insert(name, value);
        
        StructType::with_fields(new_fields, self.schema_hash)
    }
    
    fn remove_field(&mut self, name: &str) -> Option<Value> {
        let result = self.fields.remove(name);
        
        // Update size estimate
        let new_size = std::mem::size_of::<HashMap<String, Value>>() + 
                       self.fields.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
        self.gc_header.size = new_size;
        
        result
    }
}

/// Array type with homogeneous elements and operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    /// Array elements
    pub elements: Vec<Value>,
    /// Type of elements in the array
    pub element_type: TypeHash,
    /// Current capacity
    pub capacity: usize,
    /// Garbage collection header
    pub gc_header: GCHeader,
}

impl ArrayType {
    pub fn new(element_type: TypeHash) -> Self {
        let elements = Vec::new();
        let capacity = 0;
        let size = std::mem::size_of::<Vec<Value>>();
        
        ArrayType {
            elements,
            element_type,
            capacity,
            gc_header: GCHeader::new(size),
        }
    }
    
    pub fn with_capacity(element_type: TypeHash, capacity: usize) -> Self {
        let elements = Vec::with_capacity(capacity);
        let size = std::mem::size_of::<Vec<Value>>() + capacity * std::mem::size_of::<Value>();
        
        ArrayType {
            elements,
            element_type,
            capacity,
            gc_header: GCHeader::new(size),
        }
    }
    
    pub fn from_elements(elements: Vec<Value>, element_type: TypeHash) -> Self {
        let capacity = elements.capacity();
        let size = std::mem::size_of::<Vec<Value>>() + capacity * std::mem::size_of::<Value>();
        
        ArrayType {
            elements,
            element_type,
            capacity,
            gc_header: GCHeader::new(size),
        }
    }
}

/// Array operations trait
pub trait ArrayOperations {
    fn get(&self, index: usize) -> Option<&Value>;
    fn set(&mut self, index: usize, value: Value) -> Result<(), ArrayError>;
    fn push(&mut self, value: Value) -> Result<(), ArrayError>;
    fn pop(&mut self) -> Option<Value>;
    fn length(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn slice(&self, start: usize, end: usize) -> Result<Self, ArrayError> where Self: Sized;
    fn insert(&mut self, index: usize, value: Value) -> Result<(), ArrayError>;
    fn remove(&mut self, index: usize) -> Result<Value, ArrayError>;
    fn clear(&mut self);
}

impl ArrayOperations for ArrayType {
    fn get(&self, index: usize) -> Option<&Value> {
        self.elements.get(index)
    }
    
    fn set(&mut self, index: usize, value: Value) -> Result<(), ArrayError> {
        if index >= self.elements.len() {
            return Err(ArrayError::IndexOutOfBounds);
        }
        
        // TODO: Type check against element_type
        self.elements[index] = value;
        Ok(())
    }
    
    fn push(&mut self, value: Value) -> Result<(), ArrayError> {
        // TODO: Type check against element_type
        self.elements.push(value);
        
        // Update capacity and size if needed
        if self.elements.capacity() > self.capacity {
            self.capacity = self.elements.capacity();
            self.gc_header.size = std::mem::size_of::<Vec<Value>>() + 
                                  self.capacity * std::mem::size_of::<Value>();
        }
        
        Ok(())
    }
    
    fn pop(&mut self) -> Option<Value> {
        self.elements.pop()
    }
    
    fn length(&self) -> usize {
        self.elements.len()
    }
    
    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    
    fn slice(&self, start: usize, end: usize) -> Result<Self, ArrayError> {
        if start > end || end > self.elements.len() {
            return Err(ArrayError::InvalidRange);
        }
        
        let sliced_elements = self.elements[start..end].to_vec();
        Ok(ArrayType::from_elements(sliced_elements, self.element_type))
    }
    
    fn insert(&mut self, index: usize, value: Value) -> Result<(), ArrayError> {
        if index > self.elements.len() {
            return Err(ArrayError::IndexOutOfBounds);
        }
        
        // TODO: Type check against element_type
        self.elements.insert(index, value);
        
        // Update capacity and size if needed
        if self.elements.capacity() > self.capacity {
            self.capacity = self.elements.capacity();
            self.gc_header.size = std::mem::size_of::<Vec<Value>>() + 
                                  self.capacity * std::mem::size_of::<Value>();
        }
        
        Ok(())
    }
    
    fn remove(&mut self, index: usize) -> Result<Value, ArrayError> {
        if index >= self.elements.len() {
            return Err(ArrayError::IndexOutOfBounds);
        }
        
        Ok(self.elements.remove(index))
    }
    
    fn clear(&mut self) {
        self.elements.clear();
    }
}

/// Record type with heterogeneous fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordType {
    /// Ordered fields with names, types, and values
    pub fields: Vec<(String, TypeHash, Value)>,
    /// Schema hash for this record type
    pub schema_hash: ContentHash,
    /// Garbage collection header
    pub gc_header: GCHeader,
}

impl RecordType {
    pub fn new(schema_hash: ContentHash) -> Self {
        let fields = Vec::new();
        let size = std::mem::size_of::<Vec<(String, TypeHash, Value)>>();
        
        RecordType {
            fields,
            schema_hash,
            gc_header: GCHeader::new(size),
        }
    }
    
    pub fn from_fields(fields: Vec<(String, TypeHash, Value)>, schema_hash: ContentHash) -> Self {
        let size = std::mem::size_of::<Vec<(String, TypeHash, Value)>>() + 
                   fields.len() * std::mem::size_of::<(String, TypeHash, Value)>();
        
        RecordType {
            fields,
            schema_hash,
            gc_header: GCHeader::new(size),
        }
    }
}

/// Record operations trait
pub trait RecordOperations {
    fn get_by_name(&self, name: &str) -> Option<&Value>;
    fn get_by_index(&self, index: usize) -> Option<&Value>;
    fn get_field_type(&self, name: &str) -> Option<TypeHash>;
    fn set_by_name(&mut self, name: &str, value: Value) -> Result<(), RecordError>;
    fn set_by_index(&mut self, index: usize, value: Value) -> Result<(), RecordError>;
    fn field_count(&self) -> usize;
    fn field_names(&self) -> Vec<&str>;
    fn extend(&self, other: &Self) -> Self;
    fn has_field(&self, name: &str) -> bool;
}

impl RecordOperations for RecordType {
    fn get_by_name(&self, name: &str) -> Option<&Value> {
        self.fields.iter()
            .find(|(field_name, _, _)| field_name == name)
            .map(|(_, _, value)| value)
    }
    
    fn get_by_index(&self, index: usize) -> Option<&Value> {
        self.fields.get(index).map(|(_, _, value)| value)
    }
    
    fn get_field_type(&self, name: &str) -> Option<TypeHash> {
        self.fields.iter()
            .find(|(field_name, _, _)| field_name == name)
            .map(|(_, type_hash, _)| *type_hash)
    }
    
    fn set_by_name(&mut self, name: &str, value: Value) -> Result<(), RecordError> {
        if let Some((_, _, field_value)) = self.fields.iter_mut()
            .find(|(field_name, _, _)| field_name == name) {
            *field_value = value;
            Ok(())
        } else {
            Err(RecordError::FieldNotFound(name.to_string()))
        }
    }
    
    fn set_by_index(&mut self, index: usize, value: Value) -> Result<(), RecordError> {
        if let Some((_, _, field_value)) = self.fields.get_mut(index) {
            *field_value = value;
            Ok(())
        } else {
            Err(RecordError::IndexOutOfBounds)
        }
    }
    
    fn field_count(&self) -> usize {
        self.fields.len()
    }
    
    fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|(name, _, _)| name.as_str()).collect()
    }
    
    fn extend(&self, other: &Self) -> Self {
        let mut new_fields = self.fields.clone();
        new_fields.extend(other.fields.clone());
        
        RecordType::from_fields(new_fields, self.schema_hash)
    }
    
    fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|(field_name, _, _)| field_name == name)
    }
}

/// Union type with RTTI support for runtime type checking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnionType {
    /// Current variant tag
    pub variant_tag: u32,
    /// Data for the current variant
    pub variant_data: Value,
    /// All possible types in this union
    pub possible_types: Vec<TypeHash>,
    /// Garbage collection header
    pub gc_header: GCHeader,
}

impl UnionType {
    pub fn new(variant_tag: u32, variant_data: Value, possible_types: Vec<TypeHash>) -> Self {
        let size = std::mem::size_of::<UnionType>();
        
        UnionType {
            variant_tag,
            variant_data,
            possible_types,
            gc_header: GCHeader::new(size),
        }
    }
}

/// Union operations trait
pub trait UnionOperations {
    fn get_variant_tag(&self) -> u32;
    fn get_variant_data(&self) -> &Value;
    fn get_possible_types(&self) -> &[TypeHash];
    fn is_variant(&self, type_hash: TypeHash) -> bool;
    fn cast_to_variant(&self, type_hash: TypeHash) -> Result<Value, UnionError>;
    fn set_variant(&mut self, tag: u32, data: Value, type_hash: TypeHash) -> Result<(), UnionError>;
}

impl UnionOperations for UnionType {
    fn get_variant_tag(&self) -> u32 {
        self.variant_tag
    }
    
    fn get_variant_data(&self) -> &Value {
        &self.variant_data
    }
    
    fn get_possible_types(&self) -> &[TypeHash] {
        &self.possible_types
    }
    
    fn is_variant(&self, type_hash: TypeHash) -> bool {
        self.possible_types.contains(&type_hash)
    }
    
    fn cast_to_variant(&self, type_hash: TypeHash) -> Result<Value, UnionError> {
        if !self.is_variant(type_hash) {
            return Err(UnionError::InvalidVariant);
        }
        
        // TODO: Implement proper type checking and casting
        // For now, just return the current data if the type matches
        Ok(self.variant_data.clone())
    }
    
    fn set_variant(&mut self, tag: u32, data: Value, type_hash: TypeHash) -> Result<(), UnionError> {
        if !self.is_variant(type_hash) {
            return Err(UnionError::InvalidVariant);
        }
        
        self.variant_tag = tag;
        self.variant_data = data;
        Ok(())
    }
}

/// Error types for composite data structures

#[derive(Debug, Clone, PartialEq)]
pub enum StructError {
    FieldNotFound(String),
    TypeMismatch,
    InvalidOperation(String),
}

impl fmt::Display for StructError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StructError::FieldNotFound(name) => write!(f, "Field '{}' not found", name),
            StructError::TypeMismatch => write!(f, "Type mismatch in struct operation"),
            StructError::InvalidOperation(msg) => write!(f, "Invalid struct operation: {}", msg),
        }
    }
}

impl std::error::Error for StructError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayError {
    IndexOutOfBounds,
    InvalidRange,
    TypeMismatch,
    CapacityExceeded,
}

impl fmt::Display for ArrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayError::IndexOutOfBounds => write!(f, "Array index out of bounds"),
            ArrayError::InvalidRange => write!(f, "Invalid range for array operation"),
            ArrayError::TypeMismatch => write!(f, "Type mismatch in array operation"),
            ArrayError::CapacityExceeded => write!(f, "Array capacity exceeded"),
        }
    }
}

impl std::error::Error for ArrayError {}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordError {
    FieldNotFound(String),
    IndexOutOfBounds,
    TypeMismatch,
    InvalidOperation(String),
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordError::FieldNotFound(name) => write!(f, "Field '{}' not found in record", name),
            RecordError::IndexOutOfBounds => write!(f, "Record index out of bounds"),
            RecordError::TypeMismatch => write!(f, "Type mismatch in record operation"),
            RecordError::InvalidOperation(msg) => write!(f, "Invalid record operation: {}", msg),
        }
    }
}

impl std::error::Error for RecordError {}

#[derive(Debug, Clone, PartialEq)]
pub enum UnionError {
    InvalidVariant,
    TypeMismatch,
    CastError(String),
}

impl fmt::Display for UnionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnionError::InvalidVariant => write!(f, "Invalid variant for union type"),
            UnionError::TypeMismatch => write!(f, "Type mismatch in union operation"),
            UnionError::CastError(msg) => write!(f, "Cast error: {}", msg),
        }
    }
}

impl std::error::Error for UnionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    
    #[test]
    fn test_gc_header() {
        let mut header = GCHeader::new(100);
        assert_eq!(header.ref_count, 1);
        assert!(!header.is_marked());
        
        header.increment_ref();
        assert_eq!(header.ref_count, 2);
        
        assert!(!header.decrement_ref());
        assert_eq!(header.ref_count, 1);
        
        assert!(header.decrement_ref());
        assert_eq!(header.ref_count, 0);
        
        header.mark();
        assert!(header.is_marked());
    }
    
    #[test]
    fn test_struct_operations() {
        let mut s = StructType::new(ContentHash::new(b"test_struct"));
        
        assert_eq!(s.field_count(), 0);
        assert!(!s.has_field("name"));
        
        s.set_field("name".to_string(), Value::String("test".to_string())).unwrap();
        s.set_field("age".to_string(), Value::I32(25)).unwrap();
        
        assert_eq!(s.field_count(), 2);
        assert!(s.has_field("name"));
        assert!(s.has_field("age"));
        
        if let Some(Value::String(name)) = s.get_field("name") {
            assert_eq!(name, "test");
        } else {
            panic!("Expected string value");
        }
        
        let removed = s.remove_field("age");
        assert!(removed.is_some());
        assert_eq!(s.field_count(), 1);
    }
    
    #[test]
    fn test_array_operations() {
        let element_type = TypeHash::new(ContentHash::new(b"i32"));
        let mut arr = ArrayType::new(element_type);
        
        assert!(arr.is_empty());
        assert_eq!(arr.length(), 0);
        
        arr.push(Value::I32(1)).unwrap();
        arr.push(Value::I32(2)).unwrap();
        arr.push(Value::I32(3)).unwrap();
        
        assert_eq!(arr.length(), 3);
        assert!(!arr.is_empty());
        
        if let Some(Value::I32(val)) = arr.get(1) {
            assert_eq!(*val, 2);
        } else {
            panic!("Expected i32 value");
        }
        
        arr.set(1, Value::I32(20)).unwrap();
        if let Some(Value::I32(val)) = arr.get(1) {
            assert_eq!(*val, 20);
        }
        
        let popped = arr.pop();
        assert!(popped.is_some());
        assert_eq!(arr.length(), 2);
    }
    
    #[test]
    fn test_record_operations() {
        let fields = vec![
            ("name".to_string(), TypeHash::new(ContentHash::new(b"string")), Value::String("Alice".to_string())),
            ("age".to_string(), TypeHash::new(ContentHash::new(b"i32")), Value::I32(30)),
        ];
        
        let mut record = RecordType::from_fields(fields, ContentHash::new(b"person"));
        
        assert_eq!(record.field_count(), 2);
        assert!(record.has_field("name"));
        assert!(record.has_field("age"));
        
        if let Some(Value::String(name)) = record.get_by_name("name") {
            assert_eq!(name, "Alice");
        } else {
            panic!("Expected string value");
        }
        
        if let Some(Value::I32(age)) = record.get_by_index(1) {
            assert_eq!(*age, 30);
        } else {
            panic!("Expected i32 value");
        }
        
        record.set_by_name("age", Value::I32(31)).unwrap();
        if let Some(Value::I32(age)) = record.get_by_name("age") {
            assert_eq!(*age, 31);
        }
    }
    
    #[test]
    fn test_union_operations() {
        let possible_types = vec![
            TypeHash::new(ContentHash::new(b"i32")),
            TypeHash::new(ContentHash::new(b"string")),
        ];
        
        let mut union = UnionType::new(0, Value::I32(42), possible_types);
        
        assert_eq!(union.get_variant_tag(), 0);
        if let Value::I32(val) = union.get_variant_data() {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected i32 value");
        }
        
        let string_type = TypeHash::new(ContentHash::new(b"string"));
        assert!(union.is_variant(string_type));
        
        union.set_variant(1, Value::String("hello".to_string()), string_type).unwrap();
        assert_eq!(union.get_variant_tag(), 1);
        if let Value::String(val) = union.get_variant_data() {
            assert_eq!(val, "hello");
        } else {
            panic!("Expected string value");
        }
    }
}