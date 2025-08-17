use crate::source_mapping::{SourceMapRegistry, SourcePosition, GeneratedPosition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Profiling data attributed to original source constructs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    /// Total execution time
    pub total_time: Duration,
    /// Number of invocations
    pub call_count: u64,
    /// Average execution time per call
    pub average_time: Duration,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
    /// Memory allocations attributed to this location
    pub memory_allocated: u64,
    /// Memory deallocations attributed to this location
    pub memory_deallocated: u64,
}

/// Source-attributed profiling entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProfileEntry {
    /// Source location
    pub source_position: SourcePosition,
    /// Profiling data for this location
    pub profile_data: ProfileData,
    /// Child entries (for hierarchical profiling)
    pub children: Vec<SourceProfileEntry>,
}

/// Profiling session that attributes performance to original source
pub struct SourceProfiler {
    source_map_registry: SourceMapRegistry,
    /// Active profiling sessions by module
    active_sessions: HashMap<crate::ContentHash, ModuleProfilingSession>,
    /// Completed profiling data
    completed_profiles: HashMap<crate::ContentHash, Vec<SourceProfileEntry>>,
    /// Global profiling settings
    settings: ProfilingSettings,
}

/// Profiling settings
#[derive(Debug, Clone)]
pub struct ProfilingSettings {
    /// Enable function-level profiling
    pub profile_functions: bool,
    /// Enable line-level profiling
    pub profile_lines: bool,
    /// Enable memory profiling
    pub profile_memory: bool,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Minimum execution time to record
    pub min_duration_threshold: Duration,
}

/// Profiling session for a single module
#[allow(dead_code)]
struct ModuleProfilingSession {
    module_hash: crate::ContentHash,
    /// Stack of currently executing locations
    execution_stack: Vec<ExecutionFrame>,
    /// Accumulated profiling data
    profile_data: HashMap<SourcePosition, ProfileData>,
    /// Start time of the session
    session_start: Instant,
}

/// Execution frame for profiling
struct ExecutionFrame {
    source_position: SourcePosition,
    start_time: Instant,
    memory_at_start: u64,
}

impl SourceProfiler {
    pub fn new(source_map_registry: SourceMapRegistry) -> Self {
        Self {
            source_map_registry,
            active_sessions: HashMap::new(),
            completed_profiles: HashMap::new(),
            settings: ProfilingSettings::default(),
        }
    }

    /// Start profiling a module
    pub fn start_profiling(&mut self, module_hash: crate::ContentHash) {
        let session = ModuleProfilingSession {
            module_hash,
            execution_stack: Vec::new(),
            profile_data: HashMap::new(),
            session_start: Instant::now(),
        };
        
        self.active_sessions.insert(module_hash, session);
    }

    /// Stop profiling a module and generate report
    pub fn stop_profiling(&mut self, module_hash: &crate::ContentHash) -> Option<Vec<SourceProfileEntry>> {
        let session = self.active_sessions.remove(module_hash)?;
        let profile_entries = self.generate_profile_report(session);
        self.completed_profiles.insert(*module_hash, profile_entries.clone());
        Some(profile_entries)
    }

    /// Record function entry
    pub fn enter_function(&mut self, module_hash: &crate::ContentHash, generated_pos: GeneratedPosition, memory_usage: u64) {
        if let Some(session) = self.active_sessions.get_mut(module_hash) {
            if let Some(source_pos) = self.source_map_registry.map_to_source(module_hash, generated_pos) {
                let frame = ExecutionFrame {
                    source_position: source_pos,
                    start_time: Instant::now(),
                    memory_at_start: memory_usage,
                };
                session.execution_stack.push(frame);
            }
        }
    }

    /// Record function exit
    pub fn exit_function(&mut self, module_hash: &crate::ContentHash, memory_usage: u64) {
        if let Some(session) = self.active_sessions.get_mut(module_hash) {
            if let Some(frame) = session.execution_stack.pop() {
                let execution_time = frame.start_time.elapsed();
                let memory_delta = memory_usage.saturating_sub(frame.memory_at_start);
                
                // Skip if below threshold
                if execution_time < self.settings.min_duration_threshold {
                    return;
                }
                
                // Update or create profile data
                let profile_data = session.profile_data.entry(frame.source_position.clone())
                    .or_insert_with(|| ProfileData {
                        total_time: Duration::ZERO,
                        call_count: 0,
                        average_time: Duration::ZERO,
                        min_time: Duration::MAX,
                        max_time: Duration::ZERO,
                        memory_allocated: 0,
                        memory_deallocated: 0,
                    });
                
                profile_data.total_time += execution_time;
                profile_data.call_count += 1;
                profile_data.average_time = profile_data.total_time / profile_data.call_count as u32;
                profile_data.min_time = profile_data.min_time.min(execution_time);
                profile_data.max_time = profile_data.max_time.max(execution_time);
                
                if memory_delta > 0 {
                    profile_data.memory_allocated += memory_delta;
                } else {
                    profile_data.memory_deallocated += memory_delta;
                }
            }
        }
    }

