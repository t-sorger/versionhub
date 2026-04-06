# VersionHub

`versionhub` fetches package versions from multiple ecosystems and can optionally filter versions by a semver-like range.

It works both as:

- a CLI tool
- a Rust library crate

## Supported Ecosystems

- `go` (via proxy.golang.org)
- `maven` (via Maven Central metadata)
- `npm` (via registry.npmjs.org)
- `rust` (via crates.io sparse index)

## Input Format

Each package must use:

`ecosystem/package@version_range`

Where:

- `ecosystem` is one of the supported ecosystems.
- `package` is the ecosystem-specific package name.
- `@version_range` (optional) is the interested version range.

Examples:

- `go/github.com/opencontainers/runc`
- `go/github.com/opencontainers/runc@>=v1.3.1, <v1.4.2`
- `maven/org.apache.logging.log4j:log4j-core@>= 2.13.0, < 2.15.0`
- `npm/lodash@>=4.0.0, <5.0.0`
- `rust/serde@>=1.0.0, <2.0.0`

## CLI Usage

### Run with Cargo

```bash
cargo run -- --pkgs "go/github.com/opencontainers/runc@>=v1.3.1, <v1.4.2"
```

### Multiple packages

```bash
cargo run -- --pkgs \
  "go/github.com/opencontainers/runc@>=v1.3.1, <v1.4.2" \
  "maven/org.apache.logging.log4j:log4j-core@>= 2.13.0, < 2.15.0" \
  --output ./output.json \
  --log debug
```

### CLI options

- `-p`, `--pkgs`, `--pkg`, `--packages <PKG...>`: one or more package specs.
- `-c`, `--concurrency <N>`: number of concurrent requests (default: `2`).
- `-o`, `--output <PATH>`: write successful results as pretty JSON array to file.
- `-l`, `--log <LEVEL>`: log level for env_logger (`error`, `warn`, `info`, `debug`, `trace`; default: `warn`).

If `--output` is not provided, each successful result is printed as one JSON object on stdout.

## Output Shape

Each successful item has this structure:

```json
{
  "ecosystem": "npm",
  "name": "lodash",
  "versions": ["4.17.20", "4.17.21"]
}
```

With `--output`, the file contains a JSON array of these objects.

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
versionhub = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
```

### Simple API

```rust
use versionhub::get_package_versions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = get_package_versions("npm/lodash@>=4.0.0, <5.0.0").await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

### Reusing your own reqwest client

```rust
use reqwest::Client;
use versionhub::get_package_versions_with_client;
use versionhub::structs::Package;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let pkg: Package = "rust/serde@>=1.0.0, <2.0.0".parse()?;
    let result = get_package_versions_with_client(&client, pkg).await?;
    println!("{} -> {} versions", result.name, result.versions.len());
    Ok(())
}
```

## Version Range Notes

- Multiple conditions are comma-separated (logical AND), for example:
  - `>= 1.0.0, < 2.0.0`
- Supported operators:
  - `>=`, `<=`, `>`, `<`, `=`
- If no operator is provided, equality is assumed.
- Go-style versions with `v` prefix (for example `v1.2.3`) are normalized for comparison.

## Error Handling Behavior (CLI)

- Each package is processed independently.
- A failure in one package does not stop others.
- Errors are printed to stderr.
- Only successful results are printed/written.

Note: ecosystem tests call live registries, so they require network access and may vary if upstream registries change.
