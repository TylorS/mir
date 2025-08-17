//! Distributed runtime primitives

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::crdt::NodeId;

/// Distributed runtime for coordinating across multiple nodes
pub struct DistributedRuntime {
    node_id: NodeId,
    connected_nodes: HashMap<NodeId, NodeInfo>,
    event_loops: HashMap<String, Box<dyn EventLoop>>,
    schedulers: HashMap<String, Box<dyn Scheduler>>,
    queues: HashMap<String, Box<dyn Queue>>,
    pub_sub_channels: HashMap<String, Box<dyn PubSubChannel>>,
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
            event_loops: HashMap::new(),
            schedulers: HashMap::new(),
            queues: HashMap::new(),
            pub_sub_channels: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node_info: NodeInfo) {
        self.connected_nodes.insert(node_info.id, node_info);
    }
    
    pub fn get_connected_nodes(&self) -> Vec<&NodeInfo> {
        self.connected_nodes.values().collect()
    }
    
    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }
    
    // Event Loop management
    pub fn create_event_loop(&mut self, name: String) -> DistributedResult<()> {
        let event_loop = Box::new(BasicEventLoop::new());
        self.event_loops.insert(name, event_loop);
        Ok(())
    }
    
    pub fn get_event_loop_mut(&mut self, name: &str) -> Option<&mut Box<dyn EventLoop>> {
        self.event_loops.get_mut(name)
    }
    
    // Scheduler management
    pub fn create_scheduler(&mut self, name: String) -> DistributedResult<()> {
        let available_nodes: Vec<NodeId> = self.connected_nodes.keys().copied().collect();
        let scheduler = Box::new(BasicScheduler::new(available_nodes));
        self.schedulers.insert(name, scheduler);
        Ok(())
    }
    
    pub fn get_scheduler_mut(&mut self, name: &str) -> Option<&mut Box<dyn Scheduler>> {
        self.schedulers.get_mut(name)
    }
    
    // Queue management
    pub fn create_queue(&mut self, name: String, ordering: OrderingGuarantee) -> DistributedResult<()> {
        let queue = Box::new(BasicQueue::new(self.node_id, ordering));
        self.queues.insert(name, queue);
        Ok(())
    }
    
    pub fn get_queue_mut(&mut self, name: &str) -> Option<&mut Box<dyn Queue>> {
        self.queues.get_mut(name)
    }
    
    // PubSub management
    pub fn create_pub_sub_channel(&mut self, name: String) -> DistributedResult<()> {
        let channel = Box::new(BasicPubSubChannel::new(self.node_id));
        self.pub_sub_channels.insert(name, channel);
        Ok(())
    }
    
    pub fn get_pub_sub_channel_mut(&mut self, name: &str) -> Option<&mut Box<dyn PubSubChannel>> {
        self.pub_sub_channels.get_mut(name)
    }
    
    // Utility methods
    pub fn list_event_loops(&self) -> Vec<String> {
        self.event_loops.keys().cloned().collect()
    }
    
    pub fn list_schedulers(&self) -> Vec<String> {
        self.schedulers.keys().cloned().collect()
    }
    
    pub fn list_queues(&self) -> Vec<String> {
        self.queues.keys().cloned().collect()
    }
    
    pub fn list_pub_sub_channels(&self) -> Vec<String> {
        self.pub_sub_channels.keys().cloned().collect()
    }
}

/// Task identifier for scheduled tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

impl TaskId {
    pub fn new(id: u64) -> Self {
        TaskId(id)
    }
}

/// Function identifier for distributed execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionId(pub u64);

impl FunctionId {
    pub fn new(id: u64) -> Self {
        FunctionId(id)
    }
}

/// Schedule identifier for distributed scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduleId(pub u64);

impl ScheduleId {
    pub fn new(id: u64) -> Self {
        ScheduleId(id)
    }
}

/// Message identifier for pub-sub
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub u64);

impl MessageId {
    pub fn new(id: u64) -> Self {
        MessageId(id)
    }
}

/// Subscription identifier for pub-sub
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub u64);

impl SubscriptionId {
    pub fn new(id: u64) -> Self {
        SubscriptionId(id)
    }
}

