use crate::{CargoError, CommandOptions, WorkspaceCommands};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug,Parser)]
pub struct ListCommand {
    /// The path to the workspace root directory
    #[arg(short, long, default_value = ".")]
    pub workspace_root: PathBuf,
}

impl ListCommand {
    pub async fn run(&self, shared: &CommandOptions) -> Result<(), CargoError> {
        todo!()
    }
}
