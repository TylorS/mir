//! Conflict-free Replicated Data Types (CRDTs) for distributed state management
//!
//! This module provides various CRDT implementations that enable distributed systems
//! to maintain consistent state without requiring coordination between nodes.

use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

/// Node identifier for CRDT operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node_{}", self.0)
    }
}

impl NodeId {
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }
}

/// Logical timestamp for ordering operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogicalTimestamp(pub u64);

impl LogicalTimestamp {
    pub fn new(timestamp: u64) -> Self {
        LogicalTimestamp(timestamp)
    }
    
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

/// Version vector for tracking causal relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVector {
    versions: HashMap<NodeId, u64>,
}

impl Default for VersionVector {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionVector {
    pub fn new() -> Self {
        VersionVector {
            versions: HashMap::new(),
        }
    }
    
    pub fn increment(&mut self, node_id: NodeId) {
        *self.versions.entry(node_id).or_insert(0) += 1;
    }
    
    pub fn get(&self, node_id: NodeId) -> u64 {
        self.versions.get(&node_id).copied().unwrap_or(0)
    }
    
    pub fn merge(&mut self, other: &VersionVector) {
        for (&node_id, &version) in &other.versions {
            let current = self.versions.entry(node_id).or_insert(0);
            *current = (*current).max(version);
        }
    }
    
    pub fn compare(&self, other: &VersionVector) -> PartialOrdering {
        let mut less_than = false;
        let mut greater_than = false;
        
        // Check all nodes in both version vectors
        let all_nodes: HashSet<NodeId> = self.versions.keys()
            .chain(other.versions.keys())
            .copied()
            .collect();
        
        for node_id in all_nodes {
            let self_version = self.get(node_id);
            let other_version = other.get(node_id);
            
            match self_version.cmp(&other_version) {
                Ordering::Less => less_than = true,
                Ordering::Greater => greater_than = true,
                Ordering::Equal => {}
            }
        }
        
        match (less_than, greater_than) {
            (false, false) => PartialOrdering::Equal,
            (true, false) => PartialOrdering::Less,
            (false, true) => PartialOrdering::Greater,
            (true, true) => PartialOrdering::Concurrent,
        }
    }
}

/// Partial ordering for version vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialOrdering {
    Less,
    Greater,
    Equal,
    Concurrent,
}

/// Unique tag for identifying operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueTag {
    node_id: NodeId,
    sequence: u64,
}

impl UniqueTag {
    pub fn new(node_id: NodeId, sequence: u64) -> Self {
        UniqueTag { node_id, sequence }
    }
}

/// Element identifier for sequence CRDTs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId {
    node_id: NodeId,
    sequence: u64,
}

impl ElementId {
    pub fn new(node_id: NodeId, sequence: u64) -> Self {
        ElementId { node_id, sequence }
    }
}

/// Operation identifier for sequence operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId {
    node_id: NodeId,
    sequence: u64,
}

impl OperationId {
    pub fn new(node_id: NodeId, sequence: u64) -> Self {
        OperationId { node_id, sequence }
    }
}

/// Result type for CRDT operations
pub type CRDTResult<T> = Result<T, CRDTError>;

/// Errors that can occur during CRDT operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CRDTError {
    InvalidOperation(String),
    MergeConflict(String),
    SerializationError(String),
    NodeNotFound(NodeId),
    InvalidState(String),
}

impl std::fmt::Display for CRDTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CRDTError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            CRDTError::MergeConflict(msg) => write!(f, "Merge conflict: {msg}"),
            CRDTError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            CRDTError::NodeNotFound(node_id) => write!(f, "Node not found: {node_id:?}"),
            CRDTError::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
        }
    }
}

impl std::error::Error for CRDTError {}

/// Grow-only Counter (GCounter) can only increment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GCounter {
    node_id: NodeId,
    counts: HashMap<NodeId, u64>,
    version_vector: VersionVector,
}

impl GCounter {
    /// Create a new GCounter for the given node
    pub fn new(node_id: NodeId) -> Self {
        GCounter {
            node_id,
            counts: HashMap::new(),
            version_vector: VersionVector::new(),
        }
    }
    
    /// Increment the counter by the given amount
    pub fn increment(&mut self, amount: u64) -> CRDTResult<()> {
        if amount == 0 {
            return Err(CRDTError::InvalidOperation("Cannot increment by zero".to_string()));
        }
        
        *self.counts.entry(self.node_id).or_insert(0) += amount;
        self.version_vector.increment(self.node_id);
        Ok(())
    }
    
