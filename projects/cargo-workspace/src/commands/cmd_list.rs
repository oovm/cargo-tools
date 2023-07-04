use crate::{CargoError, CommandOptions};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug,Parser)]
pub struct ListCommand {
    /// The path to the workspace root directory
    #[arg(short, long, default_value = ".")]
    pub workspace_root: PathBuf,
}

impl ListCommand {
    pub async fn run(&self, shared: &CommandOptions) -> Result<(), CargoError> {
        // Use the workspace root from the command if provided, otherwise use the shared one
        let workspace_root = if self.workspace_root != PathBuf::from(".") {
            self.workspace_root.clone()
        } else {
            shared.workspace_root.clone()
        };
        
        // Find and parse the workspace
        let workspace = crate::helpers::workspace::discover_workspace_packages(&workspace_root)?;
        
        // Perform topological sort to get the correct publish order
        let sorted_packages = crate::helpers::topo_sort::topological_sort(&workspace)?;
        
        // Filter packages that should be published
        let publishable_packages = crate::helpers::topo_sort::filter_publishable_packages(sorted_packages);
        
        if publishable_packages.is_empty() {
            println!("No packages to publish in this workspace.");
            return Ok(());
        }
        
        println!("Packages in publish order:");
        for (i, package) in publishable_packages.iter().enumerate() {
            println!("{}. {} v{}", i + 1, package.name, package.version);
            
            if !package.dependencies.is_empty() {
                let workspace_deps: Vec<String> = package.dependencies.iter()
                    .filter(|dep| publishable_packages.iter().any(|p| &p.name == dep))
                    .cloned()
                    .collect();
                    
                if !workspace_deps.is_empty() {
                    println!("   Dependencies: {}", workspace_deps.join(", "));
                }
            }
        }
        
        Ok(())
    }
}