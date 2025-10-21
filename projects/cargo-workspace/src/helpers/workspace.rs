use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use crate::errors::{CargoError, Result};

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
            if content.contains("[workspace]") {
                return Ok(current_dir);
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
    let content = fs::read_to_string(path)?;
    let toml_value: toml::Value = toml::from_str(&content)?;
    
    let package = toml_value.get("package")
        .ok_or_else(|| CargoError::InvalidToml("Missing [package] section".to_string()))?;
    
    let name = package.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CargoError::InvalidToml("Missing package name".to_string()))?
        .to_string();
    
    let version = package.get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CargoError::InvalidToml("Missing package version".to_string()))?
        .to_string();
    
    let publish = package.get("publish")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let mut dependencies = Vec::new();
    
    // Extract dependencies from different sections
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = toml_value.get(section) {
            if let Some(deps_table) = deps.as_table() {
                for dep_name in deps_table.keys() {
                    dependencies.push(dep_name.clone());
                }
            }
        }
    }
    
    Ok(CargoPackage {
        name,
        version,
        path: path.parent().unwrap_or(path).to_path_buf(),
        dependencies,
        publish,
    })
}

/// Discovers all packages in the workspace
pub fn discover_workspace_packages(workspace_root: &Path) -> Result<CargoWorkspace> {
    let workspace_cargo_toml = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&workspace_cargo_toml)?;
    let toml_value: toml::Value = toml::from_str(&content)?;
    
    let workspace_section = toml_value.get("workspace")
        .ok_or_else(|| CargoError::InvalidToml("Missing [workspace] section".to_string()))?;
    
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
    
    // Parse the workspace root package if it exists
    if workspace_cargo_toml.exists() {
        if let Ok(package) = parse_cargo_toml(&workspace_cargo_toml) {
            packages.insert(package.name.clone(), package);
        }
    }
    
    // Parse all member packages
    for member_pattern in &members {
        let member_path = workspace_root.join(&member_pattern);
        
        // Handle glob patterns
        if member_pattern.contains('*') {
            for entry in fs::read_dir(workspace_root)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    let cargo_toml = path.join("Cargo.toml");
                    if cargo_toml.exists() {
                        if let Ok(package) = parse_cargo_toml(&cargo_toml) {
                            packages.insert(package.name.clone(), package);
                        }
                    }
                }
            }
        } else {
            let cargo_toml = member_path.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(package) = parse_cargo_toml(&cargo_toml) {
                    packages.insert(package.name.clone(), package);
                }
            }
        }
    }
    
    Ok(CargoWorkspace {
        root: workspace_root.to_path_buf(),
        members: members.iter().map(|m| workspace_root.join(m)).collect(),
        packages,
    })
}