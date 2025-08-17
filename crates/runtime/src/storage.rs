//! Content-addressable storage implementation

use mir_types::ContentHash;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Content-addressable storage trait
pub trait ContentAddressableStore {
    /// Store content and return its hash
    fn store(&mut self, content: &[u8]) -> ContentHash;
    
    /// Retrieve content by hash
    fn retrieve(&self, hash: ContentHash) -> Option<Vec<u8>>;
    
    /// Check if content exists
    fn exists(&self, hash: ContentHash) -> bool;
    
    /// Remove unreachable content based on reachable set
    fn garbage_collect(&mut self, reachable: &HashSet<ContentHash>);
    
    /// Get storage statistics
    fn stats(&self) -> StorageStats;
    
    /// Get all stored hashes
    fn list_hashes(&self) -> Vec<ContentHash>;
    
    /// Get content size
    fn content_size(&self, hash: ContentHash) -> Option<usize>;
    
    /// Bulk store multiple contents
    fn store_batch(&mut self, contents: &[&[u8]]) -> Vec<ContentHash>;
    
    /// Bulk retrieve multiple contents
    fn retrieve_batch(&self, hashes: &[ContentHash]) -> Vec<Option<Vec<u8>>>;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_items: usize,
    pub total_bytes: usize,
    pub unique_hashes: usize,
    pub last_gc_timestamp: Option<u64>,
    pub gc_runs: u64,
    pub items_collected: u64,
}

/// Metadata for stored content
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentMetadata {
    size: usize,
    stored_at: u64,
    access_count: u64,
    last_accessed: u64,
}

/// In-memory implementation of content-addressable storage
#[derive(Clone)]
pub struct InMemoryStore {
    storage: HashMap<ContentHash, Vec<u8>>,
    metadata: HashMap<ContentHash, ContentMetadata>,
    stats: StorageStats,
}

impl InMemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        InMemoryStore {
            storage: HashMap::new(),
            metadata: HashMap::new(),
            stats: StorageStats {
                total_items: 0,
                total_bytes: 0,
                unique_hashes: 0,
                last_gc_timestamp: None,
                gc_runs: 0,
                items_collected: 0,
            },
        }
    }
    
    /// Create a new store with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        InMemoryStore {
            storage: HashMap::with_capacity(capacity),
            metadata: HashMap::with_capacity(capacity),
            stats: StorageStats {
                total_items: 0,
                total_bytes: 0,
                unique_hashes: 0,
                last_gc_timestamp: None,
                gc_runs: 0,
                items_collected: 0,
            },
        }
    }
    
    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Update access metadata
    #[allow(dead_code)]
    fn update_access(&mut self, hash: ContentHash) {
        if let Some(metadata) = self.metadata.get_mut(&hash) {
            metadata.access_count += 1;
            metadata.last_accessed = Self::current_timestamp();
        }
    }
    
    /// Update storage statistics
    fn update_stats(&mut self) {
        self.stats.total_items = self.storage.len();
        self.stats.total_bytes = self.storage.values().map(|v| v.len()).sum();
        self.stats.unique_hashes = self.storage.len();
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentAddressableStore for InMemoryStore {
    fn store(&mut self, content: &[u8]) -> ContentHash {
        let hash = ContentHash::new(content);
        
        // Only store if not already present (deduplication)
        if let std::collections::hash_map::Entry::Vacant(e) = self.storage.entry(hash) {
            let now = Self::current_timestamp();
            
            e.insert(content.to_vec());
            self.metadata.insert(hash, ContentMetadata {
                size: content.len(),
                stored_at: now,
                access_count: 0,
                last_accessed: now,
            });
            
            self.update_stats();
        }
        
        hash
    }
    
    fn retrieve(&self, hash: ContentHash) -> Option<Vec<u8>> {
        let content = self.storage.get(&hash).cloned();
        
        // Note: We can't update access metadata here because retrieve takes &self
        // In a real implementation, we might use interior mutability or separate tracking
        
        content
    }
    
    fn exists(&self, hash: ContentHash) -> bool {
        self.storage.contains_key(&hash)
    }
    
    fn garbage_collect(&mut self, reachable: &HashSet<ContentHash>) {
        let initial_count = self.storage.len();
        
        // Remove unreachable content
        self.storage.retain(|hash, _| reachable.contains(hash));
        self.metadata.retain(|hash, _| reachable.contains(hash));
        
        let collected = initial_count - self.storage.len();
        
        // Update statistics
        self.stats.last_gc_timestamp = Some(Self::current_timestamp());
        self.stats.gc_runs += 1;
        self.stats.items_collected += collected as u64;
        self.update_stats();
    }
    
    fn stats(&self) -> StorageStats {
        self.stats.clone()
    }
    
    fn list_hashes(&self) -> Vec<ContentHash> {
        self.storage.keys().copied().collect()
    }
    
    fn content_size(&self, hash: ContentHash) -> Option<usize> {
        self.metadata.get(&hash).map(|m| m.size)
    }
    
    fn store_batch(&mut self, contents: &[&[u8]]) -> Vec<ContentHash> {
        contents.iter().map(|content| self.store(content)).collect()
    }
    
    fn retrieve_batch(&self, hashes: &[ContentHash]) -> Vec<Option<Vec<u8>>> {
        hashes.iter().map(|hash| self.retrieve(*hash)).collect()
    }
}

/// Reference counting store for automatic garbage collection
pub struct RefCountingStore {
    inner: InMemoryStore,
    ref_counts: HashMap<ContentHash, usize>,
}

impl RefCountingStore {
    /// Create a new reference counting store
    pub fn new() -> Self {
        RefCountingStore {
            inner: InMemoryStore::new(),
            ref_counts: HashMap::new(),
        }
    }
    
    /// Increment reference count for a hash
    pub fn add_ref(&mut self, hash: ContentHash) {
        *self.ref_counts.entry(hash).or_insert(0) += 1;
    }
    
    /// Decrement reference count for a hash
    pub fn remove_ref(&mut self, hash: ContentHash) {
        if let Some(count) = self.ref_counts.get_mut(&hash) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.ref_counts.remove(&hash);
            }
        }
    }
    
    /// Get reference count for a hash
    pub fn ref_count(&self, hash: ContentHash) -> usize {
        self.ref_counts.get(&hash).copied().unwrap_or(0)
    }
    
    /// Perform automatic garbage collection based on reference counts
    pub fn auto_gc(&mut self) {
        let reachable: HashSet<ContentHash> = self.ref_counts
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|(&hash, _)| hash)
            .collect();
        
        self.inner.garbage_collect(&reachable);
    }
}

