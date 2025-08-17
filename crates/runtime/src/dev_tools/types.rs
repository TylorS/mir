use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Development tool integration for HMR runtime
pub struct DevToolsIntegration {
    /// File system watcher for detecting changes
    file_watcher: Option<Box<dyn FileSystemWatcher>>,
    /// Build tool integrations
    build_tools: HashMap<String, Box<dyn BuildToolIntegration>>,
    /// Version control integration
    version_control: Option<Box<dyn VersionControlIntegration>>,
    /// CI/CD integration
    cicd_integration: Option<Box<dyn CICDIntegration>>,
    /// Configuration
    config: DevToolsConfig,
}

/// Configuration for development tools integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsConfig {
    /// Enable file system watching
    pub enable_file_watching: bool,
    /// File patterns to watch
    pub watch_patterns: Vec<String>,
    /// File patterns to ignore
    pub ignore_patterns: Vec<String>,
    /// Debounce delay for file changes (milliseconds)
    pub debounce_delay_ms: u64,
    /// Enable build tool integration
    pub enable_build_tools: bool,
    /// Enable version control integration
    pub enable_version_control: bool,
    /// Enable CI/CD integration
    pub enable_cicd: bool,
}

/// File system change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    /// Path that changed
    pub path: PathBuf,
    /// Type of change
    pub change_type: FileChangeType,
    /// Timestamp of change
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: FileChangeMetadata,
}

/// Type of file system change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

/// Additional metadata for file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeMetadata {
    /// File size (if available)
    pub size: Option<u64>,
    /// File hash (if computed)
    pub hash: Option<String>,
    /// MIME type (if detected)
    pub mime_type: Option<String>,
}

/// File system watcher trait
pub trait FileSystemWatcher: Send + Sync {
    /// Start watching for file changes
    fn start_watching(&mut self, paths: Vec<PathBuf>, patterns: Vec<String>) -> Result<(), FileWatchError>;
    
    /// Stop watching for file changes
    fn stop_watching(&mut self) -> Result<(), FileWatchError>;
    
    /// Get pending file change events
    fn get_events(&mut self) -> Result<Vec<FileChangeEvent>, FileWatchError>;
    
    /// Check if watcher is active
    fn is_watching(&self) -> bool;
}

/// Build tool integration trait
pub trait BuildToolIntegration: Send + Sync {
    /// Get the name of the build tool
    fn tool_name(&self) -> &str;
    
    /// Trigger a build
    fn trigger_build(&mut self, config: BuildConfig) -> Result<BuildResult, BuildError>;
    
    /// Get build status
    fn get_build_status(&self) -> BuildStatus;
    
    /// Cancel ongoing build
    fn cancel_build(&mut self) -> Result<(), BuildError>;
    
    /// Get build artifacts
    fn get_artifacts(&self) -> Result<Vec<BuildArtifact>, BuildError>;
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Target environment
    pub target: String,
    /// Build mode (development, production, etc.)
    pub mode: String,
    /// Additional build flags
    pub flags: Vec<String>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Source directories
    pub source_dirs: Vec<PathBuf>,
    /// Output directory
    pub output_dir: PathBuf,
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Build success status
    pub success: bool,
    /// Build duration
    pub duration_ms: u64,
    /// Generated artifacts
    pub artifacts: Vec<BuildArtifact>,
    /// Build output/logs
    pub output: String,
    /// Error messages (if any)
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

/// Build artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// Artifact path
    pub path: PathBuf,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// Content hash
    pub hash: String,
    /// Size in bytes
    pub size: u64,
    /// Source map (if available)
    pub source_map: Option<PathBuf>,
}

/// Type of build artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    JavaScript,
    TypeScript,
    WebAssembly,
    SourceMap,
    Stylesheet,
    Asset,
    Other(String),
}

/// Build status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Idle,
    Building,
    Success,
    Failed { error: String },
    Cancelled,
}

/// Version control integration trait
pub trait VersionControlIntegration: Send + Sync {
    /// Get current commit hash
    fn get_current_commit(&self) -> Result<String, VCError>;
    
