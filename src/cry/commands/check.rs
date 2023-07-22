//! 多配置 `cargo check` 与 `cargo doc`。

use serde::Deserialize;
use std::{path::Path, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

/// 一次 `cargo check` 配置。
#[derive(Debug, Clone, Copy)]
pub struct CheckProfile {
    pub label: &'static str,
    pub emoji: &'static str,
    pub args: &'static [&'static str],
}

pub const CHECK_PROFILES: [CheckProfile; 3] = [
    CheckProfile { label: "无特性 (no-features)", emoji: "👻", args: &["--no-default-features"] },
    CheckProfile { label: "默认特性 (default)", emoji: "📦", args: &[] },
    CheckProfile { label: "全特性 (all-features)", emoji: "🌟", args: &["--all-features"] },
];

/// `cargo check` 诊断行。
#[derive(Debug, Clone)]
pub struct CompileFinding {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: Option<String>,
    message: Option<Diagnostic>,
}

#[derive(Debug, Deserialize)]
struct Diagnostic {
    message: String,
    spans: Vec<DiagnosticSpan>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticSpan {
    file_name: Option<String>,
    line_start: Option<u32>,
    column_start: Option<u32>,
}

/// 运行单个配置的 `cargo check`，返回编译错误列表。
pub async fn run_cargo_check(root: &Path, profile: &CheckProfile) -> Result<Vec<CompileFinding>, String> {
    let mut command = Command::new("cargo");
    command
        .arg("check")
        .args(profile.args)
        .arg("--message-format=json")
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = command.spawn().map_err(|err| err.to_string())?;
    let stdout = child.stdout.take().expect("piped stdout");
    let mut reader = BufReader::new(stdout).lines();
    let mut findings = Vec::new();

    while let Some(line) = reader.next_line().await.map_err(|err| err.to_string())? {
        if let Ok(message) = serde_json::from_str::<CargoMessage>(&line) {
            if message.reason.as_deref() != Some("compiler-message") {
                continue;
            }
            if let Some(diagnostic) = message.message {
                if let Some(span) = diagnostic.spans.first() {
                    if let Some(path) = span.file_name.clone() {
                        findings.push(CompileFinding {
                            path,
                            line: span.line_start.unwrap_or(1),
                            column: span.column_start.unwrap_or(1),
                            message: diagnostic.message,
                        });
                    }
                }
            }
        }
    }

    let status = child.wait().await.map_err(|err| err.to_string())?;
    if !status.success() && findings.is_empty() {
        return Err("cargo check failed".into());
    }
    Ok(findings)
}

/// 运行 `cargo doc --no-deps`。
pub async fn run_cargo_doc(root: &Path) -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["doc", "--no-deps"])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|err| err.to_string())?;

    if status.success() { Ok(()) } else { Err("cargo doc failed".into()) }
}

/// 格式化编译错误日志行。
pub fn format_compile_finding(finding: &CompileFinding) -> String {
    let prefix = format!("\\\\?\\{}", finding.path);
    format!("{prefix}:{}:{}: 编译错误: {}", finding.line, finding.column, finding.message)
}
