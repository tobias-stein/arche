# Arche

A blueprint-based random name generation system.

## Project Structure

```
arche/
├── crates/
│   ├── arche-types/    # Shared domain types, validation, and API contracts
│   ├── arche-service/  # HTTP REST API server
│   └── arche-cli/      # Command-line interface
├── Cargo.toml          # Workspace manifest
└── rust-toolchain.toml # Rust toolchain pinning
```

## Build & Run

### Prerequisites

- Rust 1.85+ (see `rust-toolchain.toml`)

### Build all crates

```bash
cargo build --workspace
```

### Run tests

```bash
cargo test --workspace
```

### Run lints

```bash
cargo clippy --workspace
cargo fmt --check
```

### Run individual crates

```bash
cargo run -p arche-service
cargo run -p arche-cli
```