/// Result type for distributed operations
pub type DistributedResult<T> = Result<T, DistributedError>;

/// Errors that can occur in distributed operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedError {
    NodeNotFound(NodeId),
    TaskNotFound(TaskId),
    ScheduleNotFound(ScheduleId),
    MessageNotFound(MessageId),
    SubscriptionNotFound(SubscriptionId),
    NetworkError(String),
    TimeoutError(String),
    SerializationError(String),
    InvalidOperation(String),
    ResourceExhausted(String),
}

impl std::fmt::Display for DistributedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributedError::NodeNotFound(node_id) => write!(f, "Node not found: {node_id:?}"),
            DistributedError::TaskNotFound(task_id) => write!(f, "Task not found: {task_id:?}"),
            DistributedError::ScheduleNotFound(schedule_id) => write!(f, "Schedule not found: {schedule_id:?}"),
            DistributedError::MessageNotFound(message_id) => write!(f, "Message not found: {message_id:?}"),
            DistributedError::SubscriptionNotFound(subscription_id) => write!(f, "Subscription not found: {subscription_id:?}"),
            DistributedError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            DistributedError::TimeoutError(msg) => write!(f, "Timeout error: {msg}"),
            DistributedError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            DistributedError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            DistributedError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {msg}"),
        }
    }
}

impl std::error::Error for DistributedError {}

/// Information about a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: TaskId,
    pub function_id: FunctionId,
    pub scheduled_time: SystemTime,
    pub delay: Duration,
    pub is_recurring: bool,
    pub interval: Option<Duration>,
    pub status: TaskStatus,
}

/// Status of a scheduled task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// Event loop for distributed task scheduling
pub trait EventLoop {
    /// Schedule a task to run after a delay
    fn schedule(&mut self, task: FunctionId, delay: Duration) -> DistributedResult<TaskId>;
    
    /// Schedule a recurring task
    fn schedule_recurring(&mut self, task: FunctionId, interval: Duration) -> DistributedResult<TaskId>;
    
    /// Cancel a scheduled task
    fn cancel_task(&mut self, task_id: TaskId) -> DistributedResult<()>;
    
    /// Get information about pending tasks
    fn get_pending_tasks(&self) -> Vec<TaskInfo>;
    
    /// Get information about a specific task
    fn get_task_info(&self, task_id: TaskId) -> Option<TaskInfo>;
    
    /// Process pending tasks (should be called regularly)
    fn process_tasks(&mut self) -> DistributedResult<Vec<TaskId>>;
}

/// Basic implementation of EventLoop
#[derive(Debug)]
pub struct BasicEventLoop {
    tasks: HashMap<TaskId, TaskInfo>,
    next_task_id: u64,
    start_time: SystemTime,
}

impl Default for BasicEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicEventLoop {
    pub fn new() -> Self {
        BasicEventLoop {
            tasks: HashMap::new(),
            next_task_id: 0,
            start_time: SystemTime::now(),
        }
    }
    
    fn next_task_id(&mut self) -> TaskId {
        let id = TaskId::new(self.next_task_id);
        self.next_task_id += 1;
        id
    }
}

impl EventLoop for BasicEventLoop {
    fn schedule(&mut self, function_id: FunctionId, delay: Duration) -> DistributedResult<TaskId> {
        let task_id = self.next_task_id();
        let scheduled_time = self.start_time.checked_add(delay).unwrap_or(SystemTime::now());
        
        let task_info = TaskInfo {
            id: task_id,
            function_id,
            scheduled_time,
            delay,
            is_recurring: false,
            interval: None,
            status: TaskStatus::Pending,
        };
        
        self.tasks.insert(task_id, task_info);
        Ok(task_id)
    }
    
    fn schedule_recurring(&mut self, function_id: FunctionId, interval: Duration) -> DistributedResult<TaskId> {
        let task_id = self.next_task_id();
        let scheduled_time = self.start_time.checked_add(interval).unwrap_or(SystemTime::now());
        
        let task_info = TaskInfo {
            id: task_id,
            function_id,
            scheduled_time,
            delay: interval,
            is_recurring: true,
            interval: Some(interval),
            status: TaskStatus::Pending,
        };
        
        self.tasks.insert(task_id, task_info);
        Ok(task_id)
    }
    
