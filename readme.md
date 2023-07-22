# 📦 cargo-tools

Rust utilities for **Cargo workspaces**, built as a single crate with a `cargo-workspace` CLI.

## 🧰 Tools

| Binary            | Purpose                                                       |
|-------------------|---------------------------------------------------------------|
| `cargo-workspace` | List and publish workspace members in dependency order        |

Install as a `cargo` subcommand by placing the binary on `PATH` as `cargo-workspace`, then invoke `cargo workspace …`.

## 🚀 Install

```bash
cargo install --git https://github.com/oovm/cargo-tools.git --bin cargo-workspace
```

Track `dev` while pre-1.0:

```bash
cargo install --git https://github.com/oovm/cargo-tools.git --branch dev --bin cargo-workspace
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
  src/
    commands/     # list, publish
    helpers/      # workspace discovery, topo sort, checkpoint
    diag.rs       # tracing + miette
  documentation/
    workspace.md
```

## 🧪 CI

GitHub Actions runs `fmt`, `clippy`, `build`, `doc`, and `test` on Ubuntu, macOS, and Windows. Tag pushes publish `cargo-tools-<platform>.zip`.

## 📄 License

MPL-2.0
