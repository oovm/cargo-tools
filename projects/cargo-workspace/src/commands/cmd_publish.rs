use crate::{
    errors::{CargoError, Result},
    helpers::workspace::CargoPackage,
    CommandOptions,
};
use clap::Parser;
use std::{path::PathBuf, process::Command};
use tracing::{error, info, warn};

#[derive(Debug, Parser)]
pub struct PublishCommand {
    /// The path to the workspace root directory
    #[arg(short, long, default_value = ".")]
    pub workspace_root: PathBuf,

    /// Run in dry-run mode without actually publishing
    #[arg(long)]
    pub dry_run: bool,

    /// Skip packages that are already published
    #[arg(long)]
    pub skip_published: bool,

    /// Registry token for publishing
    #[arg(long)]
    pub token: Option<String>,
}

impl PublishCommand {
    pub async fn run(&self, shared: &CommandOptions) -> std::result::Result<(), CargoError> {
        // Use the workspace root from the command if provided, otherwise use the shared one
        let workspace_root = if self.workspace_root != PathBuf::from(".") {
            self.workspace_root.clone()
        } else {
            shared.workspace_root.clone()
        };
        
        // Use the dry_run flag from the command if provided, otherwise use the shared one
        let dry_run = self.dry_run || shared.dry_run;
        
        // Use the skip_published flag from the command if provided, otherwise use the shared one
        let skip_published = self.skip_published || shared.skip_published;
        
        // Use the token from the command if provided, otherwise use the shared one
        let token = self.token.as_ref().or(shared.token.as_ref());
        
        // Find and parse the workspace
        let workspace = crate::helpers::workspace::discover_workspace_packages(&workspace_root)?;
        
        // Perform topological sort to get the correct publish order
        let sorted_packages = crate::helpers::topo_sort::topological_sort(&workspace)?;
        
        // Filter packages that should be published
        let publishable_packages = crate::helpers::topo_sort::filter_publishable_packages(sorted_packages);
        
        if publishable_packages.is_empty() {
            println!("No packages to publish.");
            return Ok(());
        }
        
        println!("Found {} packages to publish:", publishable_packages.len());
        for package in &publishable_packages {
            println!("  - {} v{}", package.name, package.version);
        }
        
        if dry_run {
            println!("Running in dry-run mode. No packages will be published.");
        }
        
        // Publish the packages
        publish_packages(&publishable_packages, dry_run, skip_published, token.map(|s| s.as_str()))?;
        
        println!("All packages published successfully!");
        Ok(())
    }
}

/// Publishes a single package using cargo publish
pub fn publish_package(package: &CargoPackage, dry_run: bool, token: Option<&str>) -> Result<()> {
    info!("Publishing package: {} v{}", package.name, package.version);

    let mut cmd = Command::new("cargo");
    cmd.arg("publish");

    if dry_run {
        cmd.arg("--dry-run");
        info!("Running in dry-run mode for package: {}", package.name);
    }

    if let Some(token) = token {
        cmd.arg("--token");
        cmd.arg(token);
    }

    // Set the working directory to the package directory
    cmd.current_dir(&package.path);

    // Execute the command
    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("Successfully published {}: {}", package.name, stdout);
        Ok(())
    }
    else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!("Failed to publish {}: {}", package.name, stderr);
        error!("Output: {}", stdout);
        Err(CargoError::PublishError(format!("Failed to publish {}: {}", package.name, stderr)))
    }
}

/// Checks if a package is already published
pub fn is_package_published(package: &CargoPackage) -> Result<bool> {
    info!("Checking if package {} is already published", package.name);

    let mut cmd = Command::new("cargo");
    cmd.arg("search");
    cmd.arg(&package.name);
    cmd.arg("--limit");
    cmd.arg("1");

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Check if the package exists in the registry
        if stdout.contains(&package.name) {
            info!("Package {} is already published", package.name);
            Ok(true)
        }
        else {
            info!("Package {} is not published yet", package.name);
            Ok(false)
        }
    }
    else {
        // If search fails, assume the package is not published
        warn!("Failed to check if package {} is published, assuming it's not", package.name);
        Ok(false)
    }
}

/// Publishes packages in order, skipping already published ones
pub fn publish_packages(packages: &[CargoPackage], dry_run: bool, skip_published: bool, token: Option<&str>) -> Result<()> {
    for package in packages {
        if skip_published {
            match is_package_published(package) {
                Ok(true) => {
                    info!("Skipping already published package: {}", package.name);
                    continue;
                }
                Ok(false) => {
                    // Package is not published, continue with publishing
                }
                Err(e) => {
                    warn!("Failed to check if package {} is published: {}, proceeding with publish", package.name, e);
                }
            }
        }

        publish_package(package, dry_run, token)?;
    }

    Ok(())
}