    fn cancel_task(&mut self, task_id: TaskId) -> DistributedResult<()> {
        if let Some(task_info) = self.tasks.get_mut(&task_id) {
            task_info.status = TaskStatus::Cancelled;
            Ok(())
        } else {
            Err(DistributedError::TaskNotFound(task_id))
        }
    }
    
    fn get_pending_tasks(&self) -> Vec<TaskInfo> {
        self.tasks
            .values()
            .filter(|task| task.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }
    
    fn get_task_info(&self, task_id: TaskId) -> Option<TaskInfo> {
        self.tasks.get(&task_id).cloned()
    }
    
    fn process_tasks(&mut self) -> DistributedResult<Vec<TaskId>> {
        let now = SystemTime::now();
        let mut executed_tasks = Vec::new();
        
        for (task_id, task_info) in self.tasks.iter_mut() {
            if task_info.status == TaskStatus::Pending && 
               now.duration_since(task_info.scheduled_time).is_ok() {
                task_info.status = TaskStatus::Running;
                executed_tasks.push(*task_id);
                
                // If it's a recurring task, reschedule it
                if task_info.is_recurring {
                    if let Some(interval) = task_info.interval {
                        task_info.scheduled_time = now.checked_add(interval).unwrap_or(now);
                        task_info.status = TaskStatus::Pending;
                    }
                }
            }
        }
        
        Ok(executed_tasks)
    }
}

/// Information about a distributed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTask {
    pub function_id: FunctionId,
    pub payload: Vec<u8>,
    pub priority: TaskPriority,
    pub timeout: Option<Duration>,
    pub retry_policy: RetryPolicy,
}

/// Priority of a distributed task
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Retry policy for failed tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(60),
        }
    }
}

/// Status of a distributed schedule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleStatus {
    Active,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// Distributed scheduler with fault tolerance
pub trait Scheduler {
    /// Schedule a distributed task across multiple nodes
    fn schedule_distributed(&mut self, task: DistributedTask, nodes: Vec<NodeId>) -> DistributedResult<ScheduleId>;
    
    /// Reschedule a task when a node fails
    fn reschedule_on_failure(&mut self, schedule_id: ScheduleId, failed_node: NodeId) -> DistributedResult<()>;
    
    /// Get the status of a schedule
    fn get_schedule_status(&self, schedule_id: ScheduleId) -> Option<ScheduleStatus>;
    
    /// Pause a schedule
    fn pause_schedule(&mut self, schedule_id: ScheduleId) -> DistributedResult<()>;
    
    /// Resume a paused schedule
    fn resume_schedule(&mut self, schedule_id: ScheduleId) -> DistributedResult<()>;
    
    /// Cancel a schedule
    fn cancel_schedule(&mut self, schedule_id: ScheduleId) -> DistributedResult<()>;
    
    /// Get all active schedules
    fn get_active_schedules(&self) -> Vec<ScheduleId>;
}

/// Information about a schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub id: ScheduleId,
    pub task: DistributedTask,
    pub assigned_nodes: Vec<NodeId>,
    pub status: ScheduleStatus,
    pub created_at: SystemTime,
    pub last_updated: SystemTime,
    pub retry_count: u32,
}

/// Basic implementation of Scheduler
#[derive(Debug)]
pub struct BasicScheduler {
    schedules: HashMap<ScheduleId, ScheduleInfo>,
    next_schedule_id: u64,
    available_nodes: Vec<NodeId>,
}

impl BasicScheduler {
    pub fn new(available_nodes: Vec<NodeId>) -> Self {
        BasicScheduler {
            schedules: HashMap::new(),
            next_schedule_id: 0,
            available_nodes,
        }
    }
    
    fn next_schedule_id(&mut self) -> ScheduleId {
        let id = ScheduleId::new(self.next_schedule_id);
        self.next_schedule_id += 1;
        id
    }
    
    pub fn add_node(&mut self, node_id: NodeId) {
        if !self.available_nodes.contains(&node_id) {
            self.available_nodes.push(node_id);
        }
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        self.available_nodes.retain(|&id| id != node_id);
    }
}

