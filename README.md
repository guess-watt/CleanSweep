# CleanSweep

CleanSweep is a fast, safe, beginner-friendly Windows CLI storage analyzer written in Rust. It helps you inspect disk usage, discover large files, and detect duplicates without modifying, moving, or deleting any files.

## Project Status: Phase 1 — Verified Starting Point

The project has been inspected and stabilized. The baseline single-directory scanner functions correctly and passes all format and linter checks.

## Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Rust 1.85+ / 2024 edition compatible)
- Windows (prioritized) with C++ build tools

## Quick Start

Run the scanner in development mode:

```powershell
cargo run
```

Run linter and formatting checks:

```powershell
cargo fmt --check
cargo clippy
cargo test
```

## Safety Notice

CleanSweep is strictly read-only: it will never move, delete, modify, or upload your files.
