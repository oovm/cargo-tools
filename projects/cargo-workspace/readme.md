# cargo-workspace

A command-line tool for publishing Cargo workspace projects in dependency order.

## Features

- Automatically discover all projects in a workspace
- Topologically sort projects based on dependency relationships
- Publish projects in the correct order
- Support for dry-run mode
- Support for skipping already published projects
- Detect circular dependencies

## Installation

```bash
cargo install cargo-workspace
```

## Usage

### List all projects in the workspace

```bash
cargo workspace-publish list
```

### Publish all projects in the workspace

```bash
cargo workspace-publish publish
```

### Use a specific workspace root directory

```bash
cargo workspace-publish --workspace-root /path/to/workspace publish
```

### Dry-run mode (without actually publishing)

```bash
cargo workspace-publish publish --dry-run
```

### Skip already published projects

```bash
cargo workspace-publish publish --skip-published
```

### Use a publish token

```bash
cargo workspace-publish publish --token your_token
```

## How It Works

1. The tool first looks for the `Cargo.toml` file in the current directory or specified directory to determine if it's a workspace root directory
2. Parses the workspace configuration to discover all member projects
3. Parses each project's `Cargo.toml` file to extract dependency relationships
4. Uses a topological sorting algorithm to determine the correct publish order
5. Publishes each project in order, ensuring that dependent projects are published before the projects that depend on them

## Example

Assume you have a workspace with the following projects:

```
workspace/
├── Cargo.toml
├── utils/
│   └── Cargo.toml
├── core/
│   └── Cargo.toml
└── app/
    └── Cargo.toml
```

Where `core` depends on `utils`, and `app` depends on `core`.

Running `cargo workspace-publish list` will output:

```
Packages in publish order:
  utils v0.1.0 (/path/to/workspace/utils)
  core v0.1.0 (/path/to/workspace/core)
  app v0.1.0 (/path/to/workspace/app)
```

## License

This project is licensed under the MPL-2.0 license.

## Design Philosophy

This tool directly runs `cargo publish` without any special processing. While this approach may not be the fastest, it provides the strongest compatibility with the Cargo ecosystem.

## FAQ

### Does it support proxy?

Yes, as long as the `cargo publish` command itself supports proxies, this tool will work with them.

### Does it support sparse registry?

Yes, as long as the `cargo publish` command itself supports sparse registries, this tool will work with them.