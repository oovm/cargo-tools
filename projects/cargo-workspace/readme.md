# cargo-workspace

A command-line tool for publishing Cargo workspace projects in dependency order.

### Resume from a previous publish session

```bash
cargo workspace publish --resume
```

If a publish operation is interrupted (e.g., due to network issues), you can resume from where it left off using the `--resume` flag. The tool saves a checkpoint file in `target/cargo-workspace-publish.toml` after each successful package publication.

## Features

- Automatically discover all projects in a workspace
- Topologically sort projects based on dependency relationships
- Publish projects in the correct order
- Support for dry-run mode
- Support for skipping already published projects
- Detect circular dependencies
- Support for glob patterns in workspace members
- Handle workspace inheritance for package versions
- Resume from interrupted publish sessions with checkpoint mechanism

## Installation

```bash
cargo install cargo-workspace-v2
```

Or build from source:

```bash
git clone https://github.com/oovm/cargo-tools
cd cargo-tools
cargo install --path projects/cargo-workspace
```

## Usage

### Show workspace information

```bash
cargo workspace
```

This command displays information about the workspace, including:
- The workspace root directory
- Total number of packages
- List of discovered packages with their versions and paths
- Number of publishable packages
- Packages in publish order

### List all projects in the workspace

```bash
cargo workspace list
```

This command lists all packages in the workspace in the correct publish order, showing:
- Package name and version
- Workspace dependencies for each package

### Publish all projects in the workspace

```bash
cargo workspace publish
```

This command publishes all packages in the workspace in the correct dependency order.

### Use a specific workspace root directory

```bash
cargo workspace --workspace-root /path/to/workspace publish
```

This allows you to specify a different workspace root directory than the current directory.

### Dry-run mode (without actually publishing)

```bash
cargo workspace publish --dry-run
```

This shows what would be published without actually publishing anything.

### Skip already published projects

```bash
cargo workspace publish --skip-published
```

This checks if each package is already published and skips those that are.

### Use a publish token

```bash
cargo workspace publish --token your_token
```

This provides a registry token for publishing packages.

### Resume from a previous publish session

```bash
cargo workspace publish --resume
```

This resumes a publish operation that was interrupted, using the checkpoint file saved in `target/cargo-workspace-publish.toml`.

### Set interval between package publications

```bash
cargo workspace publish --publish-interval 10
```

This sets a 10-second interval between publishing each package to avoid triggering rate limits on crates.io. The default interval is 5 seconds.

## How It Works

1. The tool first looks for the `Cargo.toml` file in the current directory or specified directory to determine if it's a workspace root directory
2. Parses the workspace configuration to discover all member projects, supporting glob patterns
3. Parses each project's `Cargo.toml` file to extract dependency relationships
4. Handles workspace inheritance for package versions
5. Uses a topological sorting algorithm to determine the correct publish order
6. Publishes each project in order, ensuring that dependent projects are published before the projects that depend on them

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

Running `cargo workspace list` will output:

```
Packages in publish order:
1. utils v0.1.0
   Dependencies: 
2. core v0.1.0
   Dependencies: utils
3. app v0.1.0
   Dependencies: core
```

## Advanced Usage

### Workspace with Glob Patterns

The tool supports glob patterns in workspace members. For example, if your `Cargo.toml` contains:

```toml
[workspace]
members = [
    "core/*",
    "libs/*/lib",
    "examples/*",
]
```

The tool will correctly expand these patterns and discover all matching packages.

### Workspace Inheritance

The tool supports workspace inheritance for package versions. If a package has:

```toml
[package]
version.workspace = true
```

The tool will use the version defined in the workspace package section.

### Filtering Non-Publishable Packages

Packages with `publish = false` in their `Cargo.toml` will be automatically filtered out from the publish process.

## Error Handling

The tool provides detailed error messages for common issues:

- **Circular Dependencies**: Detects and reports circular dependencies between packages
- **Missing Workspace**: Reports when no workspace is found in the specified directory
- **Invalid TOML**: Reports parsing errors in `Cargo.toml` files

## License

This project is licensed under the MPL-2.0 license.

## Design Philosophy

This tool directly runs `cargo publish` without any special processing. While this approach may not be the fastest, it provides the strongest compatibility with the Cargo ecosystem.

## FAQ

### Does it support proxy?

Yes, as long as the `cargo publish` command itself supports proxies, this tool will work with them.

### Does it support sparse registry?

Yes, as long as the `cargo publish` command itself supports sparse registries, this tool will work with them.

### Can I use it with private registries?

Yes, the tool passes through any registry configuration to the underlying `cargo publish` command.

### How does it handle version conflicts?

The tool doesn't modify versions - it publishes packages with their existing versions. If there are version conflicts, they will be reported by the `cargo publish` command.

### Can I publish only specific packages?

Currently, the tool publishes all packages in the workspace that have `publish = true`. Selective publishing is not yet supported.