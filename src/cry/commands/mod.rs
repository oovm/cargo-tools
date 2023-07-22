//! `cargo cry` 子命令编排。

mod check;

use crate::cry::{
    log::CryLogger,
    scanner::{self, Finding},
};
use check::{CHECK_PROFILES, format_compile_finding, run_cargo_check, run_cargo_doc};
use clap::{Parser, Subcommand};
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::runtime::Runtime;

/// 顶层 `cargo` 解析（`cargo cry`）；供集成测试或外部包装使用。
#[derive(Debug, Parser)]
#[command(name = "cargo")]
pub enum CargoCryEntry {
    /// 多配置检查与仓库卫生扫描。
    Cry(CryCli),
}

/// `cargo cry` 参数与子命令。
#[derive(Debug, Parser)]
#[command(name = "cry", about = "Multi-profile cargo check and workspace hygiene scans")]
pub struct CryCli {
    /// 将日志拆分为 N 个文件 (`cargo-cry.1.log`, …)。
    #[arg(short, long)]
    pub split: Option<usize>,

    #[command(subcommand)]
    pub command: Option<CryCommand>,
}

/// 子命令；缺省时等价于 `all`。
#[derive(Debug, Subcommand)]
pub enum CryCommand {
    /// 运行常用检查（多配置编译、文档、完整性、测试位置）。
    All,
    /// 运行多配置 `cargo check`。
    Check,
    /// 检查超过 1000 行的 Rust 源文件。
    FileSize,
    /// 检查测试位置与包根目录卫生。
    MisplacedTests,
    /// 运行 `cargo doc --no-deps`。
    Doc,
}

impl CryCli {
    /// 执行子命令并在 `root` 下写入日志。
    pub fn run(self) -> Result<(), String> {
        let root = workspace_root();
        let mut logger = CryLogger::new(&root, self.split);
        let command = self.command.unwrap_or(CryCommand::All);
        let runtime = Runtime::new().map_err(|err| err.to_string())?;

        println!();
        println!("🚀 正在 {} 启动多配置检查", root.display());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let exit_code = match command {
            CryCommand::All => run_all(&root, &mut logger, &runtime)?,
            CryCommand::Check => run_check_profiles(&root, &mut logger, &runtime)?,
            CryCommand::FileSize => {
                run_labeled_scan("📏", "大文件", scan_and_log(&root, scanner::scan_large_files, &mut logger))?
            }
            CryCommand::MisplacedTests => {
                run_labeled_scan("🧪", "测试位置", scan_and_log(&root, scanner::scan_misplaced, &mut logger))?
            }
            CryCommand::Doc => run_doc(&root, &mut logger, &runtime)?,
        };

        logger.flush();

        if exit_code != 0 {
            return Err(format!("cargo cry finished with {exit_code} issue(s)"));
        }
        Ok(())
    }
}

fn workspace_root() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn run_all(root: &Path, logger: &mut CryLogger, runtime: &Runtime) -> Result<i32, String> {
    let mut issues = 0;
    issues += run_check_profiles(root, logger, runtime)?;
    issues += run_doc(root, logger, runtime)?;
    issues += run_labeled_scan("🛠️", "完整性", scan_and_log(root, scanner::scan_integrity, logger))?;
    issues += run_labeled_scan("🧪", "测试位置", scan_and_log(root, scanner::scan_misplaced, logger))?;
    issues += run_labeled_scan("📄", "文档规范", scan_and_log(root, scanner::scan_doc_spec, logger))?;
    Ok(issues)
}

fn run_check_profiles(root: &Path, logger: &mut CryLogger, runtime: &Runtime) -> Result<i32, String> {
    let mut issues = 0;
    for profile in &CHECK_PROFILES {
        print!("{} Checking {:<32}", profile.emoji, profile.label);
        let findings = runtime.block_on(run_cargo_check(root, profile))?;
        if findings.is_empty() {
            println!("... ✅ 通过");
        }
        else {
            println!("... ❌ 发现 {:>2} 个问题", findings.len());
            for finding in &findings {
                let line = format_compile_finding(finding);
                logger.push(&line);
            }
            issues += findings.len() as i32;
        }
    }
    Ok(issues)
}

fn run_doc(root: &Path, logger: &mut CryLogger, runtime: &Runtime) -> Result<i32, String> {
    print!("📚 正在检查文档...");
    match runtime.block_on(run_cargo_doc(root)) {
        Ok(()) => {
            println!(" ✅ 通过");
            Ok(0)
        }
        Err(err) => {
            println!(" ❌ 失败");
            logger.push(format!("cargo doc: {err}"));
            Ok(1)
        }
    }
}

fn run_labeled_scan(emoji: &str, label: &str, count: usize) -> Result<i32, String> {
    if count == 0 {
        println!("{emoji} 正在检查{label}... ✅ 通过");
        Ok(0)
    }
    else {
        println!("{emoji} 正在检查{label}... ❌ 发现 {count:>2} 个问题");
        Ok(count as i32)
    }
}

fn scan_and_log(root: &Path, scan: fn(&Path) -> Vec<Finding>, logger: &mut CryLogger) -> usize {
    let findings = scan(root);
    for finding in &findings {
        logger.push(scanner::format_finding(finding));
    }
    findings.len()
}
