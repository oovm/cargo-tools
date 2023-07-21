//! `cargo-workspace` 命令行入口。

use clap::Parser;

use cargo_tools::{Cargo, diag};

async fn run() -> Result<(), cargo_tools::CargoError> {
    let Cargo::Workspace(cmd) = Cargo::parse();
    cmd.run().await
}

fn main() {
    diag::main(run);
}
