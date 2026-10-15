use cleansweep::age::find_old_files;
use cleansweep::cli::{Command, parse_command};
use cleansweep::duplicates::find_duplicates;
use cleansweep::report::{export_report, generate_report_content};
use cleansweep::scanner::{FileInfo, ScanOptions, scan_directory};
use cleansweep::types::summarize_by_type;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Helper to create an isolated test directory structure.
fn create_test_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push("cleansweep_tests");
    dir.push(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("Create test dir");
    dir
}

#[test]
fn test_recursive_vs_single_level_scanning() {
    let test_dir = create_test_dir("recursive_test");
    let sub_dir = test_dir.join("subfolder");
    fs::create_dir_all(&sub_dir).expect("Create subfolder");

    // File in root
    let root_file = test_dir.join("root.txt");
    fs::write(&root_file, b"Root file content").expect("Write root file");

    // File in subfolder
    let sub_file = sub_dir.join("nested.txt");
    fs::write(&sub_file, b"Nested file content").expect("Write nested file");

    // 1. Non-recursive scan should only find root.txt
    let non_rec_opts = ScanOptions { recursive: false };
    let non_rec_res = scan_directory(&test_dir, &non_rec_opts).expect("Scan non-recursive");
    assert_eq!(non_rec_res.stats.total_files, 1);
    assert_eq!(non_rec_res.files[0].name, "root.txt");

    // 2. Recursive scan should find both root.txt and nested.txt
    let rec_opts = ScanOptions { recursive: true };
    let rec_res = scan_directory(&test_dir, &rec_opts).expect("Scan recursive");
    assert_eq!(rec_res.stats.total_files, 2);

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_duplicate_detection_different_names_same_content() {
    let test_dir = create_test_dir("duplicates_test");

    let file_a = test_dir.join("original.dat");
    let file_b = test_dir.join("copy_with_different_name.bin");
    let file_c = test_dir.join("different_content.txt");

    let identical_bytes = b"Exact same payload content for testing duplicates!";
    fs::write(&file_a, identical_bytes).expect("Write file A");
    fs::write(&file_b, identical_bytes).expect("Write file B");
    fs::write(&file_c, b"Completely different content!").expect("Write file C");

    let scan = scan_directory(&test_dir, &ScanOptions { recursive: false }).expect("Scan");
    let report = find_duplicates(&scan.files);

    // file_a and file_b must be grouped together
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].files.len(), 2);
    assert_eq!(report.groups[0].file_size, identical_bytes.len() as u64);
    assert_eq!(report.total_reclaimable_bytes, identical_bytes.len() as u64);

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_storage_usage_by_type_calculation() {
    let files = vec![
        FileInfo {
            name: "image1.png".into(),
            path: PathBuf::from("image1.png"),
            size: 400,
            modified: None,
        },
        FileInfo {
            name: "image2.PNG".into(),
            path: PathBuf::from("image2.PNG"),
            size: 600,
            modified: None,
        },
        FileInfo {
            name: "no_ext".into(),
            path: PathBuf::from("no_ext"),
            size: 1000,
            modified: None,
        },
    ];

    let total = 2000;
    let summaries = summarize_by_type(&files, total);

    assert_eq!(summaries.len(), 2);
    // (no extension) is 1000 bytes (50%)
    assert_eq!(summaries[0].extension, "(no extension)");
    assert_eq!(summaries[0].count, 1);
    assert_eq!(summaries[0].total_size, 1000);
    assert!((summaries[0].percentage - 50.0).abs() < 0.01);

    // png is 1000 bytes (50%)
    assert_eq!(summaries[1].extension, "png");
    assert_eq!(summaries[1].count, 2);
    assert_eq!(summaries[1].total_size, 1000);
}

#[test]
fn test_file_age_threshold_and_sorting() {
    let now = SystemTime::now();
    let old_time = now - Duration::from_secs(120 * 86_400); // 120 days old
    let new_time = now - Duration::from_secs(10 * 86_400); // 10 days old

    let files = vec![
        FileInfo {
            name: "old_big.zip".into(),
            path: PathBuf::from("old_big.zip"),
            size: 9000,
            modified: Some(old_time),
        },
        FileInfo {
            name: "old_small.zip".into(),
            path: PathBuf::from("old_small.zip"),
            size: 1000,
            modified: Some(old_time),
        },
        FileInfo {
            name: "new_file.txt".into(),
            path: PathBuf::from("new_file.txt"),
            size: 5000,
            modified: Some(new_time),
        },
    ];

    let summary = find_old_files(&files, 60);
    assert_eq!(summary.files.len(), 2);
    // Largest old file first
    assert_eq!(summary.files[0].name, "old_big.zip");
    assert_eq!(summary.files[1].name, "old_small.zip");
    assert_eq!(summary.total_size, 10000);
}

#[test]
fn test_cli_subcommands_and_flag_combinations() {
    // 1. largest with -r and -n
    let args1 = vec![
        "cleansweep".into(),
        "largest".into(),
        "C:\\Test".into(),
        "-r".into(),
        "-n".into(),
        "15".into(),
    ];
    match parse_command(&args1).unwrap() {
        Command::Largest {
            path,
            limit,
            recursive,
        } => {
            assert_eq!(path, PathBuf::from("C:\\Test"));
            assert_eq!(limit, 15);
            assert!(recursive);
        }
        _ => panic!("Expected Largest"),
    }

    // 2. report with custom output and force
    let args2 = vec![
        "cleansweep".into(),
        "report".into(),
        "C:\\Test".into(),
        "-o".into(),
        "my_report.txt".into(),
        "-f".into(),
    ];
    match parse_command(&args2).unwrap() {
        Command::Report {
            path,
            output,
            force,
            ..
        } => {
            assert_eq!(path, PathBuf::from("C:\\Test"));
            assert_eq!(output, Some(PathBuf::from("my_report.txt")));
            assert!(force);
        }
        _ => panic!("Expected Report"),
    }
}

#[test]
fn test_exportable_report_workflow() {
    let test_dir = create_test_dir("report_test");
    let scan_opts = ScanOptions { recursive: false };
    let scan = scan_directory(&test_dir, &scan_opts).expect("Scan");

    let dup = find_duplicates(&scan.files);
    let types = summarize_by_type(&scan.files, scan.stats.total_size);
    let old = find_old_files(&scan.files, 90);

    let report_text = generate_report_content(&scan, 5, &dup, &types, Some(&old));
    assert!(report_text.contains("CLEANSWEEP STORAGE REPORT"));

    let report_file = test_dir.join("test_report.txt");
    let saved = export_report(&report_text, &report_file, false).expect("Export report");
    assert!(saved.exists());

    let _ = fs::remove_dir_all(&test_dir);
}