impl Scheduler for BasicScheduler {
    fn schedule_distributed(&mut self, task: DistributedTask, nodes: Vec<NodeId>) -> DistributedResult<ScheduleId> {
        // Validate that all requested nodes are available
        for node_id in &nodes {
            if !self.available_nodes.contains(node_id) {
                return Err(DistributedError::NodeNotFound(*node_id));
            }
        }
        
        let schedule_id = self.next_schedule_id();
        let now = SystemTime::now();
        
        let schedule_info = ScheduleInfo {
            id: schedule_id,
            task,
            assigned_nodes: nodes,
            status: ScheduleStatus::Active,
            created_at: now,
            last_updated: now,
            retry_count: 0,
        };
        
        self.schedules.insert(schedule_id, schedule_info);
        Ok(schedule_id)
    }
    
    fn reschedule_on_failure(&mut self, schedule_id: ScheduleId, failed_node: NodeId) -> DistributedResult<()> {
        let schedule_info = self.schedules.get_mut(&schedule_id)
            .ok_or(DistributedError::ScheduleNotFound(schedule_id))?;
        
        // Remove the failed node from assigned nodes
        schedule_info.assigned_nodes.retain(|&id| id != failed_node);
        
        // Try to find a replacement node
        for &available_node in &self.available_nodes {
            if !schedule_info.assigned_nodes.contains(&available_node) {
                schedule_info.assigned_nodes.push(available_node);
                break;
            }
        }
        
        schedule_info.retry_count += 1;
        schedule_info.last_updated = SystemTime::now();
        
        // Check if we've exceeded retry limits
        if schedule_info.retry_count > schedule_info.task.retry_policy.max_retries {
            schedule_info.status = ScheduleStatus::Failed("Max retries exceeded".to_string());
        }
        
        Ok(())
    }
    
    fn get_schedule_status(&self, schedule_id: ScheduleId) -> Option<ScheduleStatus> {
        self.schedules.get(&schedule_id).map(|info| info.status.clone())
    }
    
    fn pause_schedule(&mut self, schedule_id: ScheduleId) -> DistributedResult<()> {
        let schedule_info = self.schedules.get_mut(&schedule_id)
            .ok_or(DistributedError::ScheduleNotFound(schedule_id))?;
        
        if schedule_info.status == ScheduleStatus::Active {
            schedule_info.status = ScheduleStatus::Paused;
            schedule_info.last_updated = SystemTime::now();
        }
        
        Ok(())
    }
    
    fn resume_schedule(&mut self, schedule_id: ScheduleId) -> DistributedResult<()> {
        let schedule_info = self.schedules.get_mut(&schedule_id)
            .ok_or(DistributedError::ScheduleNotFound(schedule_id))?;
        
        if schedule_info.status == ScheduleStatus::Paused {
            schedule_info.status = ScheduleStatus::Active;
            schedule_info.last_updated = SystemTime::now();
        }
        
        Ok(())
    }
    
    fn cancel_schedule(&mut self, schedule_id: ScheduleId) -> DistributedResult<()> {
        let schedule_info = self.schedules.get_mut(&schedule_id)
            .ok_or(DistributedError::ScheduleNotFound(schedule_id))?;
        
        schedule_info.status = ScheduleStatus::Cancelled;
        schedule_info.last_updated = SystemTime::now();
        
        Ok(())
    }
    
    fn get_active_schedules(&self) -> Vec<ScheduleId> {
        self.schedules
            .values()
            .filter(|info| info.status == ScheduleStatus::Active)
            .map(|info| info.id)
            .collect()
    }
}

/// Ordering guarantee for distributed queues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderingGuarantee {
    None,           // No ordering guarantees
    FIFO,          // First-in, first-out
    Priority,      // Priority-based ordering
    Causal,        // Causal ordering based on vector clocks
    Total,         // Total ordering across all nodes
}

/// Queue item with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub data: Vec<u8>,
    pub priority: Option<u32>,
    pub timestamp: SystemTime,
    pub node_id: NodeId,
    pub sequence_number: u64,
}

