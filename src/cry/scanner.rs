//! 仓库卫生扫描：manifest、测试位置与文档规范。

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use toml::Value;
use walkdir::WalkDir;

const MAX_RUST_LINES: usize = 1000;
/// 超过此行数的 `readme.md` 视为正式文档，须用 `include_str` 导入。
const MIN_README_LINES_FOR_INCLUDE_STR: usize = 10;

/// 一条扫描发现。
#[derive(Debug, Clone)]
pub struct Finding {
    pub path: PathBuf,
    pub line: Option<usize>,
    pub message: String,
}

/// 在 `root` 下发现全部 `Cargo.toml` 包目录（跳过 `target/`）。
pub fn discover_packages(root: &Path) -> Vec<PathBuf> {
    let mut packages = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && !name.starts_with(".cry-")
        })
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() && entry.file_name() == "Cargo.toml" {
            packages.push(entry.path().parent().expect("Cargo.toml parent").to_path_buf());
        }
    }
    packages.sort();
    packages.dedup();
    packages
}

/// 查找包含 `[workspace]` 的最近祖先目录。
pub fn find_workspace_root(start: &Path) -> PathBuf {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        let manifest = current.join("Cargo.toml");
        if manifest.is_file() {
            if let Ok(content) = fs::read_to_string(&manifest) {
                if let Ok(value) = toml::from_str::<Value>(&content) {
                    if value.get("workspace").is_some() {
                        return current;
                    }
                }
            }
        }
        if !current.pop() {
            break;
        }
    }
    start.canonicalize().unwrap_or_else(|_| start.to_path_buf())
}

/// 完整性检查：readme、manifest 继承、`missing_docs` 等。
pub fn scan_integrity(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(scan_readme_case(root));
    for package in discover_packages(root) {
        findings.extend(scan_package_integrity(&package));
    }
    findings
}

/// 文档规范：超过 `10` 行的 `src/readme.md` 须用 `include_str` 导入。
pub fn scan_doc_spec(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for package in discover_packages(root) {
        let lib_rs = package.join("src").join("lib.rs");
        let readme = package.join("src").join("readme.md");
        if !lib_rs.is_file() || !readme.is_file() {
            continue;
        }
        let readme_lines = count_non_empty_lines(&readme);
        if readme_lines <= MIN_README_LINES_FOR_INCLUDE_STR {
            continue;
        }
        let content = fs::read_to_string(&lib_rs).unwrap_or_default();
        if content.contains("#![doc = include_str!(\"readme.md\")]") {
            continue;
        }
        if content.contains("//!") {
            findings.push(Finding {
                path: lib_rs,
                line: None,
                message: format!(
                    "同目录 readme.md 已有 {readme_lines} 行（超过 {MIN_README_LINES_FOR_INCLUDE_STR} 行），应当使用 #![doc = include_str!(\"readme.md\")] 导入文档，禁止使用 //! 注释"
                ),
            });
        }
    }
    findings
}

/// 测试位置与包根目录文件卫生（含脚本与游离 `.rs`）。
pub fn scan_misplaced(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for package in discover_packages(root) {
        findings.extend(scan_misplaced_tests(&package));
        findings.extend(scan_misplaced_root_files(&package));
    }
    findings
}

/// 超过 `1000` 行的 Rust 源文件。
pub fn scan_large_files(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && !name.starts_with(".cry-")
        })
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let lines = content.lines().count();
        if lines > MAX_RUST_LINES {
            findings.push(Finding {
                path: entry.path().to_path_buf(),
                line: None,
                message: format!("文件过大: {lines} 行 (建议拆分模块)"),
            });
        }
    }
    findings
}

fn scan_readme_case(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && !name.starts_with(".cry-")
        })
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() == "README.md" {
            findings.push(Finding {
                path: entry.path().to_path_buf(),
                line: None,
                message: "'README.md' 文件名应当全小写为 'readme.md'".into(),
            });
        }
    }
    findings
}

