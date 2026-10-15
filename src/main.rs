use std::fs;
use std::path::Path;

/// Represents basic information about a scanned file.
struct FileInfo {
    name: String,
    size: u64,
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If no argument is provided or help is requested, show usage instructions.
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("CleanSweep - Fast & Safe CLI Storage Analyzer\n");
        println!("Usage: cleansweep <folder-path>\n");
        println!("Examples:");
        println!(r#"  cleansweep "C:\Users\anwin\Downloads""#);
        println!(r#"  cleansweep ."#);
        return;
    }

    let folder_path = &args[1];
    let path = Path::new(folder_path);

    // Validate that the path exists.
    if !path.exists() {
        eprintln!("Error: The path '{folder_path}' does not exist.");
        std::process::exit(1);
    }

    // Validate that the path is a directory, not a standalone file.
    if !path.is_dir() {
        eprintln!("Error: The path '{folder_path}' is not a directory.");
        std::process::exit(1);
    }

    println!("CleanSweep - Scanning: \"{folder_path}\"\n");

    // Read directory entries safely.
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Error: Cannot read directory '{folder_path}': {err}");
            std::process::exit(1);
        }
    };

    let mut files: Vec<FileInfo> = Vec::new();
    let mut skipped_symlinks: usize = 0;
    let mut skipped_errors: usize = 0;

    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                match entry.file_type() {
                    Ok(file_type) => {
                        // Avoid following symbolic links by default
                        if file_type.is_symlink() {
                            skipped_symlinks += 1;
                            continue;
                        }

                        // Collect only regular files
                        if file_type.is_file() {
                            match entry.metadata() {
                                Ok(metadata) => {
                                    let name = entry.file_name().to_string_lossy().into_owned();
                                    files.push(FileInfo {
                                        name,
                                        size: metadata.len(),
                                    });
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Warning: Could not read metadata for {:?}: {err}",
                                        entry.file_name()
                                    );
                                    skipped_errors += 1;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "Warning: Could not determine file type for {:?}: {err}",
                            entry.file_name()
                        );
                        skipped_errors += 1;
                    }
                }
            }
            Err(err) => {
                eprintln!("Warning: Error reading directory entry: {err}");
                skipped_errors += 1;
            }
        }
    }

    if files.is_empty() {
        println!("No regular files found in \"{folder_path}\".");
    } else {
        files.sort_by_key(|a| a.name.to_lowercase());

        let total_size: u64 = files.iter().map(|f| f.size).sum();

        println!("Files found ({}):", files.len());
        for file in &files {
            println!("{:>10}  {}", format_size(file.size), file.name);
        }

        println!("\nSummary:");
        println!("  Total regular files: {}", files.len());
        println!("  Total size:          {}", format_size(total_size));
    }

    if skipped_symlinks > 0 {
        println!("  Skipped symbolic links: {skipped_symlinks}");
    }
    if skipped_errors > 0 {
        println!("  Skipped due to errors:  {skipped_errors}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1023), "1023.00 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.00 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(2 * 1024 * 1024 * 1024 + 536_870_912), "2.50 GB");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }
}