    /// Get commit history
    fn get_commit_history(&self, limit: usize) -> Result<Vec<CommitInfo>, VCError>;
    
    /// Create a rollback point
    fn create_rollback_point(&mut self, message: String) -> Result<String, VCError>;
    
    /// Rollback to a specific commit
    fn rollback_to_commit(&mut self, commit_hash: String) -> Result<(), VCError>;
    
    /// Get changed files since commit
    fn get_changed_files(&self, since_commit: Option<String>) -> Result<Vec<FileChange>, VCError>;
    
    /// Get branch information
    fn get_branch_info(&self) -> Result<BranchInfo, VCError>;
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Author
    pub author: String,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Changed files
    pub changed_files: Vec<String>,
}

/// File change in version control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub path: PathBuf,
    /// Change type
    pub change_type: VCChangeType,
    /// Lines added
    pub lines_added: u32,
    /// Lines removed
    pub lines_removed: u32,
}

/// Version control change type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VCChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    Copied { from: PathBuf },
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Current branch name
    pub current_branch: String,
    /// Available branches
    pub branches: Vec<String>,
    /// Remote branches
    pub remote_branches: Vec<String>,
    /// Uncommitted changes
    pub has_uncommitted_changes: bool,
}

/// CI/CD integration trait
pub trait CICDIntegration: Send + Sync {
    /// Trigger deployment
    fn trigger_deployment(&mut self, config: DeploymentConfig) -> Result<DeploymentResult, CICDError>;
    
    /// Get deployment status
    fn get_deployment_status(&self, deployment_id: String) -> Result<DeploymentStatus, CICDError>;
    
    /// Cancel deployment
    fn cancel_deployment(&mut self, deployment_id: String) -> Result<(), CICDError>;
    
    /// Get deployment history
    fn get_deployment_history(&self, limit: usize) -> Result<Vec<DeploymentInfo>, CICDError>;
    
    /// Create automated deployment pipeline
    fn create_pipeline(&mut self, pipeline_config: PipelineConfig) -> Result<String, CICDError>;
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Target environment
    pub environment: String,
    /// Deployment strategy
    pub strategy: DeploymentStrategy,
    /// Artifacts to deploy
    pub artifacts: Vec<PathBuf>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Health check configuration
    pub health_checks: Vec<HealthCheckConfig>,
}

/// Deployment strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    BlueGreen,
    RollingUpdate { batch_size: u32 },
    Canary { percentage: f32 },
    Recreate,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check URL
    pub url: String,
    /// Expected status code
    pub expected_status: u16,
    /// Timeout in seconds
    pub timeout_seconds: u32,
    /// Number of retries
    pub retries: u32,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// Deployment ID
    pub deployment_id: String,
    /// Success status
    pub success: bool,
    /// Deployment URL (if successful)
    pub deployment_url: Option<String>,
    /// Deployment logs
    pub logs: String,
    /// Duration
    pub duration_ms: u64,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    InProgress { progress_percentage: f32 },
    Success { url: String },
    Failed { error: String },
    Cancelled,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Deployment ID
    pub id: String,
    /// Environment
    pub environment: String,
    /// Commit hash
    pub commit_hash: String,
    /// Deployment time
    pub timestamp: SystemTime,
    /// Status
    pub status: DeploymentStatus,
    /// Deployment URL
    pub url: Option<String>,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Pipeline name
    pub name: String,
    /// Trigger conditions
    pub triggers: Vec<PipelineTrigger>,
    /// Pipeline stages
    pub stages: Vec<PipelineStage>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

/// Pipeline trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineTrigger {
    OnPush { branches: Vec<String> },
    OnPullRequest { target_branches: Vec<String> },
    OnSchedule { cron: String },
    OnManual,
}

/// Pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Stage name
    pub name: String,
    /// Commands to execute
    pub commands: Vec<String>,
    /// Dependencies on other stages
    pub depends_on: Vec<String>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

