use crate::{commands::cmd_publish::PublishCommand, CargoError, CargoWorkspaceCommand, CommandOptions};
use clap::Subcommand;

mod cmd_list;
mod cmd_publish;

pub use self::{
    cmd_list::ListCommand,
    cmd_publish::{is_package_published, publish_package, publish_packages},
};

#[derive(Debug, Subcommand)]
pub enum WorkspaceCommands {
    /// List all packages in the workspace in publish order
    List(ListCommand),
    /// Publish all packages in the workspace
    Publish(PublishCommand),
}

impl CargoWorkspaceCommand {
    pub async fn run(&self) -> Result<(), CargoError> {
        let options = self.options.clone();
        tracing::trace!("raw options: {}", options.workspace_root.display());
        // TODO: 写入一些配置
        tracing::trace!("final options: {}", options.workspace_root.display());
        match self.command.as_ref() {
            Some(cmds) => cmds.run(&options).await,
            None => self.show_workspace_info().await,
        }
    }

    pub async fn show_workspace_info(&self) -> Result<(), CargoError> {
        // Find and parse the workspace
        let workspace = crate::helpers::workspace::discover_workspace_packages(&self.options.workspace_root)?;
        
        // Perform topological sort to get the correct publish order
        let sorted_packages = crate::helpers::topo_sort::topological_sort(&workspace)?;
        
        // Filter packages that should be published
        let publishable_packages = crate::helpers::topo_sort::filter_publishable_packages(sorted_packages);
        
        println!("Cargo Workspace Information");
        println!("=========================");
        println!("Workspace Root: {}", workspace.root.display());
        println!("Total Packages: {}", workspace.packages.len());
        println!("Publishable Packages: {}", publishable_packages.len());
        
        if !publishable_packages.is_empty() {
            println!("\nPackages in publish order:");
            for (i, package) in publishable_packages.iter().enumerate() {
                println!("{}. {} v{}", i + 1, package.name, package.version);
            }
        }
        
        println!("\nUse 'cargo workspace list' for detailed package information");
        println!("Use 'cargo workspace publish' to publish all packages");
        
        Ok(())
    }
}

impl WorkspaceCommands {
    pub async fn run(&self, shared: &CommandOptions) -> Result<(), CargoError> {
        match self {
            WorkspaceCommands::List(cmd) => cmd.run(shared).await,
            WorkspaceCommands::Publish(cmd) => cmd.run(shared).await,
        }
    }
}