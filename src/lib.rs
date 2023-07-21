//! Cargo workspace 工具库：按依赖顺序列出并发布 workspace 成员。

/// 子命令实现。
pub mod commands;
/// CLI 追踪与 miette 错误报告。
pub mod diag;
mod errors;
/// workspace 发现、拓扑排序与检查点。
pub mod helpers;

pub use crate::errors::{CargoError, Result};
use clap::{Args, Parser};
pub use commands::WorkspaceCommands;
use std::path::PathBuf;

/// 顶层 `cargo` 子命令解析（`cargo workspace` / `cargo ws`）。
#[derive(Debug, Parser)]
#[command(name = "cargo-workspace", bin_name = "cargo")]
pub enum Cargo {
    /// Workspace publish utilities.
    #[clap(alias = "ws")]
    Workspace(CargoWorkspaceCommand),
}

/// `cargo workspace` 参数与子命令。
#[derive(Debug, Parser)]
#[command(name = "cargo-workspace")]
#[command(about = "Publish Cargo workspace packages in dependency order")]
#[command(version)]
pub struct CargoWorkspaceCommand {
    #[command(flatten)]
    pub options: CommandOptions,

    #[command(subcommand)]
    pub command: Option<WorkspaceCommands>,
}

/// 各子命令共享的 CLI 选项。
#[derive(Clone, Debug, Args)]
pub struct CommandOptions {
    /// Workspace root directory.
    #[arg(short, long, default_value = ".")]
    pub workspace_root: PathBuf,

    /// Print actions without publishing.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip crates that are already on the registry.
    #[arg(long)]
    pub skip_published: bool,

    /// Registry token for `cargo publish`.
    #[arg(long)]
    pub token: Option<String>,
}
