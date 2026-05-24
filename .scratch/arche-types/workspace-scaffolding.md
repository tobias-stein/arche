---
title: Arche workspace & CI scaffolding
status: completed
---

## Parent

arche-types shared library (see `.scratch/arche-types/prd.md`)

## What to build

Set up the Cargo workspace with three crate members: `arche-types` (shared library), `arche-service` (HTTP server), and `arche-cli` (CLI binary). Each crate is a workspace member with its own `Cargo.toml`. Configure shared dependencies (serde, uuid, chrono, thiserror) in the workspace `Cargo.toml` so versions are consistent.

Set up CI configuration (GitHub Actions or equivalent): run `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check` on every push/PR to main. Add a root `rust-toolchain.toml` pinning the Rust edition. Add a root `README.md` with project overview and build instructions.

The workspace compiles cleanly (`cargo build` passes) with no warnings.

## Acceptance criteria

- [ ] `cargo new` or equivalent created three crate directories under `crates/arche-types/`, `crates/arche-service/`, `crates/arche-cli/`
- [ ] Root `Cargo.toml` defines a `[workspace]` with all three members
- [ ] Shared deps (serde, uuid, chrono, thiserror) listed in workspace `[workspace.dependencies]`
- [ ] `rust-toolchain.toml` present at root
- [ ] CI config runs `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] `cargo build` passes with zero warnings
- [ ] `README.md` with build/run instructions

## Blocked by

None - can start immediately
