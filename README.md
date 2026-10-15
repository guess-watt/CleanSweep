# CleanSweep

CleanSweep is a fast, safe, beginner-friendly Windows CLI storage analyzer written in Rust. It helps you inspect disk usage, discover large files, and detect duplicates without modifying, moving, or deleting any files.

## Project Status: Phase 2 — Proper Folder Scanning

CleanSweep accepts any user-supplied folder path, validates its existence and directory status, lists regular files with formatted sizes (B, KB, MB, GB, TB), safely skips symbolic links by default, and gracefully handles errors.

## Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Rust 1.85+ / 2024 edition compatible)
- Windows (prioritized) with C++ build tools

## Usage

Scan a specific folder:

```powershell
cargo run -- "C:\Users\anwin\Downloads"
```

Scan the current working directory:

```powershell
cargo run -- .
```

View help and usage information:

```powershell
cargo run -- --help
```

Run linter, formatting checks, and automated tests:

```powershell
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Safety Notice

CleanSweep is strictly read-only: it will never move, delete, modify, or upload your files.
