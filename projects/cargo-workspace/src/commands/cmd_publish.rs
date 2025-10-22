use crate::{
    CommandOptions,
    errors::{CargoError, Result},
    helpers::{checkpoint::PublishCheckpoint, workspace::CargoPackage},
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

    /// Use checkpoint to resume from where it left off
    #[arg(long)]
    pub resume: bool,

    /// Registry token for publishing
    #[arg(long)]
    pub token: Option<String>,

    /// Interval in seconds between publishing packages (default: 0)
    #[arg(long, default_value = "0")]
    pub publish_interval: u64,
}

impl PublishCommand {
    pub async fn run(&self, shared: &CommandOptions) -> std::result::Result<(), CargoError> {
        // Use the workspace root from the command if provided, otherwise use the shared one
        let workspace_root =
            if self.workspace_root != PathBuf::from(".") { self.workspace_root.clone() } else { shared.workspace_root.clone() };

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

        // Initialize or load checkpoint
        let mut checkpoint = if self.resume {
            match PublishCheckpoint::load(&workspace_root)? {
                Some(cp) => {
                    println!("Resuming from previous publish session.");
                    cp
                }
                None => {
                    println!("No previous publish session found, starting fresh.");
                    PublishCheckpoint::new(workspace_root.clone())
                }
            }
        } else {
            // Remove any existing checkpoint if not resuming
            PublishCheckpoint::remove(&workspace_root)?;
            PublishCheckpoint::new(workspace_root.clone())
        };

        // Filter packages based on checkpoint
        let packages_to_publish: Vec<&CargoPackage> = publishable_packages.iter()
            .filter(|p| !checkpoint.is_published(&p.name, &p.version))
            .collect();

        if packages_to_publish.is_empty() {
            println!("All packages have already been published.");
            return Ok(());
        }

        println!("Found {} packages to publish:", packages_to_publish.len());
        for package in &packages_to_publish {
            println!("  - {} v{}", package.name, package.version);
        }

        if dry_run {
            println!("Running in dry-run mode. No packages will be published.");
        }

        // Publish the packages with checkpoint support
        let result = publish_packages_with_checkpoint(
            &packages_to_publish,
            &mut checkpoint,
            dry_run,
            skip_published,
            token.map(|s| s.as_str()),
            self.publish_interval
        );

        match result {
            Ok(_) => {
                // All packages published successfully, remove checkpoint
                if !dry_run {
                    PublishCheckpoint::remove(&workspace_root)?;
                    println!("All packages published successfully!");
                } else {
                    println!("Dry-run completed successfully!");
                }
                Ok(())
            }
            Err(e) => {
                // Save checkpoint before returning error
                if !dry_run {
                    checkpoint.save()?;
                    println!("Publishing interrupted. Checkpoint saved. Use --resume to continue.");
                }
                Err(e)
            }
        }
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
        
        // Check if the error is due to the package already existing
        if stderr.contains("already exists on crates.io index") || 
           stderr.contains("crate version") && stderr.contains("is already uploaded") {
            info!("Package {} v{} already exists on crates.io, skipping", package.name, package.version);
            return Ok(());
        }
        
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

/// Publishes packages in order, skipping already published ones, with checkpoint support
pub fn publish_packages_with_checkpoint(
    packages: &[&CargoPackage],
    checkpoint: &mut PublishCheckpoint,
    dry_run: bool,
    skip_published: bool,
    token: Option<&str>,
    publish_interval: u64
) -> Result<()> {
    for (index, package) in packages.iter().enumerate() {
        if skip_published {
            match is_package_published(package) {
                Ok(true) => {
                    info!("Skipping already published package: {}", package.name);
                    // Mark as published in checkpoint even if we skipped it
                    checkpoint.mark_published(package.name.clone(), package.version.clone());
                    checkpoint.save()?;
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

        // Try to publish the package
        match publish_package(package, dry_run, token) {
            Ok(_) => {
                // Mark as published in checkpoint
                checkpoint.mark_published(package.name.clone(), package.version.clone());
                checkpoint.save()?;
                
                // If this is not the last package and not in dry-run mode, wait for the interval
                if index < packages.len() - 1 && !dry_run && publish_interval > 0 {
                    println!("Waiting {} seconds before publishing next package...", publish_interval);
                    std::thread::sleep(std::time::Duration::from_secs(publish_interval));
                }
            }
            Err(e) => {
                // Don't mark as published if there was an error
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Publishes packages in order, skipping already published ones (legacy function without checkpoint)
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