#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, rustdoc::missing_crate_level_docs)]
#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/oovm/shape-rs/dev/projects/images/Trapezohedron.svg")]

pub mod commands;
mod errors;
pub mod helpers;

pub use crate::errors::{CargoError, Result};
use clap::{Args, Parser};
pub use commands::WorkspaceCommands;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "cargo-workspace")]
#[command(about = "A tool to publish Cargo workspace packages in dependency order")]
#[command(version)]
pub struct CargoWorkspaceCommand {
    #[command(flatten)]
    pub options: CommandOptions,

    #[command(subcommand)]
    pub command: Option<WorkspaceCommands>,
}

#[derive(Clone, Debug, Args)]
pub struct CommandOptions {
    /// The path to the workspace root directory
    #[arg(short, long, default_value = ".")]
    pub workspace_root: PathBuf,

    /// Run in dry-run mode without actually publishing
    #[arg(long)]
    dry_run: bool,

    /// Skip packages that are already published
    #[arg(long)]
    skip_published: bool,

    /// Registry token for publishing
    #[arg(long)]
    pub token: Option<String>,
}
