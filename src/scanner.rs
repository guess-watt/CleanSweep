use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Basic information about a scanned file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

/// Options controlling directory traversal.
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub recursive: bool,
}

/// Summary statistics collected during a scan.
#[derive(Debug, Clone, Default)]
pub struct ScanStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size: u64,
    pub skipped_symlinks: usize,
    pub skipped_errors: usize,
}

/// Aggregated result of a directory scan.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub root_path: PathBuf,
    pub files: Vec<FileInfo>,
    pub stats: ScanStats,
}

/// Converts a byte count into a human-readable string (B, KB, MB, GB, TB).
pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * KB;
    const GB: f64 = 1024.0 * MB;
    const TB: f64 = 1024.0 * GB;

    let b = bytes as f64;

    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        format!("{:.2} KB", b / KB)
    } else if b < GB {
        format!("{:.2} MB", b / MB)
    } else if b < TB {
        format!("{:.2} GB", b / GB)
    } else {
        format!("{:.2} TB", b / TB)
    }
}

/// Scans a directory safely, either single-level or recursively.
///
/// Safety guarantees:
/// - Never follows symbolic links (avoids loops and directory escapes).
/// - Continues scanning on individual entry or subfolder read errors.
/// - Does not read file contents into memory.
pub fn scan_directory(root: &Path, options: &ScanOptions) -> Result<ScanResult, std::io::Error> {
    if !root.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path '{}' does not exist", root.display()),
        ));
    }
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path '{}' is not a directory", root.display()),
        ));
    }

    let mut files = Vec::new();
    let mut stats = ScanStats::default();

    // Work queue for directories to inspect (iterative to avoid call stack overflow)
    let mut dirs_to_visit: Vec<PathBuf> = vec![root.to_path_buf()];

    while let Some(current_dir) = dirs_to_visit.pop() {
        stats.total_dirs += 1;

        let entries = match fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!(
                    "Warning: Could not read directory '{}': {err}",
                    current_dir.display()
                );
                stats.skipped_errors += 1;
                continue;
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Warning: Failed reading directory entry: {err}");
                    stats.skipped_errors += 1;
                    continue;
                }
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(err) => {
                    eprintln!(
                        "Warning: Could not determine file type for '{}': {err}",
                        entry.path().display()
                    );
                    stats.skipped_errors += 1;
                    continue;
                }
            };

            // Rule: Never follow symbolic links by default to prevent cycles and escapes
            if file_type.is_symlink() {
                stats.skipped_symlinks += 1;
                continue;
            }

            if file_type.is_dir() {
                if options.recursive {
                    dirs_to_visit.push(entry.path());
                }
            } else if file_type.is_file() {
                match entry.metadata() {
                    Ok(metadata) => {
                        let size = metadata.len();
                        let modified = metadata.modified().ok();
                        let name = entry.file_name().to_string_lossy().into_owned();

                        stats.total_files += 1;
                        stats.total_size += size;

                        files.push(FileInfo {
                            name,
                            path: entry.path(),
                            size,
                            modified,
                        });
                    }
                    Err(err) => {
                        eprintln!(
                            "Warning: Could not inspect metadata for '{}': {err}",
                            entry.path().display()
                        );
                        stats.skipped_errors += 1;
                    }
                }
            }
        }
    }

    Ok(ScanResult {
        root_path: root.to_path_buf(),
        files,
        stats,
    })
}

/// Returns up to `limit` files sorted from largest to smallest.
pub fn get_largest_files(mut files: Vec<FileInfo>, limit: usize) -> Vec<FileInfo> {
    files.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
    files.truncate(limit);
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn test_get_largest_files() {
        let sample = vec![
            FileInfo {
                name: "small.txt".into(),
                path: PathBuf::from("small.txt"),
                size: 10,
                modified: None,
            },
            FileInfo {
                name: "huge.zip".into(),
                path: PathBuf::from("huge.zip"),
                size: 10_000,
                modified: None,
            },
            FileInfo {
                name: "medium.mp4".into(),
                path: PathBuf::from("medium.mp4"),
                size: 500,
                modified: None,
            },
        ];

        let top = get_largest_files(sample, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].name, "huge.zip");
        assert_eq!(top[1].name, "medium.mp4");
    }

    #[test]
    fn test_scan_directory_non_existent() {
        let options = ScanOptions::default();
        let result = scan_directory(Path::new("non_existent_folder_xyz123"), &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_directory_current_dir() {
        let options = ScanOptions { recursive: false };
        let result = scan_directory(Path::new("."), &options).expect("Should scan current dir");
        assert!(result.stats.total_files > 0);
        assert!(result.stats.total_size > 0);
    }
}
