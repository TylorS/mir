//! Distributed runtime primitives

use std::collections::HashMap;

/// Node identifier in a distributed system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }
}

/// Distributed runtime for coordinating across multiple nodes
pub struct DistributedRuntime {
    node_id: NodeId,
    connected_nodes: HashMap<NodeId, NodeInfo>,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: NodeId,
    pub address: String,
    pub status: NodeStatus,
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Connected,
    Disconnected,
    Updating,
}

impl DistributedRuntime {
    pub fn new(node_id: NodeId) -> Self {
        DistributedRuntime {
            node_id,
            connected_nodes: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node_info: NodeInfo) {
        self.connected_nodes.insert(node_info.id, node_info);
    }
    
    pub fn get_connected_nodes(&self) -> Vec<&NodeInfo> {
        self.connected_nodes.values().collect()
    }
}