impl Default for RefCountingStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentAddressableStore for RefCountingStore {
    fn store(&mut self, content: &[u8]) -> ContentHash {
        let hash = self.inner.store(content);
        self.add_ref(hash);
        hash
    }
    
    fn retrieve(&self, hash: ContentHash) -> Option<Vec<u8>> {
        self.inner.retrieve(hash)
    }
    
    fn exists(&self, hash: ContentHash) -> bool {
        self.inner.exists(hash)
    }
    
    fn garbage_collect(&mut self, reachable: &HashSet<ContentHash>) {
        self.inner.garbage_collect(reachable);
        // Also clean up reference counts for non-existent items
        self.ref_counts.retain(|hash, _| reachable.contains(hash));
    }
    
    fn stats(&self) -> StorageStats {
        self.inner.stats()
    }
    
    fn list_hashes(&self) -> Vec<ContentHash> {
        self.inner.list_hashes()
    }
    
    fn content_size(&self, hash: ContentHash) -> Option<usize> {
        self.inner.content_size(hash)
    }
    
    fn store_batch(&mut self, contents: &[&[u8]]) -> Vec<ContentHash> {
        let hashes = self.inner.store_batch(contents);
        for &hash in &hashes {
            self.add_ref(hash);
        }
        hashes
    }
    
    fn retrieve_batch(&self, hashes: &[ContentHash]) -> Vec<Option<Vec<u8>>> {
        self.inner.retrieve_batch(hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_storage_operations() {
        let mut store = InMemoryStore::new();
        
        let content1 = b"hello world";
        let content2 = b"goodbye world";
        
        let hash1 = store.store(content1);
        let hash2 = store.store(content2);
        
        assert_ne!(hash1, hash2);
        assert!(store.exists(hash1));
        assert!(store.exists(hash2));
        
        assert_eq!(store.retrieve(hash1), Some(content1.to_vec()));
        assert_eq!(store.retrieve(hash2), Some(content2.to_vec()));
    }
    
    #[test]
    fn test_deduplication() {
        let mut store = InMemoryStore::new();
        
        let content = b"duplicate content";
        let hash1 = store.store(content);
        let hash2 = store.store(content);
        
        assert_eq!(hash1, hash2);
        assert_eq!(store.stats().total_items, 1);
    }
    
    #[test]
    fn test_garbage_collection() {
        let mut store = InMemoryStore::new();
        
        let content1 = b"keep this";
        let content2 = b"remove this";
        
        let hash1 = store.store(content1);
        let hash2 = store.store(content2);
        
        assert_eq!(store.stats().total_items, 2);
        
        // Only hash1 is reachable
        let mut reachable = HashSet::new();
        reachable.insert(hash1);
        
        store.garbage_collect(&reachable);
        
        assert_eq!(store.stats().total_items, 1);
        assert!(store.exists(hash1));
        assert!(!store.exists(hash2));
    }
    
    #[test]
    fn test_batch_operations() {
        let mut store = InMemoryStore::new();
        
        let contents = vec![b"content1".as_slice(), b"content2".as_slice(), b"content3".as_slice()];
        let hashes = store.store_batch(&contents);
        
        assert_eq!(hashes.len(), 3);
        assert_eq!(store.stats().total_items, 3);
        
        let retrieved = store.retrieve_batch(&hashes);
        assert_eq!(retrieved.len(), 3);
        assert_eq!(retrieved[0], Some(b"content1".to_vec()));
        assert_eq!(retrieved[1], Some(b"content2".to_vec()));
        assert_eq!(retrieved[2], Some(b"content3".to_vec()));
    }
    
    #[test]
    fn test_ref_counting_store() {
        let mut store = RefCountingStore::new();
        
        let content = b"ref counted content";
        let hash = store.store(content);
        
        assert_eq!(store.ref_count(hash), 1);
        
        store.add_ref(hash);
        assert_eq!(store.ref_count(hash), 2);
        
        store.remove_ref(hash);
        assert_eq!(store.ref_count(hash), 1);
        
        store.remove_ref(hash);
        assert_eq!(store.ref_count(hash), 0);
        
        // Auto GC should remove the content
        store.auto_gc();
        assert!(!store.exists(hash));
    }
}