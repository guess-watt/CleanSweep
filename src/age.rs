use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::scanner::FileInfo;

/// Details for an old/inactive file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldFileInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub age_days: u64,
}

/// Result of the file age analysis.
#[derive(Debug, Clone, Default)]
pub struct OldFilesSummary {
    pub days_threshold: u64,
    pub files: Vec<OldFileInfo>,
    pub total_size: u64,
    pub skipped_timestamp_errors: usize,
}

/// Identifies files whose last modification time is older than `days_threshold`.
///
/// Safety & Transparency:
/// - These files are labeled "old" / "inactive". They are NOT assumed to be safe to delete.
/// - System clock drift or future timestamps are handled gracefully without panicking.
pub fn find_old_files(files: &[FileInfo], days_threshold: u64) -> OldFilesSummary {
    let now = SystemTime::now();
    let threshold_duration = Duration::from_secs(days_threshold.saturating_mul(86_400));

    let mut old_files = Vec::new();
    let mut total_size = 0;
    let mut skipped_timestamp_errors = 0;

    for file in files {
        match file.modified {
            Some(modified_time) => {
                match now.duration_since(modified_time) {
                    Ok(age_duration) => {
                        if age_duration >= threshold_duration {
                            let age_days = age_duration.as_secs() / 86_400;
                            total_size += file.size;
                            old_files.push(OldFileInfo {
                                name: file.name.clone(),
                                path: file.path.clone(),
                                size: file.size,
                                age_days,
                            });
                        }
                    }
                    Err(_) => {
                        // File modified time is in the future (clock skew)
                        skipped_timestamp_errors += 1;
                    }
                }
            }
            None => {
                // OS could not provide a modification timestamp
                skipped_timestamp_errors += 1;
            }
        }
    }

    // Sort old files from largest to smallest
    old_files.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));

    OldFilesSummary {
        days_threshold,
        files: old_files,
        total_size,
        skipped_timestamp_errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_old_files_filtering() {
        let now = SystemTime::now();
        let hundred_days_ago = now - Duration::from_secs(100 * 86_400);
        let ten_days_ago = now - Duration::from_secs(10 * 86_400);

        let files = vec![
            FileInfo {
                name: "old_archive.zip".into(),
                path: PathBuf::from("old_archive.zip"),
                size: 5000,
                modified: Some(hundred_days_ago),
            },
            FileInfo {
                name: "recent_doc.txt".into(),
                path: PathBuf::from("recent_doc.txt"),
                size: 200,
                modified: Some(ten_days_ago),
            },
            FileInfo {
                name: "no_timestamp.dat".into(),
                path: PathBuf::from("no_timestamp.dat"),
                size: 100,
                modified: None,
            },
        ];

        let summary = find_old_files(&files, 30);
        assert_eq!(summary.days_threshold, 30);
        assert_eq!(summary.files.len(), 1);
        assert_eq!(summary.files[0].name, "old_archive.zip");
        assert_eq!(summary.files[0].size, 5000);
        assert!(summary.files[0].age_days >= 99);
        assert_eq!(summary.total_size, 5000);
        assert_eq!(summary.skipped_timestamp_errors, 1);
    }
}
