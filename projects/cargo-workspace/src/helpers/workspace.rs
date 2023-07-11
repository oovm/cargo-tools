use crate::errors::{CargoError, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use glob::glob;

/// Represents a Cargo package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: Vec<String>,
    pub publish: bool,
}

/// Represents a Cargo workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoWorkspace {
    pub root: PathBuf,
    pub members: Vec<PathBuf>,
    pub packages: HashMap<String, CargoPackage>,
}

/// Finds the workspace root by searching for Cargo.toml
pub fn find_workspace_root(start_dir: &Path) -> Result<PathBuf> {
    let mut current_dir = start_dir.to_path_buf();

    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml)?;
            // Parse the TOML file properly instead of just checking for "[workspace]" string
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                // Check if this is a workspace by looking for a workspace section
                if toml_value.get("workspace").is_some() {
                    return Ok(current_dir);
                }
            }
        }

        if !current_dir.pop() {
            break;
        }
    }

    Err(CargoError::MissingWorkspace)
}

/// Parses a Cargo.toml file and extracts package information
pub fn parse_cargo_toml(path: &Path) -> Result<CargoPackage> {
    parse_cargo_toml_with_workspace(path, None)
}

/// Parses a Cargo.toml file and extracts package information with workspace context
pub fn parse_cargo_toml_with_workspace(path: &Path, workspace_package: Option<&toml::Value>) -> Result<CargoPackage> {
    let content = fs::read_to_string(path)?;
    let toml_value: toml::Value = toml::from_str(&content)?;

    let package = toml_value.get("package").ok_or_else(|| CargoError::InvalidToml("Missing [package] section".to_string()))?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CargoError::InvalidToml("Missing package name".to_string()))?
        .to_string();

    // Handle workspace inheritance for version
    let version = if let Some(version_value) = package.get("version") {
        if let Some(version_str) = version_value.as_str() {
            version_str.to_string()
        } else if let Some(version_table) = version_value.as_table() {
            // Check for workspace inheritance
            if version_table.get("workspace").and_then(|v| v.as_bool()).unwrap_or(false) {
                // Try to get version from workspace package
                if let Some(ws_pkg) = workspace_package {
                    ws_pkg.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0")
                        .to_string()
                } else {
                    "0.0.0".to_string() // Default version when workspace info is not available
                }
            } else {
                return Err(CargoError::InvalidToml("Invalid version format".to_string()));
            }
        } else {
            return Err(CargoError::InvalidToml("Invalid version format".to_string()));
        }
    } else {
        return Err(CargoError::InvalidToml("Missing package version".to_string()));
    };

    let publish = package.get("publish").and_then(|v| v.as_bool()).unwrap_or(true);

    let mut dependencies = Vec::new();

    // Extract dependencies from different sections
    for section in ["dependencies", "build-dependencies"] { // Skip dev-dependencies to avoid self-references
        if let Some(deps) = toml_value.get(section) {
            if let Some(deps_table) = deps.as_table() {
                for (dep_name, dep_value) in deps_table {
                    // Skip self-dependencies (packages depending on themselves)
                    if dep_name == &name {
                        continue;
                    }
                    
                    // Handle workspace dependencies that have workspace = true
                    if let Some(dep_obj) = dep_value.as_table() {
                        if dep_obj.get("workspace").and_then(|v| v.as_bool()).unwrap_or(false) {
                            // This is a workspace dependency, add it to dependencies
                            dependencies.push(dep_name.clone());
                            continue;
                        }
                    }
                    // Add regular dependencies
                    dependencies.push(dep_name.clone());
                }
            }
        }
    }

    Ok(CargoPackage { name, version, path: path.parent().unwrap_or(path).to_path_buf(), dependencies, publish })
}

/// Expands a glob pattern to matching paths
fn expand_glob_pattern(workspace_root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    
    // Convert the pattern to an absolute path pattern
    let absolute_pattern = workspace_root.join(pattern);
    let pattern_str = absolute_pattern.to_string_lossy();
    
    // Use the glob crate to expand the pattern
    match glob(&pattern_str) {
        Ok(entries) => {
            for entry in entries.flatten() {
                // Only include directories
                if entry.is_dir() {
                    result.push(entry);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read glob pattern {}: {}", pattern, e);
        }
    }
    
    Ok(result)
}

/// Discovers all packages in the workspace
pub fn discover_workspace_packages(workspace_root: &Path) -> Result<CargoWorkspace> {
    let workspace_cargo_toml = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&workspace_cargo_toml)?;
    let toml_value: toml::Value = toml::from_str(&content)?;

    let workspace_section =
        toml_value.get("workspace").ok_or_else(|| CargoError::InvalidToml("Missing [workspace] section".to_string()))?;

    // Get workspace package info for inheritance
    let workspace_package = workspace_section.get("package");

    let mut members = Vec::new();
    if let Some(members_value) = workspace_section.get("members") {
        if let Some(members_array) = members_value.as_array() {
            for member in members_array {
                if let Some(member_str) = member.as_str() {
                    members.push(member_str.to_string());
                }
            }
        }
    }

    let mut packages = HashMap::new();
    let mut member_paths = Vec::new();

    // Parse the workspace root package if it exists
    if workspace_cargo_toml.exists() {
        if let Ok(package) = parse_cargo_toml_with_workspace(&workspace_cargo_toml, workspace_package) {
            packages.insert(package.name.clone(), package);
        }
    }

    // Parse all member packages
    for member_pattern in &members {
        // Expand glob patterns
        let expanded_paths = expand_glob_pattern(workspace_root, member_pattern)?;
        
        for member_path in expanded_paths {
            member_paths.push(member_path.clone());
            
            let cargo_toml = member_path.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(package) = parse_cargo_toml_with_workspace(&cargo_toml, workspace_package) {
                    packages.insert(package.name.clone(), package);
                }
            }
        }
    }

    Ok(CargoWorkspace {
        root: workspace_root.to_path_buf(),
        members: member_paths,
        packages,
    })
}