/// Distributed queue with ordering guarantees
pub trait Queue {
    /// Push an item to the queue
    fn push(&mut self, item: Vec<u8>, priority: Option<u32>) -> DistributedResult<()>;
    
    /// Pop an item from the queue
    fn pop(&mut self) -> DistributedResult<Option<Vec<u8>>>;
    
    /// Peek at the next item without removing it
    fn peek(&self) -> DistributedResult<Option<Vec<u8>>>;
    
    /// Get the current size of the queue
    fn size(&self) -> usize;
    
    /// Check if the queue is empty
    fn is_empty(&self) -> bool;
    
    /// Distribute the queue across multiple nodes
    fn distribute_across_nodes(&mut self, nodes: Vec<NodeId>) -> DistributedResult<()>;
    
    /// Get the ordering guarantee of this queue
    fn ordering_guarantee(&self) -> OrderingGuarantee;
    
    /// Clear all items from the queue
    fn clear(&mut self);
}

/// Basic implementation of a distributed queue
#[derive(Debug)]
pub struct BasicQueue {
    items: VecDeque<QueueItem>,
    ordering: OrderingGuarantee,
    node_id: NodeId,
    next_sequence: u64,
    distributed_nodes: Vec<NodeId>,
}

impl BasicQueue {
    pub fn new(node_id: NodeId, ordering: OrderingGuarantee) -> Self {
        BasicQueue {
            items: VecDeque::new(),
            ordering,
            node_id,
            next_sequence: 0,
            distributed_nodes: vec![node_id],
        }
    }
    
    fn next_sequence(&mut self) -> u64 {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        seq
    }
    
    fn sort_items(&mut self) {
        match self.ordering {
            OrderingGuarantee::Priority => {
                // Sort by priority (higher priority first), then by timestamp
                let mut items: Vec<_> = self.items.drain(..).collect();
                items.sort_by(|a, b| {
                    match (a.priority, b.priority) {
                        (Some(a_pri), Some(b_pri)) => b_pri.cmp(&a_pri).then(a.timestamp.cmp(&b.timestamp)),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.timestamp.cmp(&b.timestamp),
                    }
                });
                self.items = items.into();
            }
            OrderingGuarantee::Causal => {
                // Sort by sequence number for causal ordering
                let mut items: Vec<_> = self.items.drain(..).collect();
                items.sort_by_key(|item| item.sequence_number);
                self.items = items.into();
            }
            OrderingGuarantee::Total => {
                // Sort by timestamp for total ordering
                let mut items: Vec<_> = self.items.drain(..).collect();
                items.sort_by_key(|item| item.timestamp);
                self.items = items.into();
            }
            _ => {
                // No sorting needed for FIFO or None
            }
        }
    }
}

impl Queue for BasicQueue {
    fn push(&mut self, data: Vec<u8>, priority: Option<u32>) -> DistributedResult<()> {
        let item = QueueItem {
            data,
            priority,
            timestamp: SystemTime::now(),
            node_id: self.node_id,
            sequence_number: self.next_sequence(),
        };
        
        self.items.push_back(item);
        
        // Sort items if needed based on ordering guarantee
        if self.ordering != OrderingGuarantee::FIFO && self.ordering != OrderingGuarantee::None {
            self.sort_items();
        }
        
        Ok(())
    }
    
    fn pop(&mut self) -> DistributedResult<Option<Vec<u8>>> {
        Ok(self.items.pop_front().map(|item| item.data))
    }
    
    fn peek(&self) -> DistributedResult<Option<Vec<u8>>> {
        Ok(self.items.front().map(|item| item.data.clone()))
    }
    
    fn size(&self) -> usize {
        self.items.len()
    }
    
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn distribute_across_nodes(&mut self, nodes: Vec<NodeId>) -> DistributedResult<()> {
        self.distributed_nodes = nodes;
        Ok(())
    }
    
    fn ordering_guarantee(&self) -> OrderingGuarantee {
        self.ordering.clone()
    }
    
    fn clear(&mut self) {
        self.items.clear();
    }
}

/// Topic configuration for pub-sub channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub retention_policy: RetentionPolicy,
    pub ordering_guarantee: OrderingGuarantee,
    pub replication_factor: u32,
    pub partitioning_strategy: PartitioningStrategy,
    pub max_message_size: usize,
    pub ttl: Option<Duration>,
}

