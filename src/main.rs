use cleansweep::cli::{Command, parse_command, print_help};
use cleansweep::scanner::{
    ScanOptions, ScanResult, format_size, get_largest_files, scan_directory,
};
use cleansweep::{age, duplicates, report, types};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let command = match parse_command(&args) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    match command {
        Command::Help => {
            print_help();
        }
        Command::Scan {
            path,
            limit,
            recursive,
        } => {
            let scan = run_scan(&path, recursive);
            print_scan_overview(&scan, limit);
        }
        Command::Largest {
            path,
            limit,
            recursive,
        } => {
            let scan = run_scan(&path, recursive);
            print_largest_files(&scan, limit);
        }
        Command::Duplicates { path, recursive } => {
            let scan = run_scan(&path, recursive);
            print_duplicates(&scan);
        }
        Command::Types { path, recursive } => {
            let scan = run_scan(&path, recursive);
            print_file_types(&scan);
        }
        Command::Old {
            path,
            days,
            limit,
            recursive,
        } => {
            let scan = run_scan(&path, recursive);
            print_old_files(&scan, days, limit);
        }
        Command::Report {
            path,
            output,
            days,
            limit,
            recursive,
            force,
        } => {
            let scan = run_scan(&path, recursive);
            run_and_export_report(&scan, output, days, limit, force);
        }
    }
}

/// Helper that invokes the scanner and handles top-level filesystem errors.
fn run_scan(path: &Path, recursive: bool) -> ScanResult {
    println!(
        "CleanSweep - Scanning: \"{}\" [Mode: {}]...",
        path.display(),
        if recursive {
            "Recursive"
        } else {
            "Single Folder"
        }
    );

    let options = ScanOptions { recursive };
    match scan_directory(path, &options) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error scanning '{}': {err}", path.display());
            std::process::exit(1);
        }
    }
}

fn print_scan_overview(scan: &ScanResult, limit: usize) {
    println!("\n--- Storage Overview ---");
    println!("  Total regular files:   {}", scan.stats.total_files);
    println!("  Directories inspected: {}", scan.stats.total_dirs);
    println!(
        "  Total storage:         {}",
        format_size(scan.stats.total_size)
    );
    if scan.stats.skipped_symlinks > 0 {
        println!("  Skipped symlinks:      {}", scan.stats.skipped_symlinks);
    }
    if scan.stats.skipped_errors > 0 {
        println!("  Skipped read errors:   {}", scan.stats.skipped_errors);
    }

    if scan.files.is_empty() {
        println!("\nNo regular files found.");
        return;
    }

    println!("\nTop {} largest file(s):", limit.min(scan.files.len()));
    let largest = get_largest_files(scan.files.clone(), limit);
    for (idx, file) in largest.iter().enumerate() {
        println!(
            "  {:>2}. {:>10}  {}",
            idx + 1,
            format_size(file.size),
            file.name
        );
        println!("      Path: {}", file.path.display());
    }
}

fn print_largest_files(scan: &ScanResult, limit: usize) {
    if scan.files.is_empty() {
        println!(
            "\nNo regular files found in '{}'.",
            scan.root_path.display()
        );
        return;
    }

    let largest = get_largest_files(scan.files.clone(), limit);
    let count = largest.len();

    println!(
        "\nFound {} file(s) across {} (showing top {} largest):\n",
        scan.stats.total_files,
        format_size(scan.stats.total_size),
        count
    );

    for (idx, file) in largest.iter().enumerate() {
        println!(
            "  {:>2}. {:>10}  {}",
            idx + 1,
            format_size(file.size),
            file.name
        );
        println!("      Path: {}", file.path.display());
    }
}

