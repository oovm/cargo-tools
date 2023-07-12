use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Checkpoint data for tracking published packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishCheckpoint {
    /// Workspace root directory (full path)
    pub workspace_root: PathBuf,
    /// Set of already published packages with version (format: "name@version")
    pub published_packages: HashSet<String>,
    /// Timestamp of the last checkpoint
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PublishCheckpoint {
    /// Create a new checkpoint for the given workspace
    pub fn new(workspace_root: PathBuf) -> Self {
        // Convert to absolute path
        let workspace_root = std::fs::canonicalize(&workspace_root)
            .unwrap_or_else(|_| workspace_root.clone());
            
        Self {
            workspace_root,
            published_packages: HashSet::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Mark a package as published
    pub fn mark_published(&mut self, package_name: String, package_version: String) {
        let package_with_version = format!("{}@{}", package_name, package_version);
        self.published_packages.insert(package_with_version);
        self.timestamp = chrono::Utc::now();
    }

    /// Check if a package is marked as published
    pub fn is_published(&self, package_name: &str, package_version: &str) -> bool {
        let package_with_version = format!("{}@{}", package_name, package_version);
        self.published_packages.contains(&package_with_version)
    }

    /// Get the checkpoint file path for a workspace
    pub fn checkpoint_path(workspace_root: &Path) -> PathBuf {
        let target_dir = workspace_root.join("target");
        target_dir.join("cargo-workspace-publish.toml")
    }

    /// Save the checkpoint to file
    pub fn save(&self) -> Result<(), crate::errors::CargoError> {
        let checkpoint_path = Self::checkpoint_path(&self.workspace_root);
        
        // Create target directory if it doesn't exist
        if let Some(parent) = checkpoint_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::errors::CargoError::IoError(format!("Failed to create directory {}: {}", parent.display(), e)))?;
        }

        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| crate::errors::CargoError::IoError(format!("Failed to serialize checkpoint: {}", e)))?;
        
        std::fs::write(&checkpoint_path, toml_string)
            .map_err(|e| crate::errors::CargoError::IoError(format!("Failed to write checkpoint file {}: {}", checkpoint_path.display(), e)))?;
        
        tracing::info!("Checkpoint saved to {}", checkpoint_path.display());
        Ok(())
    }

    /// Load checkpoint from file
    pub fn load(workspace_root: &Path) -> Result<Option<Self>, crate::errors::CargoError> {
        let checkpoint_path = Self::checkpoint_path(workspace_root);
        
        if !checkpoint_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&checkpoint_path)
            .map_err(|e| crate::errors::CargoError::IoError(format!("Failed to read checkpoint file {}: {}", checkpoint_path.display(), e)))?;
        
        let checkpoint: Self = toml::from_str(&content)
            .map_err(|e| crate::errors::CargoError::IoError(format!("Failed to parse checkpoint file {}: {}", checkpoint_path.display(), e)))?;
        
        tracing::info!("Checkpoint loaded from {}", checkpoint_path.display());
        Ok(Some(checkpoint))
    }

    /// Remove the checkpoint file
    pub fn remove(workspace_root: &Path) -> Result<(), crate::errors::CargoError> {
        let checkpoint_path = Self::checkpoint_path(workspace_root);
        
        if checkpoint_path.exists() {
            std::fs::remove_file(&checkpoint_path)
                .map_err(|e| crate::errors::CargoError::IoError(format!("Failed to remove checkpoint file {}: {}", checkpoint_path.display(), e)))?;
            tracing::info!("Checkpoint file removed: {}", checkpoint_path.display());
        }
        
        Ok(())
    }
}