    /// Get the current value of the counter
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }
    
    /// Merge with another GCounter
    pub fn merge(&mut self, other: &GCounter) -> CRDTResult<()> {
        for (&node_id, &count) in &other.counts {
            let current = self.counts.entry(node_id).or_insert(0);
            *current = (*current).max(count);
        }
        self.version_vector.merge(&other.version_vector);
        Ok(())
    }
    
    /// Compare with another GCounter using version vectors
    pub fn compare(&self, other: &GCounter) -> PartialOrdering {
        self.version_vector.compare(&other.version_vector)
    }
    
    /// Get the count for a specific node
    pub fn get_node_count(&self, node_id: NodeId) -> u64 {
        self.counts.get(&node_id).copied().unwrap_or(0)
    }
    
    /// Get all node counts
    pub fn get_all_counts(&self) -> &HashMap<NodeId, u64> {
        &self.counts
    }
}

/// Increment/Decrement Counter (PNCounter) - can increment and decrement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PNCounter {
    positive: GCounter,
    negative: GCounter,
}

impl PNCounter {
    /// Create a new PNCounter for the given node
    pub fn new(node_id: NodeId) -> Self {
        PNCounter {
            positive: GCounter::new(node_id),
            negative: GCounter::new(node_id),
        }
    }
    
    /// Increment the counter by the given amount
    pub fn increment(&mut self, amount: u64) -> CRDTResult<()> {
        self.positive.increment(amount)
    }
    
    /// Decrement the counter by the given amount
    pub fn decrement(&mut self, amount: u64) -> CRDTResult<()> {
        self.negative.increment(amount)
    }
    
    /// Get the current value of the counter (positive - negative)
    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }
    
    /// Merge with another PNCounter
    pub fn merge(&mut self, other: &PNCounter) -> CRDTResult<()> {
        self.positive.merge(&other.positive)?;
        self.negative.merge(&other.negative)?;
        Ok(())
    }
    
    /// Compare with another PNCounter
    pub fn compare(&self, other: &PNCounter) -> PartialOrdering {
        let pos_cmp = self.positive.compare(&other.positive);
        let neg_cmp = self.negative.compare(&other.negative);
        
        match (pos_cmp, neg_cmp) {
            (PartialOrdering::Equal, PartialOrdering::Equal) => PartialOrdering::Equal,
            (PartialOrdering::Less, PartialOrdering::Less) |
            (PartialOrdering::Less, PartialOrdering::Equal) |
            (PartialOrdering::Equal, PartialOrdering::Less) => PartialOrdering::Less,
            (PartialOrdering::Greater, PartialOrdering::Greater) |
            (PartialOrdering::Greater, PartialOrdering::Equal) |
            (PartialOrdering::Equal, PartialOrdering::Greater) => PartialOrdering::Greater,
            _ => PartialOrdering::Concurrent,
        }
    }
    
    /// Get the positive counter
    pub fn positive_counter(&self) -> &GCounter {
        &self.positive
    }
    
    /// Get the negative counter
    pub fn negative_counter(&self) -> &GCounter {
        &self.negative
    }
}

/// Grow-only Set (GSet) can only add elements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + serde::de::DeserializeOwned")]
pub struct GSet<T> 
where 
    T: Clone + Eq + std::hash::Hash + Serialize + serde::de::DeserializeOwned
{
    elements: HashSet<T>,
    version_vector: VersionVector,
    node_id: NodeId,
}

impl<T> GSet<T> 
where 
    T: Clone + Eq + std::hash::Hash + Serialize + for<'de> Deserialize<'de>
{
    /// Create a new GSet for the given node
    pub fn new(node_id: NodeId) -> Self {
        GSet {
            elements: HashSet::new(),
            version_vector: VersionVector::new(),
            node_id,
        }
    }
    
    /// Add an element to the set
    pub fn add(&mut self, element: T) -> CRDTResult<()> {
        if self.elements.insert(element) {
            self.version_vector.increment(self.node_id);
        }
        Ok(())
    }
    
    /// Check if the set contains an element
    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains(element)
    }
    
    /// Get all elements in the set
    pub fn elements(&self) -> &HashSet<T> {
        &self.elements
    }
    
    /// Get the size of the set
    pub fn size(&self) -> usize {
        self.elements.len()
    }
    
    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    
    /// Merge with another GSet
    pub fn merge(&mut self, other: &GSet<T>) -> CRDTResult<()> {
        let old_size = self.elements.len();
        self.elements.extend(other.elements.iter().cloned());
        
        if self.elements.len() > old_size {
            self.version_vector.merge(&other.version_vector);
        }
        Ok(())
    }
    
    /// Compare with another GSet
    pub fn compare(&self, other: &GSet<T>) -> PartialOrdering {
        self.version_vector.compare(&other.version_vector)
    }
    
    /// Create an iterator over the elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }
}

