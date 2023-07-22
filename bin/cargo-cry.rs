//! `cargo cry` 入口：多配置 `cargo check` 与仓库卫生扫描。

use cargo_tools::cry::commands::CryCli;
use clap::Parser;

fn main() {
    let cli = CryCli::parse();
    if let Err(err) = cli.run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
