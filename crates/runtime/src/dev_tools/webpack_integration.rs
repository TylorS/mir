use super::types::{BuildToolIntegration, BuildConfig, BuildResult, BuildStatus, BuildError, BuildArtifact, ArtifactType};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// Webpack build tool integration
pub struct WebpackIntegration {
    /// Webpack configuration file path
    config_path: PathBuf,
    /// Current build status
    status: BuildStatus,
    /// Last build result
    last_result: Option<BuildResult>,
}

impl WebpackIntegration {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            status: BuildStatus::Idle,
            last_result: None,
        }
    }

    /// Parse webpack stats output
    fn parse_webpack_stats(&self, stats_json: &str) -> Result<Vec<BuildArtifact>, BuildError> {
        let stats: Value = serde_json::from_str(stats_json)
            .map_err(|e| BuildError::ConfigError(format!("Failed to parse webpack stats: {e}")))?;

        let mut artifacts = Vec::new();

        if let Some(assets) = stats.get("assets").and_then(|a| a.as_array()) {
            for asset in assets {
                if let (Some(name), Some(size)) = (
                    asset.get("name").and_then(|n| n.as_str()),
                    asset.get("size").and_then(|s| s.as_u64())
                ) {
                    let path = PathBuf::from(name);
                    let artifact_type = self.determine_artifact_type(&path);
                    
                    // Generate a simple hash (in real implementation, would compute actual hash)
                    let hash = format!("{:x}", size.wrapping_mul(31));
                    
                    // Check for source map
                    let source_map = if name.ends_with(".js") {
                        let map_name = format!("{name}.map");
                        if assets.iter().any(|a| a.get("name").and_then(|n| n.as_str()) == Some(&map_name)) {
                            Some(PathBuf::from(map_name))
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
                }
            }
        }

        Ok(artifacts)
    }

    /// Determine artifact type from file extension
    fn determine_artifact_type(&self, path: &Path) -> ArtifactType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("js") => ArtifactType::JavaScript,
            Some("ts") => ArtifactType::TypeScript,
            Some("wasm") => ArtifactType::WebAssembly,
            Some("map") => ArtifactType::SourceMap,
            Some("css") => ArtifactType::Stylesheet,
            Some(ext) => ArtifactType::Other(ext.to_string()),
            None => ArtifactType::Other("unknown".to_string()),
        }
    }

    /// Execute webpack command
    fn execute_webpack(&self, args: Vec<String>, env_vars: HashMap<String, String>) -> Result<(String, bool), BuildError> {
        let _start_time = Instant::now();
        
        let mut cmd = Command::new("npx");
        cmd.arg("webpack")
           .args(&args)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output()
            .map_err(|e| BuildError::IOError(format!("Failed to execute webpack: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let combined_output = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{stdout}\n{stderr}")
        };

        Ok((combined_output, output.status.success()))
    }
}

impl BuildToolIntegration for WebpackIntegration {
    fn tool_name(&self) -> &str {
        "webpack"
    }

    fn trigger_build(&mut self, config: BuildConfig) -> Result<BuildResult, BuildError> {
        self.status = BuildStatus::Building;
        let start_time = Instant::now();

        // Prepare webpack arguments
        let mut args = vec![
            "--config".to_string(),
            self.config_path.to_string_lossy().to_string(),
        ];

        // Add mode
        args.push("--mode".to_string());
        args.push(config.mode.clone());

        // Add output path
        args.push("--output-path".to_string());
        args.push(config.output_dir.to_string_lossy().to_string());

        // Add additional flags
        args.extend(config.flags);

        // Add stats output for parsing artifacts
        args.push("--stats".to_string());
        args.push("json".to_string());

        // Execute webpack
        let (output, success) = self.execute_webpack(args, config.env_vars)?;
        let duration = start_time.elapsed();

        // Parse output for artifacts and errors
        let mut artifacts = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Try to parse JSON stats from output
        if let Some(stats_start) = output.find('{') {
            if let Some(stats_end) = output.rfind('}') {
                let stats_json = &output[stats_start..=stats_end];
                match self.parse_webpack_stats(stats_json) {
                    Ok(parsed_artifacts) => artifacts = parsed_artifacts,
                    Err(e) => errors.push(format!("Failed to parse webpack stats: {e}")),
                }
            }
        }

        // Extract errors and warnings from output
        for line in output.lines() {
            if line.contains("ERROR") {
                errors.push(line.to_string());
            } else if line.contains("WARNING") {
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
        // In a real implementation, this would kill the webpack process
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
    fn test_webpack_integration_creation() {
        let config_path = PathBuf::from("webpack.config.js");
        let integration = WebpackIntegration::new(config_path.clone());
        
        assert_eq!(integration.tool_name(), "webpack");
        assert_eq!(integration.config_path, config_path);
        assert!(matches!(integration.status, BuildStatus::Idle));
    }

    #[test]
    fn test_artifact_type_determination() {
        let integration = WebpackIntegration::new(PathBuf::from("webpack.config.js"));
        
        assert!(matches!(
            integration.determine_artifact_type(&PathBuf::from("bundle.js")),
            ArtifactType::JavaScript
        ));
        
        assert!(matches!(
            integration.determine_artifact_type(&PathBuf::from("bundle.js.map")),
            ArtifactType::SourceMap
        ));
        
        assert!(matches!(
            integration.determine_artifact_type(&PathBuf::from("styles.css")),
            ArtifactType::Stylesheet
        ));
    }

    #[test]
    fn test_webpack_stats_parsing() {
        let integration = WebpackIntegration::new(PathBuf::from("webpack.config.js"));
        
        let stats_json = r#"{
            "assets": [
                {
                    "name": "bundle.js",
                    "size": 12345
                },
                {
                    "name": "bundle.js.map",
                    "size": 67890
                }
            ]
        }"#;
        
        let artifacts = integration.parse_webpack_stats(stats_json).unwrap();
        assert_eq!(artifacts.len(), 2);
        
        let js_artifact = &artifacts[0];
        assert_eq!(js_artifact.path, PathBuf::from("bundle.js"));
        assert!(matches!(js_artifact.artifact_type, ArtifactType::JavaScript));
        assert_eq!(js_artifact.size, 12345);
        assert_eq!(js_artifact.source_map, Some(PathBuf::from("bundle.js.map")));
    }
}