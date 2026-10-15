use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::age::OldFilesSummary;
use crate::duplicates::DuplicateReport;
use crate::scanner::{ScanResult, format_size, get_largest_files};
use crate::types::FileTypeSummary;

/// Generates a comprehensive plain-text storage analysis report.
pub fn generate_report_content(
    scan_result: &ScanResult,
    largest_limit: usize,
    duplicate_report: &DuplicateReport,
    type_summaries: &[FileTypeSummary],
    old_summary: Option<&OldFilesSummary>,
) -> String {
    let mut out = String::new();

    out.push_str("=================================================================\n");
    out.push_str("                   CLEANSWEEP STORAGE REPORT                     \n");
    out.push_str("=================================================================\n\n");

    out.push_str(&format!(
        "Scan Target:    {}\n",
        scan_result.root_path.display()
    ));
    out.push_str(&format!(
        "Scan Timestamp: {:?}\n\n",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    ));

    out.push_str("1. OVERALL STORAGE SUMMARY\n");
    out.push_str("-----------------------------------------------------------------\n");
    out.push_str(&format!(
        "  Total regular files:   {}\n",
        scan_result.stats.total_files
    ));
    out.push_str(&format!(
        "  Total directories:     {}\n",
        scan_result.stats.total_dirs
    ));
    out.push_str(&format!(
        "  Total storage used:    {}\n",
        format_size(scan_result.stats.total_size)
    ));
    out.push_str(&format!(
        "  Skipped symlinks:      {}\n",
        scan_result.stats.skipped_symlinks
    ));
    out.push_str(&format!(
        "  Skipped read errors:   {}\n\n",
        scan_result.stats.skipped_errors
    ));

    out.push_str(&format!("2. LARGEST FILES (Top {largest_limit})\n"));
    out.push_str("-----------------------------------------------------------------\n");
    let largest = get_largest_files(scan_result.files.clone(), largest_limit);
    if largest.is_empty() {
        out.push_str("  No regular files found.\n\n");
    } else {
        for (idx, file) in largest.iter().enumerate() {
            out.push_str(&format!(
                "  {:>2}. {:>10}  {}\n      Path: {}\n",
                idx + 1,
                format_size(file.size),
                file.name,
                file.path.display()
            ));
        }
        out.push('\n');
    }

    out.push_str("3. DUPLICATE FILES & RECLAIMABLE STORAGE\n");
    out.push_str("-----------------------------------------------------------------\n");
    out.push_str(&format!(
        "  Identified duplicate groups: {}\n",
        duplicate_report.groups.len()
    ));
    out.push_str(&format!(
        "  Total redundant files:       {}\n",
        duplicate_report.total_duplicate_files
    ));
    out.push_str(&format!(
        "  Potential reclaimable space: {}\n",
        format_size(duplicate_report.total_reclaimable_bytes)
    ));
    if duplicate_report.empty_files_count > 0 {
        out.push_str(&format!(
            "  Empty (0-byte) files:        {}\n",
            duplicate_report.empty_files_count
        ));
    }
    out.push('\n');

    for (group_idx, group) in duplicate_report.groups.iter().enumerate().take(15) {
        out.push_str(&format!(
            "  Group #{}: size {} each | {} copies | reclaimable: {}\n",
            group_idx + 1,
            format_size(group.file_size),
            group.files.len(),
            format_size(group.reclaimable_bytes())
        ));
        for (copy_idx, path) in group.files.iter().enumerate() {
            let label = if copy_idx == 0 { "[Keep]" } else { "[Copy]" };
            out.push_str(&format!("    {label} {}\n", path.display()));
        }
        out.push('\n');
    }

    out.push_str("4. STORAGE USAGE BY FILE TYPE\n");
    out.push_str("-----------------------------------------------------------------\n");
    out.push_str("  Extension          Count        Total Size   % of Total\n");
    out.push_str("  ---------------------------------------------------------------\n");
    for item in type_summaries.iter().take(20) {
        out.push_str(&format!(
            "  .{:<15}  {:>7}  {:>14}  {:>10.2}%\n",
            item.extension,
            item.count,
            format_size(item.total_size),
            item.percentage
        ));
    }
    out.push('\n');

    if let Some(old) = old_summary {
        out.push_str(&format!(
            "5. FILE-AGE ANALYSIS (Files older than {} days)\n",
            old.days_threshold
        ));
        out.push_str("-----------------------------------------------------------------\n");
        out.push_str(&format!("  Files identified: {}\n", old.files.len()));
        out.push_str(&format!(
            "  Total size:       {}\n\n",
            format_size(old.total_size)
        ));
        for (idx, f) in old.files.iter().enumerate().take(15) {
            out.push_str(&format!(
                "  {:>2}. {:>10}  ({} days old)  {}\n",
                idx + 1,
                format_size(f.size),
                f.age_days,
                f.path.display()
            ));
        }
        out.push('\n');
    }

    out.push_str("=================================================================\n");
    out.push_str("SAFETY NOTICE:\n");
    out.push_str("CleanSweep is strictly read-only and never modifies or deletes files.\n");
    out.push_str("Always verify file contents manually before taking any cleanup action.\n");
    out.push_str("=================================================================\n");

    out
}

/// Saves the generated report to disk safely.
///
/// Refuses to overwrite an existing file unless `force` is set to true.
pub fn export_report(
    content: &str,
    target_path: &Path,
    force: bool,
) -> Result<PathBuf, std::io::Error> {
    if target_path.exists() && !force {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "Report file '{}' already exists. Use --force to overwrite.",
                target_path.display()
            ),
        ));
    }

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(target_path)?;
    file.write_all(content.as_bytes())?;

    Ok(target_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ScanStats;

    #[test]
    fn test_generate_report_content() {
        let scan_result = ScanResult {
            root_path: PathBuf::from("C:\\test"),
            files: Vec::new(),
            stats: ScanStats {
                total_files: 10,
                total_dirs: 2,
                total_size: 50_000,
                skipped_symlinks: 0,
                skipped_errors: 0,
            },
        };

        let dup = DuplicateReport::default();
        let types = Vec::new();

        let content = generate_report_content(&scan_result, 5, &dup, &types, None);
        assert!(content.contains("CLEANSWEEP STORAGE REPORT"));
        assert!(content.contains("C:\\test"));
        assert!(content.contains("50.00 KB") || content.contains("48.83 KB"));
    }

    #[test]
    fn test_export_report_protects_existing_file() {
        let temp_dir = std::env::temp_dir();
        let report_file = temp_dir.join("cleansweep_test_existing_report.txt");

        // Create file first
        {
            let mut f = File::create(&report_file).expect("Create test file");
            f.write_all(b"original").expect("Write test file");
        }

        // Export without force should return AlreadyExists
        let result = export_report("new content", &report_file, false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );

        // Export with force should succeed
        let result_force = export_report("new content", &report_file, true);
        assert!(result_force.is_ok());

        let _ = std::fs::remove_file(report_file);
    }
}
