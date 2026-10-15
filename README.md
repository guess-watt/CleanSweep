# CleanSweep — Rust CLI Storage Analyzer for Windows

CleanSweep is a fast, safe, beginner-friendly Windows command-line utility built in Rust to help you analyze, understand, and optimize disk usage.

It allows you to recursively scan folders, find the largest space consumers, detect exact duplicate files using cryptographic SHA-256 hashing, break down storage by file type, analyze file age, and export structured cleanup reports.

---

## 🛡️ Safety Guarantees

CleanSweep is strictly **read-only**:
- **Never deletes, moves, or alters any file.**
- **Never uploads or sends your data anywhere.** File analysis happens entirely on your local machine.
- **Never reads file contents unnecessarily.** File sizes are retrieved instantly via filesystem metadata. Content is read strictly in streaming chunks during duplicate verification.
- **Protects against circular directory links:** Symbolic links are safely detected and skipped by default.
- **Tolerates permission errors:** Inaccessible folders and files are skipped gracefully without aborting the scan.

---

## ✨ Features

- **Recursive & Fast Folder Scanning**: Traverses folder trees iteratively to avoid call stack overflow.
- **Largest Files Finder**: Ranks files from largest to smallest with customizable limits (`-n`, `--limit`).
- **Duplicate File Detection**:
  - Groups files by exact byte size first (files with unique sizes are never hashed).
  - Uses streaming **SHA-256** to verify matches without loading entire files into memory.
  - Distinguishes original copies (`[Keep]`) from redundant copies (`[Copy]`).
  - Accurately calculates potential reclaimable space without double-counting.
- **Storage Usage by File Type**: Categorizes files by extension, showing file counts, total storage, and percentages.
- **File-Age Analysis**: Uses true filesystem modification timestamps (`--days`) to highlight inactive files.
- **Exportable Reports**: Exports complete analysis summaries to plain text or markdown files (`--output`), preventing accidental overwrites unless `--force` is specified.
- **Human-Readable Sizes**: Dynamically formats byte counts into `B`, `KB`, `MB`, `GB`, and `TB`.

---

## 📋 Prerequisites

- **Operating System**: Windows (prioritized) or cross-platform (Linux/macOS compatible).
- **Rust Toolchain**: Rust 1.85+ / 2024 edition compatible with Cargo.
- **C++ Build Tools**: Visual Studio C++ Build Tools (on Windows).

---

## 🚀 Installation & Building

### 1. Build in Development Mode

```powershell
cargo build
```

### 2. Build an Optimized Windows Release Executable

```powershell
cargo build --release
```

The compiled release binary is located at:
`target\release\cleansweep.exe`

You can run it directly:
```powershell
.\target\release\cleansweep.exe --help
```

---

## 📖 CLI Usage & Command Examples

### 1. Quick Storage Overview (`scan`)

Scan a folder and display an overview of total files, directories, storage used, and the top 10 largest files:

```powershell
cargo run -- scan "C:\Users\username\Downloads"
```

Include all subdirectories recursively:

```powershell
cargo run -- scan "C:\Users\username\Downloads" --recursive
```

### 2. Find the Largest Files (`largest`)

List the top 15 largest files recursively across a folder:

```powershell
cargo run -- largest "C:\Users\username\Downloads" --limit 15 -r
# Or using shorthand flags:
cargo run -- largest "C:\Users\username\Downloads" -n 15 -r
```

### 3. Detect Duplicate Files (`duplicates`)

Find duplicate files based on content hash (even if their filenames are completely different) and calculate reclaimable storage:

```powershell
cargo run -- duplicates "C:\Users\username\Downloads"
```

Scan duplicates recursively across all subfolders:

```powershell
cargo run -- duplicates "C:\Users\username\Downloads" -r
```

### 4. Storage Usage by File Type (`types`)

Summarize storage consumption categorized by file extension (PDF, ZIP, EXE, etc.):

```powershell
cargo run -- types "C:\Users\username\Downloads" -r
```

### 5. File-Age Analysis (`old`)

Identify files that have not been modified for at least 90 days:

```powershell
cargo run -- old "C:\Users\username\Downloads" --days 90
```

### 6. Export a Comprehensive Cleanup Report (`report`)

Generate and export a full analysis report to a file:

```powershell
cargo run -- report "C:\Users\username\Downloads" -r --output "reports\my-report.txt" -f
```

---

## 🧪 Automated Testing & Code Quality

CleanSweep includes 22 automated unit and integration tests covering path validation, recursive traversal, SHA-256 streaming, file-age filtering, extension grouping, and argument parsing.

Run the test suite:

```powershell
cargo test
```

Run code formatting checks:

```powershell
cargo fmt --check
```

Run linter checks:

```powershell
cargo clippy -- -D warnings
```

---

## ⚠️ Limitations & Notes

- **File Extensions**: File extensions are useful for categorization, but they do not guarantee the underlying MIME type or file format.
- **Read-Only by Design**: CleanSweep does not provide automatic file deletion. Any cleanup actions should be executed manually by the user after reviewing the report.
- **Permissions**: Files or directories protected by strict Windows administrative permissions (e.g. `System Volume Information`) will be skipped and tallied as skipped errors.
