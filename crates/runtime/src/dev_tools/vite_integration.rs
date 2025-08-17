use super::types::{BuildToolIntegration, BuildConfig, BuildResult, BuildStatus, BuildError, BuildArtifact, ArtifactType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// Vite build tool integration
pub struct ViteIntegration {
    /// Vite configuration file path
    config_path: Option<PathBuf>,
    /// Current build status
    status: BuildStatus,
    /// Last build result
    last_result: Option<BuildResult>,
    /// Project root directory
    project_root: PathBuf,
}

impl ViteIntegration {
    pub fn new(project_root: PathBuf, config_path: Option<PathBuf>) -> Self {
        Self {
            config_path,
            status: BuildStatus::Idle,
            last_result: None,
            project_root,
        }
    }

    /// Execute vite command
    fn execute_vite(&self, command: &str, args: Vec<String>, env_vars: HashMap<String, String>) -> Result<(String, bool), BuildError> {
        let mut cmd = Command::new("npx");
        cmd.arg("vite")
           .arg(command)
           .args(&args)
           .current_dir(&self.project_root)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Add config file if specified
        if let Some(ref config_path) = self.config_path {
            cmd.arg("--config").arg(config_path);
        }

        // Set environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output()
            .map_err(|e| BuildError::IOError(format!("Failed to execute vite: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let combined_output = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{stdout}\n{stderr}")
        };

        Ok((combined_output, output.status.success()))
    }

    /// Scan output directory for build artifacts
    fn scan_build_artifacts(&self, output_dir: &PathBuf) -> Result<Vec<BuildArtifact>, BuildError> {
        let mut artifacts = Vec::new();

        if !output_dir.exists() {
            return Ok(artifacts);
        }

        fn scan_directory(dir: &PathBuf, artifacts: &mut Vec<BuildArtifact>) -> Result<(), BuildError> {
            let entries = std::fs::read_dir(dir)
                .map_err(|e| BuildError::IOError(format!("Failed to read directory {}: {}", dir.display(), e)))?;

            for entry in entries {
                let entry = entry.map_err(|e| BuildError::IOError(format!("Failed to read directory entry: {e}")))?;
                let path = entry.path();

                if path.is_file() {
                    let metadata = entry.metadata()
                        .map_err(|e| BuildError::IOError(format!("Failed to read file metadata: {e}")))?;
                    
                    let size = metadata.len();
                    let artifact_type = determine_artifact_type(&path);
                    
                    // Generate a simple hash (in real implementation, would compute actual file hash)
                    let hash = format!("{:x}", size.wrapping_mul(31));
                    
                    // Check for source map
                    let source_map = if path.extension().and_then(|ext| ext.to_str()) == Some("js") {
                        let map_path = path.with_extension("js.map");
                        if map_path.exists() {
                            Some(map_path)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    artifacts.push(BuildArtifact {
                        path,
                        artifact_type,
                        hash,
                        size,
                        source_map,
                    });
                } else if path.is_dir() {
                    scan_directory(&path, artifacts)?;
                }
            }

            Ok(())
        }

        scan_directory(output_dir, &mut artifacts)?;
        Ok(artifacts)
    }
}

/// Determine artifact type from file extension
fn determine_artifact_type(path: &Path) -> ArtifactType {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("js") => ArtifactType::JavaScript,
        Some("ts") => ArtifactType::TypeScript,
        Some("wasm") => ArtifactType::WebAssembly,
        Some("map") => ArtifactType::SourceMap,
        Some("css") => ArtifactType::Stylesheet,
        Some("html") => ArtifactType::Other("html".to_string()),
        Some("svg") | Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") => ArtifactType::Asset,
        Some(ext) => ArtifactType::Other(ext.to_string()),
        None => ArtifactType::Other("unknown".to_string()),
    }
}

impl BuildToolIntegration for ViteIntegration {
    fn tool_name(&self) -> &str {
        "vite"
    }

    fn trigger_build(&mut self, config: BuildConfig) -> Result<BuildResult, BuildError> {
        self.status = BuildStatus::Building;
        let start_time = Instant::now();

        // Prepare vite arguments
        let mut args = Vec::new();

        // Add output directory
        args.push("--outDir".to_string());
        args.push(config.output_dir.to_string_lossy().to_string());

        // Add mode-specific arguments
        match config.mode.as_str() {
            "production" => {
                args.push("--minify".to_string());
            }
            "development" => {
                args.push("--sourcemap".to_string());
            }
            _ => {}
        }

        // Add additional flags
        args.extend(config.flags);

        // Set NODE_ENV based on mode
        let mut env_vars = config.env_vars;
        env_vars.insert("NODE_ENV".to_string(), config.mode.clone());

        // Execute vite build
        let (output, success) = self.execute_vite("build", args, env_vars)?;
        let duration = start_time.elapsed();

        // Scan for build artifacts
        let artifacts = if success {
            self.scan_build_artifacts(&config.output_dir)?
        } else {
            Vec::new()
        };

        // Extract errors and warnings from output
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for line in output.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("error") {
                errors.push(line.to_string());
            } else if line_lower.contains("warning") {
                warnings.push(line.to_string());
            }
        }

        let error_message = if !errors.is_empty() {
            errors.first().cloned().unwrap_or_else(|| "Build failed".to_string())
        } else {
            "Build failed".to_string()
        };

        let result = BuildResult {
            success,
            duration_ms: duration.as_millis() as u64,
            artifacts,
            output,
            errors,
            warnings,
        };

        self.status = if success {
            BuildStatus::Success
        } else {
            BuildStatus::Failed { 
                error: error_message
            }
        };

        self.last_result = Some(result.clone());
        Ok(result)
    }

    fn get_build_status(&self) -> BuildStatus {
        self.status.clone()
    }

    fn cancel_build(&mut self) -> Result<(), BuildError> {
        // In a real implementation, this would kill the vite process
        self.status = BuildStatus::Cancelled;
        Ok(())
    }

    fn get_artifacts(&self) -> Result<Vec<BuildArtifact>, BuildError> {
        if let Some(ref result) = self.last_result {
            Ok(result.artifacts.clone())
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vite_integration_creation() {
        let project_root = PathBuf::from(".");
        let config_path = Some(PathBuf::from("vite.config.js"));
        let integration = ViteIntegration::new(project_root.clone(), config_path.clone());
        
        assert_eq!(integration.tool_name(), "vite");
        assert_eq!(integration.project_root, project_root);
        assert_eq!(integration.config_path, config_path);
        assert!(matches!(integration.status, BuildStatus::Idle));
    }

    #[test]
    fn test_artifact_type_determination() {
        assert!(matches!(
            determine_artifact_type(&PathBuf::from("bundle.js")),
            ArtifactType::JavaScript
        ));
        
        assert!(matches!(
            determine_artifact_type(&PathBuf::from("bundle.js.map")),
            ArtifactType::SourceMap
        ));
        
        assert!(matches!(
            determine_artifact_type(&PathBuf::from("styles.css")),
            ArtifactType::Stylesheet
        ));

        assert!(matches!(
            determine_artifact_type(&PathBuf::from("index.html")),
            ArtifactType::Other(_)
        ));

        assert!(matches!(
            determine_artifact_type(&PathBuf::from("logo.png")),
            ArtifactType::Asset
        ));
    }
}