fn scan_package_integrity(package: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let manifest_path = package.join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path).unwrap_or_default();
    let value = toml::from_str::<Value>(&content).unwrap_or(Value::Table(toml::map::Map::new()));

    if value.get("package").is_none() {
        findings.push(Finding { path: manifest_path.clone(), line: None, message: "缺少 [package] 部分".into() });
        return findings;
    }

    if !package.join("readme.md").is_file() {
        findings.push(Finding { path: package.to_path_buf(), line: None, message: "缺少 readme.md 文件".into() });
    }

    let lib_rs = package.join("src").join("lib.rs");
    if lib_rs.is_file() {
        let lib = fs::read_to_string(&lib_rs).unwrap_or_default();
        if let Some(message) = missing_docs_finding(&manifest_path, &value, &lib) {
            findings.push(Finding { path: lib_rs, line: None, message });
        }
    }

    let workspace_root = find_workspace_root(package);
    let workspace_manifest = workspace_root.join("Cargo.toml");
    let workspace_value = fs::read_to_string(&workspace_manifest).ok().and_then(|text| toml::from_str::<Value>(&text).ok());

    let is_member = package != workspace_root;
    if let Some(ws) = workspace_value {
        if is_member {
            findings.extend(scan_workspace_member_manifest(&manifest_path, &value, &ws));
        }
    }

    findings
}

fn scan_workspace_member_manifest(manifest_path: &Path, package: &Value, workspace: &Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    let pkg = package.get("package").and_then(Value::as_table);
    let ws_pkg = workspace.get("workspace").and_then(|w| w.get("package")).and_then(Value::as_table);
    let ws_deps = workspace.get("workspace").and_then(|w| w.get("dependencies")).and_then(Value::as_table);

    if let Some(pkg) = pkg {
        for field in ["version", "edition", "authors", "license"] {
            if ws_pkg.is_some_and(|ws| ws.contains_key(field)) {
                if !uses_workspace_inherit(pkg, field) {
                    findings.push(Finding {
                        path: manifest_path.to_path_buf(),
                        line: None,
                        message: format!("package.{field} 应当使用 workspace = true (以便在根目录统一管理)"),
                    });
                }
            }
            else if (field == "authors" || field == "license") && !pkg.contains_key(field) {
                findings.push(Finding {
                    path: manifest_path.to_path_buf(),
                    line: None,
                    message: format!("缺少 package.{field} 字段"),
                });
            }
        }
    }

    for section in ["dependencies", "dev-dependencies"] {
        if let Some(deps) = package.get(section).and_then(Value::as_table) {
            if let Some(ws_deps) = ws_deps {
                for name in deps.keys() {
                    if ws_deps.contains_key(name) && !dep_uses_workspace(deps.get(name).unwrap()) {
                        findings.push(Finding {
                            path: manifest_path.to_path_buf(),
                            line: None,
                            message: format!("依赖 '{name}' 应当使用 workspace = true (以便在根目录统一管理依赖)"),
                        });
                    }
                }
            }
        }
    }

    findings
}

fn uses_workspace_inherit(pkg: &toml::map::Map<String, Value>, field: &str) -> bool {
    match pkg.get(field) {
        Some(Value::Table(table)) => table.get("workspace").and_then(Value::as_bool) == Some(true),
        _ => false,
    }
}

fn dep_uses_workspace(dep: &Value) -> bool {
    match dep {
        Value::Table(table) => table.get("workspace").and_then(Value::as_bool) == Some(true),
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MissingDocsLevel {
    Allow = 0,
    Warn = 1,
    Deny = 2,
    Forbid = 3,
}

fn missing_docs_finding(manifest_path: &Path, package: &Value, lib_rs: &str) -> Option<String> {
    let _ = manifest_path;
    let manifest_level = missing_docs_level_from_manifest(package);
    let lib_level = missing_docs_level_from_lib_rs(lib_rs);
    let effective = manifest_level.max(lib_level);

    if effective >= MissingDocsLevel::Warn {
        return None;
    }

    Some(
        "缺少 missing_docs 门禁 (至少设为 warn，建议 deny 或 forbid；可在 lib.rs 使用 #![deny(missing_docs)] 或在 Cargo.toml [lints.rust] 配置)"
            .into(),
    )
}

fn missing_docs_level_from_manifest(package: &Value) -> MissingDocsLevel {
    package
        .get("lints")
        .and_then(|l| l.get("rust"))
        .and_then(|r| r.get("missing_docs"))
        .and_then(Value::as_str)
        .map(parse_missing_docs_level)
        .unwrap_or(MissingDocsLevel::Allow)
}

fn missing_docs_level_from_lib_rs(lib_rs: &str) -> MissingDocsLevel {
    let mut level = MissingDocsLevel::Allow;
    for line in lib_rs.lines().take(40) {
        let trimmed = line.trim();
        if trimmed.contains("#![forbid(missing_docs)]") {
            return MissingDocsLevel::Forbid;
        }
        if trimmed.contains("#![deny(missing_docs)]") {
            level = level.max(MissingDocsLevel::Deny);
        }
        if trimmed.contains("#![warn(missing_docs)]") {
            level = level.max(MissingDocsLevel::Warn);
        }
    }
    level
}

fn parse_missing_docs_level(raw: &str) -> MissingDocsLevel {
    match raw.trim_matches('"') {
        "forbid" => MissingDocsLevel::Forbid,
        "deny" => MissingDocsLevel::Deny,
        "warn" => MissingDocsLevel::Warn,
        _ => MissingDocsLevel::Allow,
    }
}

fn count_non_empty_lines(path: &Path) -> usize {
    fs::read_to_string(path).map(|content| content.lines().count()).unwrap_or(0)
}

fn scan_misplaced_tests(package: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let src = package.join("src");
    if !src.is_dir() {
        return findings;
    }

    for entry in WalkDir::new(&src).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[test]") || trimmed.starts_with("mod tests") {
                findings.push(Finding {
                    path: entry.path().to_path_buf(),
                    line: Some(line_no + 1),
                    message: "测试位置不当 (请将测试移动到 tests/ 目录，tests/ 与 Cargo.toml 同级)".into(),
                });
            }
        }
    }
    findings
}

