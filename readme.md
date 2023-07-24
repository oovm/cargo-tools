# 📦 cargo-tools

Rust utilities for **Cargo workspaces**, built as a single crate with a `cargo-workspace` CLI.

## 🧰 Tools

| Binary            | Purpose                                                       |
|-------------------|---------------------------------------------------------------|
| `cargo-workspace` | List and publish workspace members in dependency order        |
| `cargo-cry`       | Multi-profile `cargo check`, `cargo doc`, and repo hygiene    |

Install as `cargo` subcommands by placing binaries on `PATH` (`cargo-workspace`, `cargo-cry`).

## 🚀 Install

```bash
cargo install --git https://github.com/oovm/cargo-tools.git --bin cargo-workspace --bin cargo-cry
```

Track `dev` while pre-1.0:

```bash
cargo install --git https://github.com/oovm/cargo-tools.git --branch dev --bin cargo-workspace --bin cargo-cry
```

Download a prebuilt archive from [GitHub Releases](https://github.com/oovm/cargo-tools/releases) (`cargo-tools-<platform>.zip`).

## 📖 Usage

```bash
cargo workspace
cargo workspace list
cargo workspace publish --dry-run
cargo workspace publish --skip-published --resume
```

See [documentation/workspace.md](documentation/workspace.md) for full options, checkpoint/resume, and workspace glob patterns.

```bash
cargo cry
cargo cry check
cargo cry doc
cargo cry file-size
cargo cry misplaced-tests
```

`cargo cry` runs no-features / default / all-features `cargo check`, `cargo doc --no-deps`, and workspace hygiene scans (manifest inheritance, test placement, `missing_docs` at least `warn`, doc `include_str` when `src/readme.md` exceeds 10 lines). It does **not** scan JVM / JavaScript / PowerShell artifacts.

## 🛠️ Development

Requires the pinned toolchain in [`rust-toolchain.toml`](rust-toolchain.toml) (nightly + `rustfmt` + `clippy`).

```bash
git clone https://github.com/oovm/cargo-tools.git
cd cargo-tools
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test --release
RUST_LOG=info cargo run --bin cargo-workspace -- workspace list
```

CLI errors use **miette**; set `RUST_LOG=debug` for **tracing** on stderr.

## 📦 Layout

```text
cargo-tools/
  bin/cargo-workspace.rs
  bin/cargo-cry.rs
  src/
    commands/     # list, publish
    cry/          # cargo cry scans and multi-profile check
    helpers/      # workspace discovery, topo sort, checkpoint
    diag.rs       # tracing + miette
  documentation/
    workspace.md
```

## 🧪 CI

GitHub Actions runs `fmt`, `clippy`, `build`, `doc`, and `test` on Ubuntu, macOS, and Windows. Tag pushes publish `cargo-tools-<platform>.zip`.

## 📄 License

MPL-2.0