impl Default for TopicConfig {
    fn default() -> Self {
        TopicConfig {
            retention_policy: RetentionPolicy::TimeBasedRetention(Duration::from_secs(3600)), // 1 hour
            ordering_guarantee: OrderingGuarantee::FIFO,
            replication_factor: 1,
            partitioning_strategy: PartitioningStrategy::RoundRobin,
            max_message_size: 1024 * 1024, // 1MB
            ttl: None,
        }
    }
}

/// Retention policy for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    NoRetention,
    CountBasedRetention(usize),
    TimeBasedRetention(Duration),
    SizeBasedRetention(usize),
}

/// Partitioning strategy for distributed topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitioningStrategy {
    RoundRobin,
    Hash,
    Random,
    KeyBased(String),
}

/// Message in a pub-sub channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub topic: String,
    pub payload: Vec<u8>,
    pub timestamp: SystemTime,
    pub publisher_node: NodeId,
    pub headers: HashMap<String, String>,
}

/// Subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub topic_pattern: String,
    pub handler: FunctionId,
    pub node_id: NodeId,
    pub created_at: SystemTime,
    pub message_count: u64,
}

/// Pub-sub channel with topic routing and filtering
pub trait PubSubChannel {
    /// Publish a message to a topic
    fn publish(&mut self, topic: String, message: Vec<u8>) -> DistributedResult<MessageId>;
    
    /// Publish a message with headers
    fn publish_with_headers(&mut self, topic: String, message: Vec<u8>, headers: HashMap<String, String>) -> DistributedResult<MessageId>;
    
    /// Subscribe to a topic pattern with a handler function
    fn subscribe(&mut self, topic_pattern: String, handler: FunctionId) -> DistributedResult<SubscriptionId>;
    
    /// Unsubscribe from a topic
    fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> DistributedResult<()>;
    
    /// Create a new topic with configuration
    fn create_topic(&mut self, topic: String, config: TopicConfig) -> DistributedResult<()>;
    
    /// Delete a topic
    fn delete_topic(&mut self, topic: String) -> DistributedResult<()>;
    
    /// Get all active subscriptions
    fn get_subscriptions(&self) -> Vec<Subscription>;
    
    /// Get messages for a topic (for debugging/monitoring)
    fn get_topic_messages(&self, topic: String, limit: Option<usize>) -> Vec<Message>;
    
    /// Get topic configuration
    fn get_topic_config(&self, topic: String) -> Option<TopicConfig>;
    
    /// List all topics
    fn list_topics(&self) -> Vec<String>;
}

/// Basic implementation of PubSubChannel
#[derive(Debug)]
pub struct BasicPubSubChannel {
    topics: HashMap<String, TopicConfig>,
    messages: HashMap<String, VecDeque<Message>>,
    subscriptions: HashMap<SubscriptionId, Subscription>,
    next_message_id: u64,
    next_subscription_id: u64,
    node_id: NodeId,
}

impl BasicPubSubChannel {
    pub fn new(node_id: NodeId) -> Self {
        BasicPubSubChannel {
            topics: HashMap::new(),
            messages: HashMap::new(),
            subscriptions: HashMap::new(),
            next_message_id: 0,
            next_subscription_id: 0,
            node_id,
        }
    }
    
    fn next_message_id(&mut self) -> MessageId {
        let id = MessageId::new(self.next_message_id);
        self.next_message_id += 1;
        id
    }
    
    fn next_subscription_id(&mut self) -> SubscriptionId {
        let id = SubscriptionId::new(self.next_subscription_id);
        self.next_subscription_id += 1;
        id
    }
    