/// Two-Phase Set can add and remove elements, but removed elements cannot be re-added
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + serde::de::DeserializeOwned")]
pub struct TwoPhaseSet<T> 
where 
    T: Clone + Eq + std::hash::Hash + Serialize + serde::de::DeserializeOwned
{
    added: GSet<T>,
    removed: GSet<T>,
}

impl<T> TwoPhaseSet<T> 
where 
    T: Clone + Eq + std::hash::Hash + Serialize + for<'de> Deserialize<'de>
{
    /// Create a new TwoPhaseSet for the given node
    pub fn new(node_id: NodeId) -> Self {
        TwoPhaseSet {
            added: GSet::new(node_id),
            removed: GSet::new(node_id),
        }
    }
    
    /// Add an element to the set
    pub fn add(&mut self, element: T) -> CRDTResult<()> {
        if self.removed.contains(&element) {
            return Err(CRDTError::InvalidOperation(
                "Cannot add element that has been removed".to_string()
            ));
        }
        self.added.add(element)
    }
    
    /// Remove an element from the set
    pub fn remove(&mut self, element: &T) -> CRDTResult<()> {
        if !self.added.contains(element) {
            return Err(CRDTError::InvalidOperation(
                "Cannot remove element that was never added".to_string()
            ));
        }
        self.removed.add(element.clone())
    }
    
    /// Check if the set contains an element
    pub fn contains(&self, element: &T) -> bool {
        self.added.contains(element) && !self.removed.contains(element)
    }
    
    /// Get all elements currently in the set
    pub fn elements(&self) -> HashSet<T> {
        self.added.elements()
            .iter()
            .filter(|element| !self.removed.contains(element))
            .cloned()
            .collect()
    }
    
    /// Get the size of the set
    pub fn size(&self) -> usize {
        self.elements().len()
    }
    
    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
    
    /// Merge with another TwoPhaseSet
    pub fn merge(&mut self, other: &TwoPhaseSet<T>) -> CRDTResult<()> {
        self.added.merge(&other.added)?;
        self.removed.merge(&other.removed)?;
        Ok(())
    }
    
    /// Compare with another TwoPhaseSet
    pub fn compare(&self, other: &TwoPhaseSet<T>) -> PartialOrdering {
        let added_cmp = self.added.compare(&other.added);
        let removed_cmp = self.removed.compare(&other.removed);
        
        match (added_cmp, removed_cmp) {
            (PartialOrdering::Equal, PartialOrdering::Equal) => PartialOrdering::Equal,
            (PartialOrdering::Less, PartialOrdering::Less) |
            (PartialOrdering::Less, PartialOrdering::Equal) |
            (PartialOrdering::Equal, PartialOrdering::Less) => PartialOrdering::Less,
            (PartialOrdering::Greater, PartialOrdering::Greater) |
            (PartialOrdering::Greater, PartialOrdering::Equal) |
            (PartialOrdering::Equal, PartialOrdering::Greater) => PartialOrdering::Greater,
            _ => PartialOrdering::Concurrent,
        }
    }
    
    /// Get the added set
    pub fn added_set(&self) -> &GSet<T> {
        &self.added
    }
    
    /// Get the removed set
    pub fn removed_set(&self) -> &GSet<T> {
        &self.removed
    }
}

/// Observed-Remove Set (ORSet) can add and remove elements with unique tags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + serde::de::DeserializeOwned")]
pub struct ORSet<T> 
where 
    T: Clone + Eq + std::hash::Hash + Serialize + serde::de::DeserializeOwned
{
    elements: HashMap<T, HashSet<UniqueTag>>,
    removed_tags: HashSet<UniqueTag>,
    node_id: NodeId,
    next_sequence: u64,
    version_vector: VersionVector,
}