/// Error types for development tools integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileWatchError {
    NotEnabled,
    NoWatcher,
    WatcherError(String),
    IOError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildError {
    NotEnabled,
    ToolNotFound(String),
    BuildFailed(String),
    ConfigError(String),
    IOError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VCError {
    NotEnabled,
    NoIntegration,
    GitError(String),
    CommitNotFound(String),
    IOError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CICDError {
    NotEnabled,
    NoIntegration,
    DeploymentFailed(String),
    ConfigError(String),
    NetworkError(String),
}

impl DevToolsIntegration {
    pub fn new(config: DevToolsConfig) -> Self {
        Self {
            file_watcher: None,
            build_tools: HashMap::new(),
            version_control: None,
            cicd_integration: None,
            config,
        }
    }

    /// Set file system watcher
    pub fn set_file_watcher(&mut self, watcher: Box<dyn FileSystemWatcher>) {
        self.file_watcher = Some(watcher);
    }

    /// Add build tool integration
    pub fn add_build_tool(&mut self, integration: Box<dyn BuildToolIntegration>) {
        let name = integration.tool_name().to_string();
        self.build_tools.insert(name, integration);
    }

    /// Set version control integration
    pub fn set_version_control(&mut self, vc: Box<dyn VersionControlIntegration>) {
        self.version_control = Some(vc);
    }

    /// Set CI/CD integration
    pub fn set_cicd_integration(&mut self, cicd: Box<dyn CICDIntegration>) {
        self.cicd_integration = Some(cicd);
    }

    /// Start file watching
    pub fn start_file_watching(&mut self, paths: Vec<PathBuf>) -> Result<(), FileWatchError> {
        if !self.config.enable_file_watching {
            return Err(FileWatchError::NotEnabled);
        }

        if let Some(ref mut watcher) = self.file_watcher {
            watcher.start_watching(paths, self.config.watch_patterns.clone())
        } else {
            Err(FileWatchError::NoWatcher)
        }
    }

    /// Get file change events
    pub fn get_file_changes(&mut self) -> Result<Vec<FileChangeEvent>, FileWatchError> {
        if let Some(ref mut watcher) = self.file_watcher {
            watcher.get_events()
        } else {
            Ok(Vec::new())
        }
    }

    /// Trigger build with specific tool
    pub fn trigger_build(&mut self, tool_name: &str, config: BuildConfig) -> Result<BuildResult, BuildError> {
        if !self.config.enable_build_tools {
            return Err(BuildError::NotEnabled);
        }

        if let Some(tool) = self.build_tools.get_mut(tool_name) {
            tool.trigger_build(config)
        } else {
            Err(BuildError::ToolNotFound(tool_name.to_string()))
        }
    }

    /// Get current commit from version control
    pub fn get_current_commit(&self) -> Result<String, VCError> {
        if !self.config.enable_version_control {
            return Err(VCError::NotEnabled);
        }

        if let Some(ref vc) = self.version_control {
            vc.get_current_commit()
        } else {
            Err(VCError::NoIntegration)
        }
    }

    /// Create rollback point
    pub fn create_rollback_point(&mut self, message: String) -> Result<String, VCError> {
        if let Some(ref mut vc) = self.version_control {
            vc.create_rollback_point(message)
        } else {
            Err(VCError::NoIntegration)
        }
    }

    /// Trigger deployment
    pub fn trigger_deployment(&mut self, config: DeploymentConfig) -> Result<DeploymentResult, CICDError> {
        if !self.config.enable_cicd {
            return Err(CICDError::NotEnabled);
        }

        if let Some(ref mut cicd) = self.cicd_integration {
            cicd.trigger_deployment(config)
        } else {
            Err(CICDError::NoIntegration)
        }
    }
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enable_file_watching: true,
            watch_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
                "**/*.json".to_string(),
                "**/*.toml".to_string(),
            ],
            ignore_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
            ],
            debounce_delay_ms: 100,
            enable_build_tools: true,
            enable_version_control: true,
            enable_cicd: false,
        }
    }
}

