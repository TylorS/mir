//! Content-addressable storage implementation

use mir_types::ContentHash;
use std::collections::{HashMap, HashSet};

/// Content-addressable storage trait
pub trait ContentAddressableStore {
    fn store(&mut self, content: &[u8]) -> ContentHash;
    fn retrieve(&self, hash: ContentHash) -> Option<Vec<u8>>;
    fn exists(&self, hash: ContentHash) -> bool;
    fn garbage_collect(&mut self, reachable: &HashSet<ContentHash>);
}

/// In-memory implementation of content-addressable storage
pub struct InMemoryStore {
    storage: HashMap<ContentHash, Vec<u8>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        InMemoryStore {
            storage: HashMap::new(),
        }
    }
}

impl ContentAddressableStore for InMemoryStore {
    fn store(&mut self, content: &[u8]) -> ContentHash {
        let hash = ContentHash::new(content);
        self.storage.insert(hash, content.to_vec());
        hash
    }
    
    fn retrieve(&self, hash: ContentHash) -> Option<Vec<u8>> {
        self.storage.get(&hash).cloned()
    }
    
    fn exists(&self, hash: ContentHash) -> bool {
        self.storage.contains_key(&hash)
    }
    
    fn garbage_collect(&mut self, reachable: &HashSet<ContentHash>) {
        self.storage.retain(|hash, _| reachable.contains(hash));
    }
}