impl<T> ORSet<T> 
where 
    T: Clone + Eq + std::hash::Hash + Serialize + for<'de> Deserialize<'de>
{
    /// Create a new ORSet for the given node
    pub fn new(node_id: NodeId) -> Self {
        ORSet {
            elements: HashMap::new(),
            removed_tags: HashSet::new(),
            node_id,
            next_sequence: 0,
            version_vector: VersionVector::new(),
        }
    }
    
    /// Add an element to the set, returning the unique tag
    pub fn add(&mut self, element: T) -> CRDTResult<UniqueTag> {
        let tag = UniqueTag::new(self.node_id, self.next_sequence);
        self.next_sequence += 1;
        
        self.elements.entry(element).or_default().insert(tag);
        self.version_vector.increment(self.node_id);
        
        Ok(tag)
    }
    
    /// Remove an element from the set by removing all its tags
    pub fn remove(&mut self, element: &T) -> CRDTResult<()> {
        if let Some(tags) = self.elements.get(element) {
            for &tag in tags {
                self.removed_tags.insert(tag);
            }
            self.version_vector.increment(self.node_id);
            Ok(())
        } else {
            Err(CRDTError::InvalidOperation(
                "Cannot remove element that is not in the set".to_string()
            ))
        }
    }
    
    /// Remove an element by its specific tag
    pub fn remove_tag(&mut self, tag: UniqueTag) -> CRDTResult<()> {
        self.removed_tags.insert(tag);
        self.version_vector.increment(self.node_id);
        Ok(())
    }
    
    /// Check if the set contains an element
    pub fn contains(&self, element: &T) -> bool {
        if let Some(tags) = self.elements.get(element) {
            tags.iter().any(|tag| !self.removed_tags.contains(tag))
        } else {
            false
        }
    }
    
    /// Get all elements currently in the set
    pub fn elements(&self) -> HashSet<T> {
        self.elements
            .iter()
            .filter_map(|(element, tags)| {
                if tags.iter().any(|tag| !self.removed_tags.contains(tag)) {
                    Some(element.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get the size of the set
    pub fn size(&self) -> usize {
        self.elements()
            .len()
    }
    
    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
    
    /// Merge with another ORSet
    pub fn merge(&mut self, other: &ORSet<T>) -> CRDTResult<()> {
        // Merge elements and their tags
        for (element, tags) in &other.elements {
            let entry = self.elements.entry(element.clone()).or_default();
            entry.extend(tags.iter().copied());
        }
        
        // Merge removed tags
        self.removed_tags.extend(other.removed_tags.iter().copied());
        
        // Update sequence number to avoid conflicts
        self.next_sequence = self.next_sequence.max(other.next_sequence);
        
        // Merge version vectors
        self.version_vector.merge(&other.version_vector);
        
        Ok(())
    }
    
    /// Compare with another ORSet
    pub fn compare(&self, other: &ORSet<T>) -> PartialOrdering {
        self.version_vector.compare(&other.version_vector)
    }
    
    /// Get all tags for an element
    pub fn get_tags(&self, element: &T) -> Option<&HashSet<UniqueTag>> {
        self.elements.get(element)
    }
    
    /// Get all removed tags
    pub fn removed_tags(&self) -> &HashSet<UniqueTag> {
        &self.removed_tags
    }
    
    /// Get the number of active tags for an element
    pub fn active_tag_count(&self, element: &T) -> usize {
        if let Some(tags) = self.elements.get(element) {
            tags.iter().filter(|tag| !self.removed_tags.contains(tag)).count()
        } else {
            0
        }
    }
}

/// Last-Writer-Wins Register (LWWRegister) stores a single value with timestamp
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + serde::de::DeserializeOwned")]
pub struct LWWRegister<T> 
where 
    T: Clone + Eq + Serialize + serde::de::DeserializeOwned
{
    value: T,
    timestamp: LogicalTimestamp,
    node_id: NodeId,
    version_vector: VersionVector,
}

impl<T> LWWRegister<T> 
where 
    T: Clone + Eq + Serialize + for<'de> Deserialize<'de>
{
    /// Create a new LWWRegister with an initial value
    pub fn new(node_id: NodeId, initial_value: T) -> Self {
        let mut version_vector = VersionVector::new();
        version_vector.increment(node_id);
        
        LWWRegister {
            value: initial_value,
            timestamp: LogicalTimestamp::new(0),
            node_id,
            version_vector,
        }
    }
    
    /// Set the value with a specific timestamp
    pub fn set(&mut self, value: T, timestamp: LogicalTimestamp) -> CRDTResult<()> {
        if timestamp > self.timestamp || 
           (timestamp == self.timestamp && self.node_id < NodeId(u64::MAX)) {
            self.value = value;
            self.timestamp = timestamp;
            self.version_vector.increment(self.node_id);
        }
        Ok(())
    }
    
    /// Set the value with the current logical time
    pub fn set_with_current_time(&mut self, value: T) -> CRDTResult<()> {
        let mut new_timestamp = self.timestamp;
        new_timestamp.increment();
        self.set(value, new_timestamp)
    }
    
    /// Get the current value
    pub fn get(&self) -> &T {
        &self.value
    }
    
    /// Get the timestamp of the current value
    pub fn get_timestamp(&self) -> LogicalTimestamp {
        self.timestamp
    }
    
    /// Get the node ID that last wrote the value
    pub fn get_writer_node(&self) -> NodeId {
        self.node_id
    }
    
    /// Merge with another LWWRegister
    pub fn merge(&mut self, other: &LWWRegister<T>) -> CRDTResult<()> {
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Greater => {
                // Other has newer timestamp, use its value
                self.value = other.value.clone();
                self.timestamp = other.timestamp;
                self.node_id = other.node_id;
            }
            Ordering::Equal => {
                // Same timestamp, use node ID as tiebreaker (deterministic)
                if other.node_id > self.node_id {
                    self.value = other.value.clone();
                    self.node_id = other.node_id;
                }
            }
            Ordering::Less => {
                // Our timestamp is newer, keep our value
            }
        }
        
        self.version_vector.merge(&other.version_vector);
        Ok(())
    }
    
    /// Compare with another LWWRegister
    pub fn compare(&self, other: &LWWRegister<T>) -> PartialOrdering {
        self.version_vector.compare(&other.version_vector)
    }
    
    /// Check if this register has the same value as another
    pub fn has_same_value(&self, other: &LWWRegister<T>) -> bool {
        self.value == other.value
    }
}

/// Multi-Value Register (MVRegister) stores multiple concurrent values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + serde::de::DeserializeOwned")]
pub struct MVRegister<T> 
where 
    T: Clone + Eq + Serialize + serde::de::DeserializeOwned
{
    values: HashMap<NodeId, (T, LogicalTimestamp)>,
    version_vector: VersionVector,
    node_id: NodeId,
    current_timestamp: LogicalTimestamp,
}

impl<T> MVRegister<T> 
where 
    T: Clone + Eq + Serialize + for<'de> Deserialize<'de>
{
    /// Create a new MVRegister with an initial value
    pub fn new(node_id: NodeId, initial_value: T) -> Self {
        let mut values = HashMap::new();
        let mut version_vector = VersionVector::new();
        let timestamp = LogicalTimestamp::new(0);
        
        values.insert(node_id, (initial_value, timestamp));
        version_vector.increment(node_id);
        
        MVRegister {
            values,
            version_vector,
            node_id,
            current_timestamp: timestamp,
        }
    }
    
    /// Set a new value for this node
    pub fn set(&mut self, value: T) -> CRDTResult<()> {
        self.current_timestamp.increment();
        self.values.insert(self.node_id, (value, self.current_timestamp));
        self.version_vector.increment(self.node_id);
        Ok(())
    }
    
    /// Get all current values (including concurrent ones)
    pub fn get_values(&self) -> Vec<&T> {
        // Find the maximum timestamp
        let max_timestamp = self.values
            .values()
            .map(|(_, timestamp)| *timestamp)
            .max()
            .unwrap_or(LogicalTimestamp::new(0));
        
        // Return all values with the maximum timestamp
        self.values
            .values()
            .filter(|(_, timestamp)| *timestamp == max_timestamp)
            .map(|(value, _)| value)
            .collect()
    }
    
    /// Get all concurrent values (values that are not dominated by others)
    pub fn get_concurrent_values(&self) -> Vec<&T> {
        let mut concurrent_values = Vec::new();
        
        for (node_id, (value, timestamp)) in &self.values {
            let mut is_concurrent = true;
            
            // Check if this value is dominated by any other value
            for (other_node_id, (_, other_timestamp)) in &self.values {
                if node_id != other_node_id && other_timestamp > timestamp {
                    is_concurrent = false;
                    break;
                }
            }
            
            if is_concurrent {
                concurrent_values.push(value);
            }
        }
        
        concurrent_values
    }
    
    /// Get the value for a specific node
    pub fn get_node_value(&self, node_id: NodeId) -> Option<&T> {
        self.values.get(&node_id).map(|(value, _)| value)
    }
    
    /// Get the timestamp for a specific node's value
    pub fn get_node_timestamp(&self, node_id: NodeId) -> Option<LogicalTimestamp> {
        self.values.get(&node_id).map(|(_, timestamp)| *timestamp)
    }
    
    /// Check if there are concurrent values
    pub fn has_conflicts(&self) -> bool {
        self.get_concurrent_values().len() > 1
    }
    
    /// Merge with another MVRegister
    pub fn merge(&mut self, other: &MVRegister<T>) -> CRDTResult<()> {
        for (&node_id, (value, timestamp)) in &other.values {
            match self.values.get(&node_id) {
                Some((_, our_timestamp)) => {
                    // Keep the value with the later timestamp
                    if timestamp > our_timestamp {
                        self.values.insert(node_id, (value.clone(), *timestamp));
                    }
                }
                None => {
                    // We don't have a value from this node, add it
                    self.values.insert(node_id, (value.clone(), *timestamp));
                }
            }
        }
        
        // Update our current timestamp to be at least as high as the other's
        self.current_timestamp = self.current_timestamp.max(other.current_timestamp);
        
        // Merge version vectors
        self.version_vector.merge(&other.version_vector);
        
        Ok(())
    }
    
    /// Compare with another MVRegister
    pub fn compare(&self, other: &MVRegister<T>) -> PartialOrdering {
        self.version_vector.compare(&other.version_vector)
    }
    
    /// Resolve conflicts using a resolver function
    pub fn resolve_conflicts<F>(&mut self, resolver: F) -> CRDTResult<()>
    where 
        F: Fn(&[T]) -> T
    {
        let concurrent_values: Vec<T> = self.get_concurrent_values()
            .into_iter()
            .cloned()
            .collect();
        
        if concurrent_values.len() > 1 {
            let resolved_value = resolver(&concurrent_values);
            self.set(resolved_value)?;
        }
        
        Ok(())
    }
    
    /// Get the number of nodes that have written values
    pub fn node_count(&self) -> usize {
        self.values.len()
    }
    
    /// Get all node IDs that have written values
    pub fn get_writer_nodes(&self) -> Vec<NodeId> {
        self.values.keys().copied().collect()
    }
    
    /// Clear old values, keeping only the most recent ones
    pub fn compact(&mut self) {
        let max_timestamp = self.values
            .values()
            .map(|(_, timestamp)| *timestamp)
            .max()
            .unwrap_or(LogicalTimestamp::new(0));
        
        self.values.retain(|_, (_, timestamp)| *timestamp == max_timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gcounter() {
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);
        
        let mut counter1 = GCounter::new(node1);
        let mut counter2 = GCounter::new(node2);
        
        counter1.increment(5).unwrap();
        counter2.increment(3).unwrap();
        
        assert_eq!(counter1.value(), 5);
        assert_eq!(counter2.value(), 3);
        
        counter1.merge(&counter2).unwrap();
        assert_eq!(counter1.value(), 8);
    }
    
    #[test]
    fn test_gset() {
        let node = NodeId::new(1);
        let mut set1 = GSet::new(node);
        let mut set2 = GSet::new(node);
        
        set1.add("a".to_string()).unwrap();
        set1.add("b".to_string()).unwrap();
        set2.add("b".to_string()).unwrap();
        set2.add("c".to_string()).unwrap();
        
        assert_eq!(set1.size(), 2);
        assert!(set1.contains(&"a".to_string()));
        assert!(set1.contains(&"b".to_string()));
        
        set1.merge(&set2).unwrap();
        assert_eq!(set1.size(), 3);
        assert!(set1.contains(&"c".to_string()));
    }
    
    #[test]
    fn test_orset() {
        let node = NodeId::new(1);
        let mut set1 = ORSet::new(node);
        
        let _tag_a = set1.add("a".to_string()).unwrap();
        let tag_b = set1.add("b".to_string()).unwrap();
        
        assert_eq!(set1.elements().len(), 2);
        assert!(set1.contains(&"a".to_string()));
        assert!(set1.contains(&"b".to_string()));
        
        // Remove "b" by tag
        set1.remove_tag(tag_b).unwrap();
        assert_eq!(set1.elements().len(), 1);
        assert!(set1.contains(&"a".to_string()));
        assert!(!set1.contains(&"b".to_string()));
    }
}