    #[allow(dead_code)]
    fn topic_matches_pattern(&self, topic: &str, pattern: &str) -> bool {
        // Simple pattern matching - supports * as wildcard
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return topic.starts_with(prefix) && topic.ends_with(suffix);
            }
        }
        
        topic == pattern
    }
    
    fn cleanup_old_messages(&mut self, topic: &str) {
        if let Some(config) = self.topics.get(topic) {
            if let Some(messages) = self.messages.get_mut(topic) {
                match &config.retention_policy {
                    RetentionPolicy::NoRetention => {
                        messages.clear();
                    }
                    RetentionPolicy::CountBasedRetention(max_count) => {
                        while messages.len() > *max_count {
                            messages.pop_front();
                        }
                    }
                    RetentionPolicy::TimeBasedRetention(max_age) => {
                        let cutoff = SystemTime::now() - *max_age;
                        while let Some(front) = messages.front() {
                            if front.timestamp < cutoff {
                                messages.pop_front();
                            } else {
                                break;
                            }
                        }
                    }
                    RetentionPolicy::SizeBasedRetention(max_size) => {
                        let mut total_size = messages.iter().map(|m| m.payload.len()).sum::<usize>();
                        while total_size > *max_size && !messages.is_empty() {
                            if let Some(msg) = messages.pop_front() {
                                total_size -= msg.payload.len();
                            }
                        }
                    }
                }
            }
        }
    }
}

impl PubSubChannel for BasicPubSubChannel {
    fn publish(&mut self, topic: String, message: Vec<u8>) -> DistributedResult<MessageId> {
        self.publish_with_headers(topic, message, HashMap::new())
    }
    
    fn publish_with_headers(&mut self, topic: String, message: Vec<u8>, headers: HashMap<String, String>) -> DistributedResult<MessageId> {
        // Check if topic exists, create with default config if not
        if !self.topics.contains_key(&topic) {
            self.create_topic(topic.clone(), TopicConfig::default())?;
        }
        
        // Check message size limit
        if let Some(config) = self.topics.get(&topic) {
            if message.len() > config.max_message_size {
                return Err(DistributedError::InvalidOperation(
                    format!("Message size {} exceeds limit {}", message.len(), config.max_message_size)
                ));
            }
        }
        
        let message_id = self.next_message_id();
        let msg = Message {
            id: message_id,
            topic: topic.clone(),
            payload: message,
            timestamp: SystemTime::now(),
            publisher_node: self.node_id,
            headers,
        };
        
        // Add message to topic
        self.messages.entry(topic.clone()).or_default().push_back(msg);
        
        // Clean up old messages based on retention policy
        self.cleanup_old_messages(&topic);
        
        Ok(message_id)
    }
    
    fn subscribe(&mut self, topic_pattern: String, handler: FunctionId) -> DistributedResult<SubscriptionId> {
        let subscription_id = self.next_subscription_id();
        let subscription = Subscription {
            id: subscription_id,
            topic_pattern,
            handler,
            node_id: self.node_id,
            created_at: SystemTime::now(),
            message_count: 0,
        };
        
        self.subscriptions.insert(subscription_id, subscription);
        Ok(subscription_id)
    }
    
    fn unsubscribe(&mut self, subscription_id: SubscriptionId) -> DistributedResult<()> {
        if self.subscriptions.remove(&subscription_id).is_some() {
            Ok(())
        } else {
            Err(DistributedError::SubscriptionNotFound(subscription_id))
        }
    }
    
    fn create_topic(&mut self, topic: String, config: TopicConfig) -> DistributedResult<()> {
        self.topics.insert(topic.clone(), config);
        self.messages.entry(topic).or_default();
        Ok(())
    }
    
    fn delete_topic(&mut self, topic: String) -> DistributedResult<()> {
        self.topics.remove(&topic);
        self.messages.remove(&topic);
        Ok(())
    }
    
    fn get_subscriptions(&self) -> Vec<Subscription> {
        self.subscriptions.values().cloned().collect()
    }
    
    fn get_topic_messages(&self, topic: String, limit: Option<usize>) -> Vec<Message> {
        if let Some(messages) = self.messages.get(&topic) {
            let messages: Vec<_> = messages.iter().cloned().collect();
            if let Some(limit) = limit {
                messages.into_iter().take(limit).collect()
            } else {
                messages
            }
        } else {
            Vec::new()
        }
    }
    
    fn get_topic_config(&self, topic: String) -> Option<TopicConfig> {
        self.topics.get(&topic).cloned()
    }
    
    fn list_topics(&self) -> Vec<String> {
        self.topics.keys().cloned().collect()
    }
}