fn scan_misplaced_root_files(package: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let workspace_root = find_workspace_root(package);
    let allowed_dirs: HashSet<&str> =
        HashSet::from(["src", "tests", "benches", "examples", "target", ".git", ".github", "scripts", "documentation", "bin"]);

    let Ok(entries) = fs::read_dir(package)
    else {
        return findings;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if allowed_dirs.contains(name.as_str()) || name.starts_with('.') {
                continue;
            }
            continue;
        }

        if name == "Cargo.toml" || name == "readme.md" || name == "build.rs" {
            continue;
        }

        if name.ends_with(".rs") {
            findings.push(Finding {
                path: package.to_path_buf(),
                line: None,
                message: format!(
                    "发现随地大小便行为: Rust 文件 '{name}' 不应直接放在 Cargo.toml 同级目录，应当放在 Cargo.toml 目录的 tests/ 目录下"
                ),
            });
        }
    }

    let _ = workspace_root;
    findings
}

/// 将发现格式化为日志行（带 `\\?\` 长路径前缀）。
pub fn format_finding(finding: &Finding) -> String {
    let prefix = format!("\\\\?\\{}", finding.path.display());
    match finding.line {
        Some(line) => format!("{prefix}:{line}: {}", finding.message),
        None => format!("{prefix}: {}", finding.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_docs_accepts_warn_or_stricter() {
        let manifest = r#"
[package]
name = "demo"

[lints.rust]
missing_docs = "warn"
"#;
        let value = toml::from_str::<Value>(manifest).unwrap();
        assert!(missing_docs_finding(Path::new("Cargo.toml"), &value, "").is_none());

        let deny_manifest = manifest.replace("warn", "deny");
        let deny_value = toml::from_str::<Value>(&deny_manifest).unwrap();
        assert!(missing_docs_finding(Path::new("Cargo.toml"), &deny_value, "").is_none());
    }

    #[test]
    fn missing_docs_rejects_allow_or_absent() {
        let manifest = r#"
[package]
name = "demo"

[lints.rust]
missing_docs = "allow"
"#;
        let value = toml::from_str::<Value>(manifest).unwrap();
        assert!(missing_docs_finding(Path::new("Cargo.toml"), &value, "").is_some());

        let bare = r#"
[package]
name = "demo"
"#;
        let value = toml::from_str::<Value>(bare).unwrap();
        assert!(missing_docs_finding(Path::new("Cargo.toml"), &value, "").is_some());
    }

    #[test]
    fn missing_docs_lib_rs_attribute_counts() {
        let bare = r#"[package]
name = "demo"
"#;
        let value = toml::from_str::<Value>(bare).unwrap();
        let lib = "#![warn(missing_docs)]\npub fn demo() {}\n";
        assert!(missing_docs_finding(Path::new("Cargo.toml"), &value, lib).is_none());
    }

    #[test]
    fn short_readme_skips_include_str_rule() {
        let temp = std::env::temp_dir().join(format!("cargo-cry-readme-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("src")).unwrap();
        fs::write(temp.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n").unwrap();
        fs::write(temp.join("src/readme.md"), "short\n".repeat(8)).unwrap();
        fs::write(temp.join("src/lib.rs"), "//! inline docs\n").unwrap();

        let findings = scan_doc_spec(&temp);
        assert!(findings.is_empty());

        fs::write(temp.join("src/readme.md"), "line\n".repeat(12)).unwrap();
        let findings = scan_doc_spec(&temp);
        assert_eq!(findings.len(), 1);

        let _ = fs::remove_dir_all(&temp);
    }
}
