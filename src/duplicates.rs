use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crate::scanner::FileInfo;

/// Represents a set of files that share identical contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub file_size: u64,
    pub hash: String,
    pub files: Vec<PathBuf>,
}

impl DuplicateGroup {
    /// Storage that could be reclaimed if all copies except one were deleted.
    /// Formula: (total_copies - 1) * file_size.
    pub fn reclaimable_bytes(&self) -> u64 {
        if self.files.len() > 1 {
            (self.files.len() as u64 - 1) * self.file_size
        } else {
            0
        }
    }
}

/// Comprehensive duplicate detection summary.
#[derive(Debug, Clone, Default)]
pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub total_duplicate_files: usize,
    pub total_reclaimable_bytes: u64,
    pub empty_files_count: usize,
}

/// Computes the SHA-256 cryptographic hash of a file by reading it in 64 KB chunks.
///
/// This avoids loading entire files into memory, keeping RAM usage low
/// even for multi-gigabyte videos or archives.
pub fn hash_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Identifies duplicate files across a collection of scanned files.
///
/// Optimization:
/// 1. Group files by exact byte size first. If a file has a unique size,
///    it cannot be a duplicate and its contents are NEVER read.
/// 2. For files with identical sizes (>= 1 byte), compute SHA-256 hashes.
/// 3. Files with identical sizes AND identical SHA-256 hashes are verified duplicates.
pub fn find_duplicates(files: &[FileInfo]) -> DuplicateReport {
    let mut size_map: HashMap<u64, Vec<&FileInfo>> = HashMap::new();
    let mut empty_files_count = 0;

    for file in files {
        if file.size == 0 {
            empty_files_count += 1;
        } else {
            size_map.entry(file.size).or_default().push(file);
        }
    }

    let mut duplicate_groups = Vec::new();
    let mut total_duplicate_files = 0;
    let mut total_reclaimable = 0;

    // Only compare files that share the exact same size
    for (&size, candidates) in &size_map {
        if candidates.len() < 2 {
            continue; // Unique size: cannot be a duplicate!
        }

        let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for candidate in candidates {
            match hash_file_sha256(&candidate.path) {
                Ok(hash) => {
                    hash_map
                        .entry(hash)
                        .or_default()
                        .push(candidate.path.clone());
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Could not read file for hashing '{}': {err}",
                        candidate.path.display()
                    );
                }
            }
        }

        for (hash, matching_paths) in hash_map {
            if matching_paths.len() >= 2 {
                let group = DuplicateGroup {
                    file_size: size,
                    hash,
                    files: matching_paths,
                };
                total_duplicate_files += group.files.len();
                total_reclaimable += group.reclaimable_bytes();
                duplicate_groups.push(group);
            }
        }
    }

    // Sort groups so the ones with the most reclaimable space appear first
    duplicate_groups.sort_by(|a, b| {
        b.reclaimable_bytes()
            .cmp(&a.reclaimable_bytes())
            .then_with(|| b.file_size.cmp(&a.file_size))
    });

    DuplicateReport {
        groups: duplicate_groups,
        total_duplicate_files,
        total_reclaimable_bytes: total_reclaimable,
        empty_files_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_duplicate_group_reclaimable() {
        let group = DuplicateGroup {
            file_size: 1000,
            hash: "abc".into(),
            files: vec![
                PathBuf::from("a.txt"),
                PathBuf::from("b.txt"),
                PathBuf::from("c.txt"),
            ],
        };
        // 3 files, each 1000 bytes. Keeping 1 original means 2 * 1000 = 2000 reclaimable.
        assert_eq!(group.reclaimable_bytes(), 2000);
    }

    #[test]
    fn test_hash_file_sha256() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("cleansweep_test_hash.txt");

        {
            let mut file = File::create(&file_path).expect("Create temp file");
            file.write_all(b"CleanSweep Test Content")
                .expect("Write temp file");
        }

        let hash = hash_file_sha256(&file_path).expect("Should compute hash");
        // SHA-256 of "CleanSweep Test Content" is well-defined
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // 256 bits = 64 hex characters

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_find_duplicates_ignores_unique_sizes() {
        let sample = vec![
            FileInfo {
                name: "file1.txt".into(),
                path: PathBuf::from("file1.txt"),
                size: 100,
                modified: None,
            },
            FileInfo {
                name: "file2.txt".into(),
                path: PathBuf::from("file2.txt"),
                size: 200,
                modified: None,
            },
        ];

        let report = find_duplicates(&sample);
        assert!(report.groups.is_empty());
        assert_eq!(report.total_reclaimable_bytes, 0);
    }
}