impl std::fmt::Display for FileWatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileWatchError::NotEnabled => write!(f, "File watching is not enabled"),
            FileWatchError::NoWatcher => write!(f, "No file watcher configured"),
            FileWatchError::WatcherError(msg) => write!(f, "File watcher error: {msg}"),
            FileWatchError::IOError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for FileWatchError {}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::NotEnabled => write!(f, "Build tools are not enabled"),
            BuildError::ToolNotFound(tool) => write!(f, "Build tool not found: {tool}"),
            BuildError::BuildFailed(msg) => write!(f, "Build failed: {msg}"),
            BuildError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            BuildError::IOError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for BuildError {}

impl std::fmt::Display for VCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VCError::NotEnabled => write!(f, "Version control is not enabled"),
            VCError::NoIntegration => write!(f, "No version control integration configured"),
            VCError::GitError(msg) => write!(f, "Git error: {msg}"),
            VCError::CommitNotFound(hash) => write!(f, "Commit not found: {hash}"),
            VCError::IOError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for VCError {}

impl std::fmt::Display for CICDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CICDError::NotEnabled => write!(f, "CI/CD integration is not enabled"),
            CICDError::NoIntegration => write!(f, "No CI/CD integration configured"),
            CICDError::DeploymentFailed(msg) => write!(f, "Deployment failed: {msg}"),
            CICDError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            CICDError::NetworkError(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl std::error::Error for CICDError {}
#[
cfg(test)]
mod tests {
    use super::*;

    struct MockFileWatcher {
        watching: bool,
        events: Vec<FileChangeEvent>,
    }

    impl FileSystemWatcher for MockFileWatcher {
        fn start_watching(&mut self, _paths: Vec<PathBuf>, _patterns: Vec<String>) -> Result<(), FileWatchError> {
            self.watching = true;
            Ok(())
        }

        fn stop_watching(&mut self) -> Result<(), FileWatchError> {
            self.watching = false;
            Ok(())
        }

        fn get_events(&mut self) -> Result<Vec<FileChangeEvent>, FileWatchError> {
            Ok(std::mem::take(&mut self.events))
        }

        fn is_watching(&self) -> bool {
            self.watching
        }
    }

    struct MockBuildTool {
        name: String,
        status: BuildStatus,
    }

    impl BuildToolIntegration for MockBuildTool {
        fn tool_name(&self) -> &str {
            &self.name
        }

        fn trigger_build(&mut self, _config: BuildConfig) -> Result<BuildResult, BuildError> {
            self.status = BuildStatus::Success;
            Ok(BuildResult {
                success: true,
                duration_ms: 1000,
                artifacts: vec![],
                output: "Build successful".to_string(),
                errors: vec![],
                warnings: vec![],
            })
        }

        fn get_build_status(&self) -> BuildStatus {
            self.status.clone()
        }

        fn cancel_build(&mut self) -> Result<(), BuildError> {
            self.status = BuildStatus::Cancelled;
            Ok(())
        }

        fn get_artifacts(&self) -> Result<Vec<BuildArtifact>, BuildError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_dev_tools_integration() {
        let config = DevToolsConfig::default();
        let mut integration = DevToolsIntegration::new(config);

        // Add mock file watcher
        let watcher = MockFileWatcher {
            watching: false,
            events: vec![],
        };
        integration.set_file_watcher(Box::new(watcher));

        // Add mock build tool
        let build_tool = MockBuildTool {
            name: "webpack".to_string(),
            status: BuildStatus::Idle,
        };
        integration.add_build_tool(Box::new(build_tool));

        // Test file watching
        assert!(integration.start_file_watching(vec![PathBuf::from("src")]).is_ok());

        // Test build triggering
        let build_config = BuildConfig {
            target: "web".to_string(),
            mode: "development".to_string(),
            flags: vec![],
            env_vars: HashMap::new(),
            source_dirs: vec![PathBuf::from("src")],
            output_dir: PathBuf::from("dist"),
        };

        let result = integration.trigger_build("webpack", build_config);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_dev_tools_config_default() {
        let config = DevToolsConfig::default();
        
        assert!(config.enable_file_watching);
        assert!(config.enable_build_tools);
        assert!(config.enable_version_control);
        assert!(!config.enable_cicd);
        assert_eq!(config.debounce_delay_ms, 100);
        assert!(!config.watch_patterns.is_empty());
        assert!(!config.ignore_patterns.is_empty());
    }
}