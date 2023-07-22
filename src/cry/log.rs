//! `cargo-cry.log` 写入与 `--split` 拆分。

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// 收集检查日志并在结束时落盘。
pub struct CryLogger {
    root: PathBuf,
    split: Option<usize>,
    lines: Vec<String>,
}

impl CryLogger {
    /// 在 `root` 下写入 `cargo-cry.log`（或拆分后的 `cargo-cry.N.log`）。
    pub fn new(root: impl AsRef<Path>, split: Option<usize>) -> Self {
        Self { root: root.as_ref().to_path_buf(), split, lines: Vec::new() }
    }

    /// 追加一行日志。
    pub fn push(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// 将缓冲内容写入日志文件，并打印日志路径提示。
    pub fn flush(&self) {
        if self.lines.is_empty() {
            return;
        }

        if let Some(parts) = self.split.filter(|n| *n > 1) {
            let chunk = self.lines.len().div_ceil(parts);
            for (index, chunk_lines) in self.lines.chunks(chunk.max(1)).enumerate() {
                let path = self.root.join(format!("cargo-cry.{}.log", index + 1));
                write_lines(&path, chunk_lines);
                println!("📝 日志: {}", path.display());
            }
            return;
        }

        let path = self.root.join("cargo-cry.log");
        write_lines(&path, &self.lines);
        println!("📝 日志: {}", path.display());
    }
}

fn write_lines(path: &Path, lines: &[String]) {
    if let Ok(mut file) = fs::File::create(path) {
        for line in lines {
            let _ = writeln!(file, "{line}");
        }
    }
}
