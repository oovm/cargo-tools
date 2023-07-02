use crate::{commands::cmd_publish::PublishCommand, CargoError, CargoWorkspaceCommand, CommandOptions};
use clap::{Parser, Subcommand};

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
        match self.command.as_ref() {
            Some(cmds) => cmds.run(&self.options).await,
            None => self.show_workspace_info().await,
        }
    }

    pub async fn show_workspace_info(&self) -> Result<(), CargoError> {
        todo!()
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