fn print_duplicates(scan: &ScanResult) {
    println!("\nAnalyzing files for duplicates by comparing SHA-256 contents...");
    let report = duplicates::find_duplicates(&scan.files);

    if report.groups.is_empty() {
        println!("No duplicate files detected! All files have distinct contents.");
        if report.empty_files_count > 0 {
            println!(
                "Note: Found {} empty (0-byte) file(s).",
                report.empty_files_count
            );
        }
        return;
    }

    println!(
        "\nIdentified {} duplicate group(s) across {} redundant files.",
        report.groups.len(),
        report.total_duplicate_files
    );
    println!(
        "Estimated reclaimable storage: {}\n",
        format_size(report.total_reclaimable_bytes)
    );

    for (idx, group) in report.groups.iter().enumerate() {
        println!(
            "Group #{} — Each file: {} ({} copies, reclaimable: {})",
            idx + 1,
            format_size(group.file_size),
            group.files.len(),
            format_size(group.reclaimable_bytes())
        );
        for (f_idx, path) in group.files.iter().enumerate() {
            let tag = if f_idx == 0 { "[Keep]" } else { "[Copy]" };
            println!("  {} {}", tag, path.display());
        }
        println!();
    }

    println!(
        "Safety Notice: CleanSweep never deletes files. Review paths manually before deleting."
    );
}

fn print_file_types(scan: &ScanResult) {
    if scan.files.is_empty() {
        println!("\nNo regular files found.");
        return;
    }

    let summaries = types::summarize_by_type(&scan.files, scan.stats.total_size);

    println!("\nStorage Usage by File Type:\n");
    println!("  Extension          Count        Total Size   % of Total");
    println!("  ---------------------------------------------------------------");
    for s in summaries {
        println!(
            "  .{:<15}  {:>7}  {:>14}  {:>10.2}%",
            s.extension,
            s.count,
            format_size(s.total_size),
            s.percentage
        );
    }
    println!("\nNote: File extensions do not guarantee the underlying format.");
}

fn print_old_files(scan: &ScanResult, days: u64, limit: usize) {
    if scan.files.is_empty() {
        println!("\nNo regular files found.");
        return;
    }

    let summary = age::find_old_files(&scan.files, days);

    println!("\nFile-Age Analysis (Threshold: >= {} days):\n", days);
    println!("  Files identified:  {}", summary.files.len());
    println!("  Total storage:     {}", format_size(summary.total_size));

    if summary.skipped_timestamp_errors > 0 {
        println!(
            "  Skipped (future/unavailable timestamp): {}",
            summary.skipped_timestamp_errors
        );
    }

    if summary.files.is_empty() {
        println!("\nNo files older than {} days found.", days);
        return;
    }

    let count = summary.files.len().min(limit);
    println!("\nTop {} largest old file(s):", count);
    for (idx, file) in summary.files.iter().take(count).enumerate() {
        println!(
            "  {:>2}. {:>10}  ({} days old)  {}",
            idx + 1,
            format_size(file.size),
            file.age_days,
            file.name
        );
        println!("      Path: {}", file.path.display());
    }

    println!("\nSafety Notice: Files are categorized as old, NOT safe to delete.");
}

fn run_and_export_report(
    scan: &ScanResult,
    output: Option<PathBuf>,
    days: u64,
    limit: usize,
    force: bool,
) {
    println!("\nGenerating comprehensive analysis report...");

    let dup_report = duplicates::find_duplicates(&scan.files);
    let type_summaries = types::summarize_by_type(&scan.files, scan.stats.total_size);
    let old_summary = age::find_old_files(&scan.files, days);

    let report_text = report::generate_report_content(
        scan,
        limit,
        &dup_report,
        &type_summaries,
        Some(&old_summary),
    );

    let default_name = format!(
        "cleansweep-report-{}.txt",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    let target_path = output.unwrap_or_else(|| PathBuf::from("reports").join(default_name));

    match report::export_report(&report_text, &target_path, force) {
        Ok(saved_path) => {
            println!("Report exported successfully to: {}", saved_path.display());
            println!("Summary:");
            println!("  Files inspected: {}", scan.stats.total_files);
            println!("  Total storage:   {}", format_size(scan.stats.total_size));
            println!(
                "  Duplicate space: {}",
                format_size(dup_report.total_reclaimable_bytes)
            );
        }
        Err(err) => {
            eprintln!("Failed to export report: {err}");
            std::process::exit(1);
        }
    }
}
