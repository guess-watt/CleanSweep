use std::collections::HashMap;

use crate::scanner::FileInfo;

/// Aggregated storage usage for a specific file extension.
#[derive(Debug, Clone, PartialEq)]
pub struct FileTypeSummary {
    pub extension: String,
    pub count: usize,
    pub total_size: u64,
    pub percentage: f64,
}

/// Summarizes scanned files by their extension, ordered by total storage usage descending.
///
/// Extensions are normalized to lowercase. Files without an extension are grouped under "(no extension)".
pub fn summarize_by_type(files: &[FileInfo], total_scan_size: u64) -> Vec<FileTypeSummary> {
    let mut type_map: HashMap<String, (usize, u64)> = HashMap::new();

    for file in files {
        let ext = file
            .path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "(no extension)".to_string());

        let entry = type_map.entry(ext).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += file.size;
    }

    let mut summaries: Vec<FileTypeSummary> = type_map
        .into_iter()
        .map(|(extension, (count, total_size))| {
            let percentage = if total_scan_size > 0 {
                (total_size as f64 / total_scan_size as f64) * 100.0
            } else {
                0.0
            };
            FileTypeSummary {
                extension,
                count,
                total_size,
                percentage,
            }
        })
        .collect();

    // Sort by total storage usage descending; tie-break alphabetically by extension
    summaries.sort_by(|a, b| {
        b.total_size
            .cmp(&a.total_size)
            .then_with(|| a.extension.cmp(&b.extension))
    });

    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_summarize_by_type_grouping_and_sorting() {
        let files = vec![
            FileInfo {
                name: "a.PDF".into(),
                path: PathBuf::from("a.PDF"),
                size: 200,
                modified: None,
            },
            FileInfo {
                name: "b.pdf".into(),
                path: PathBuf::from("b.pdf"),
                size: 300,
                modified: None,
            },
            FileInfo {
                name: "c.zip".into(),
                path: PathBuf::from("c.zip"),
                size: 1000,
                modified: None,
            },
            FileInfo {
                name: "LICENSE".into(),
                path: PathBuf::from("LICENSE"),
                size: 50,
                modified: None,
            },
        ];

        let total_size = 200 + 300 + 1000 + 50; // 1550
        let summaries = summarize_by_type(&files, total_size);

        assert_eq!(summaries.len(), 3);

        // zip should be first (1000 bytes)
        assert_eq!(summaries[0].extension, "zip");
        assert_eq!(summaries[0].count, 1);
        assert_eq!(summaries[0].total_size, 1000);

        // pdf should be second (500 bytes: a.PDF + b.pdf)
        assert_eq!(summaries[1].extension, "pdf");
        assert_eq!(summaries[1].count, 2);
        assert_eq!(summaries[1].total_size, 500);

        // (no extension) should be third (50 bytes)
        assert_eq!(summaries[2].extension, "(no extension)");
        assert_eq!(summaries[2].count, 1);
        assert_eq!(summaries[2].total_size, 50);
    }
}
