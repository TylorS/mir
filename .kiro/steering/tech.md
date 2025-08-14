# Technology Stack

## Build System & Language

- **Primary Language**: Rust
- **Build System**: Cargo workspace with multiple crates
- **Package Manager**: Cargo with workspace dependencies

## Core Dependencies

- **Serialization**: `serde` with derive features for JSON serialization
- **Hashing**: `xxhash-rust` with xxh3 features for content-addressable storage
- **JSON Processing**: `serde_json` for IR serialization

## Architecture

- **Workspace Structure**: Multi-crate Cargo workspace
- **Target Platforms**: WASM, JavaScript/TypeScript, WASI
- **IR Format**: JSON-serializable intermediate representation
- **Storage**: Content-addressable with SHA-256 hashing

## Backend Targets

- **WASM**: WebAssembly compilation with optional NAPI bindings for Node.js
- **JavaScript/TypeScript**: Direct compilation to JS/TS with WASM integration
- **WASI**: WebAssembly System Interface with component model support

## Common Commands

```bash
# Build the entire workspace
cargo build

# Build in release mode
cargo build --release

# Run tests across all crates
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Build specific crate
cargo build -p <crate-name>

# Run tests for specific crate
cargo test -p <crate-name>
```

## Development Workflow

- Use `cargo check` for fast feedback during development
- Run `cargo clippy` before commits for linting
- Use `cargo fmt` to maintain consistent code style
- Test individual crates with `-p` flag for faster iteration