    /// Record line execution (for line-level profiling)
    pub fn record_line_execution(&mut self, module_hash: &crate::ContentHash, generated_pos: GeneratedPosition, execution_time: Duration) {
        if !self.settings.profile_lines {
            return;
        }
        
        if let Some(session) = self.active_sessions.get_mut(module_hash) {
            if let Some(source_pos) = self.source_map_registry.map_to_source(module_hash, generated_pos) {
                let profile_data = session.profile_data.entry(source_pos)
                    .or_insert_with(|| ProfileData {
                        total_time: Duration::ZERO,
                        call_count: 0,
                        average_time: Duration::ZERO,
                        min_time: Duration::MAX,
                        max_time: Duration::ZERO,
                        memory_allocated: 0,
                        memory_deallocated: 0,
                    });
                
                profile_data.total_time += execution_time;
                profile_data.call_count += 1;
                profile_data.average_time = profile_data.total_time / profile_data.call_count as u32;
                profile_data.min_time = profile_data.min_time.min(execution_time);
                profile_data.max_time = profile_data.max_time.max(execution_time);
            }
        }
    }

    /// Get profiling report for a module
    pub fn get_profile_report(&self, module_hash: &crate::ContentHash) -> Option<&Vec<SourceProfileEntry>> {
        self.completed_profiles.get(module_hash)
    }

    /// Generate profile report from session data
    fn generate_profile_report(&self, session: ModuleProfilingSession) -> Vec<SourceProfileEntry> {
        let mut entries: Vec<SourceProfileEntry> = session.profile_data
            .into_iter()
            .map(|(source_pos, profile_data)| SourceProfileEntry {
                source_position: source_pos,
                profile_data,
                children: Vec::new(), // TODO: Build hierarchical structure
            })
            .collect();
        
        // Sort by total execution time (descending)
        entries.sort_by(|a, b| b.profile_data.total_time.cmp(&a.profile_data.total_time));
        
        entries
    }

    /// Export profiling data in various formats
    pub fn export_profile(&self, module_hash: &crate::ContentHash, format: ProfileExportFormat) -> Option<String> {
        let profile = self.get_profile_report(module_hash)?;
        
        match format {
            ProfileExportFormat::Json => {
                serde_json::to_string_pretty(profile).ok()
            }
            ProfileExportFormat::FlameGraph => {
                self.generate_flame_graph_data(profile)
            }
            ProfileExportFormat::CallTree => {
                self.generate_call_tree_data(profile)
            }
        }
    }

    /// Generate flame graph compatible data
    fn generate_flame_graph_data(&self, profile: &[SourceProfileEntry]) -> Option<String> {
        let mut output = String::new();
        
        for entry in profile {
            let stack_name = format!("{}:{}:{}", 
                entry.source_position.file.display(),
                entry.source_position.line + 1,
                entry.source_position.name.as_deref().unwrap_or("unknown")
            );
            
            output.push_str(&format!("{} {}\n", 
                stack_name, 
                entry.profile_data.total_time.as_micros()
            ));
        }
        
        Some(output)
    }

    /// Generate call tree data
    fn generate_call_tree_data(&self, profile: &[SourceProfileEntry]) -> Option<String> {
        let mut output = String::new();
        
        for entry in profile {
            output.push_str(&format!("{}:{}:{} - {} calls, {}ms total, {}ms avg\n",
                entry.source_position.file.display(),
                entry.source_position.line + 1,
                entry.source_position.name.as_deref().unwrap_or("unknown"),
                entry.profile_data.call_count,
                entry.profile_data.total_time.as_millis(),
                entry.profile_data.average_time.as_millis()
            ));
        }
        
        Some(output)
    }

    /// Update profiling settings
    pub fn update_settings(&mut self, settings: ProfilingSettings) {
        self.settings = settings;
    }
}

/// Profile export formats
#[derive(Debug, Clone)]
pub enum ProfileExportFormat {
    Json,
    FlameGraph,
    CallTree,
}

impl Default for ProfilingSettings {
    fn default() -> Self {
        Self {
            profile_functions: true,
            profile_lines: false,
            profile_memory: true,
            sampling_rate: 1.0,
            min_duration_threshold: Duration::from_micros(1),
        }
    }
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            total_time: Duration::ZERO,
            call_count: 0,
            average_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            memory_allocated: 0,
            memory_deallocated: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_mapping::{SourceMap, SourceMapping};
    use std::path::PathBuf;

    #[test]
    fn test_profiling_session() {
        let mut registry = crate::source_mapping::SourceMapRegistry::new();
        let module_hash = crate::ContentHash::from_bytes([1; 8]);
        
        // Register a simple source map
        let source_map = SourceMap {
            version: 3,
            sources: vec![PathBuf::from("src/main.rs")],
            sources_content: None,
            mappings: vec![
                SourceMapping {
                    generated_line: 0,
                    generated_column: 0,
                    source_index: Some(0),
                    original_line: Some(5),
                    original_column: Some(0),
                    name_index: Some(0),
                }
            ],
            names: vec!["test_function".to_string()],
        };
        
        registry.register_source_map(module_hash, source_map);
        
        let mut profiler = SourceProfiler::new(registry);
        profiler.start_profiling(module_hash);
        
        // Simulate function execution
        profiler.enter_function(&module_hash, GeneratedPosition { line: 0, column: 0 }, 1000);
        std::thread::sleep(Duration::from_millis(1));
        profiler.exit_function(&module_hash, 1100);
        
        let report = profiler.stop_profiling(&module_hash);
        assert!(report.is_some());
        
        let entries = report.unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries[0].profile_data.call_count, 1);
        assert!(entries[0].profile_data.total_time > Duration::ZERO);
    }
}