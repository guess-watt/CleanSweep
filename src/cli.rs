use std::path::PathBuf;

/// The parsed CLI command and its options.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Scan {
        path: PathBuf,
        limit: usize,
        recursive: bool,
    },
    Largest {
        path: PathBuf,
        limit: usize,
        recursive: bool,
    },
    Duplicates {
        path: PathBuf,
        recursive: bool,
    },
    Types {
        path: PathBuf,
        recursive: bool,
    },
    Old {
        path: PathBuf,
        days: u64,
        limit: usize,
        recursive: bool,
    },
    Report {
        path: PathBuf,
        output: Option<PathBuf>,
        days: u64,
        limit: usize,
        recursive: bool,
        force: bool,
    },
    Help,
}

pub fn print_help() {
    println!("CleanSweep - Fast & Safe CLI Storage Analyzer for Windows\n");
    println!("Usage:");
    println!("  cleansweep <command> <folder-path> [options]\n");
    println!("Commands:");
    println!("  scan <path>        Quick overview of folder storage and top largest files");
    println!("  largest <path>     List the N largest files in descending order");
    println!("  duplicates <path>  Find duplicate files by comparing SHA-256 contents");
    println!("  types <path>       Summarize storage usage grouped by file extension");
    println!("  old <path>         Identify files modified more than N days ago");
    println!("  report <path>      Generate and export a comprehensive storage report");
    println!("  help               Display this help guide\n");
    println!("Common Options:");
    println!("  -r, --recursive    Scan subdirectories recursively");
    println!("  -n, --limit <N>    Limit number of files displayed (default: 10)");
    println!("  -d, --days <N>     Age threshold in days for old files (default: 90)");
    println!(
        "  -o, --output <path> Path to write the report file (default: cleansweep-report.txt)"
    );
    println!("  -f, --force        Overwrite output report file if it already exists\n");
    println!("Examples:");
    println!(r#"  cleansweep scan "C:\Users\username\Downloads""#);
    println!(r#"  cleansweep scan . -r -n 5"#);
    println!(r#"  cleansweep largest "C:\Users\username\Downloads" --limit 15 -r"#);
    println!(r#"  cleansweep duplicates "C:\Users\username\Downloads" -r"#);
    println!(r#"  cleansweep types "C:\Users\username\Downloads" -r"#);
    println!(r#"  cleansweep old "C:\Users\username\Downloads" --days 180"#);
    println!(
        r#"  cleansweep report "C:\Users\username\Downloads" -r --output "reports\storage-report.txt""#
    );
}

/// Parses CLI arguments into a structured `Command`.
pub fn parse_command(args: &[String]) -> Result<Command, String> {
    if args.len() < 2 {
        return Ok(Command::Help);
    }

    let first = args[1].as_str();

    if first == "help" || first == "--help" || first == "-h" {
        return Ok(Command::Help);
    }

    // Determine subcommand and remaining args
    let (subcmd, path_idx) = match first {
        "scan" | "largest" | "duplicates" | "types" | "old" | "report" => (first, 2),
        _ => {
            // Backward compatibility: If no recognized command word was given, treat args[1] as the path for "scan"
            ("scan", 1)
        }
    };

    if path_idx >= args.len() {
        return Err(format!(
            "Command '{subcmd}' requires a folder path argument. Use 'cleansweep help' for examples."
        ));
    }

    let target_path = PathBuf::from(&args[path_idx]);

    // Parse flags from the remaining arguments
    let mut limit: usize = 10;
    let mut days: u64 = 90;
    let mut recursive: bool = false;
    let mut output: Option<PathBuf> = None;
    let mut force: bool = false;

    let mut i = path_idx + 1;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--recursive" => {
                recursive = true;
                i += 1;
            }
            "-n" | "--limit" => {
                if i + 1 >= args.len() {
                    return Err("Flag '--limit' requires a positive integer value.".into());
                }
                match args[i + 1].parse::<usize>() {
                    Ok(n) if n > 0 => {
                        limit = n;
                        i += 2;
                    }
                    _ => {
                        return Err(format!(
                            "Invalid limit '{}': must be an integer greater than 0.",
                            args[i + 1]
                        ));
                    }
                }
            }
            "-d" | "--days" => {
                if i + 1 >= args.len() {
                    return Err("Flag '--days' requires a number of days.".into());
                }
                match args[i + 1].parse::<u64>() {
                    Ok(d) if d > 0 => {
                        days = d;
                        i += 2;
                    }
                    _ => {
                        return Err(format!(
                            "Invalid days '{}': must be a positive integer.",
                            args[i + 1]
                        ));
                    }
                }
            }
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    return Err("Flag '--output' requires a file path.".into());
                }
                output = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "-f" | "--force" => {
                force = true;
                i += 1;
            }
            other => {
                // Positional limit number support: cleansweep largest <path> 5
                if let Ok(n) = other.parse::<usize>()
                    && n > 0
                {
                    limit = n;
                    i += 1;
                    continue;
                }
                return Err(format!(
                    "Unrecognized argument: '{other}'. Use 'cleansweep help' for options."
                ));
            }
        }
    }

    match subcmd {
        "scan" => Ok(Command::Scan {
            path: target_path,
            limit,
            recursive,
        }),
        "largest" => Ok(Command::Largest {
            path: target_path,
            limit,
            recursive,
        }),
        "duplicates" => Ok(Command::Duplicates {
            path: target_path,
            recursive,
        }),
        "types" => Ok(Command::Types {
            path: target_path,
            recursive,
        }),
        "old" => Ok(Command::Old {
            path: target_path,
            days,
            limit,
            recursive,
        }),
        "report" => Ok(Command::Report {
            path: target_path,
            output,
            days,
            limit,
            recursive,
            force,
        }),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        let args = vec!["cleansweep".into(), "help".into()];
        assert_eq!(parse_command(&args).unwrap(), Command::Help);

        let args_short = vec!["cleansweep".into(), "-h".into()];
        assert_eq!(parse_command(&args_short).unwrap(), Command::Help);
    }

    #[test]
    fn test_parse_scan_subcommand() {
        let args = vec![
            "cleansweep".into(),
            "scan".into(),
            "C:\\Downloads".into(),
            "-r".into(),
            "--limit".into(),
            "5".into(),
        ];
        match parse_command(&args).unwrap() {
            Command::Scan {
                path,
                limit,
                recursive,
            } => {
                assert_eq!(path, PathBuf::from("C:\\Downloads"));
                assert_eq!(limit, 5);
                assert!(recursive);
            }
            _ => panic!("Expected Command::Scan"),
        }
    }

    #[test]
    fn test_parse_duplicates_subcommand() {
        let args = vec![
            "cleansweep".into(),
            "duplicates".into(),
            "C:\\Downloads".into(),
            "-r".into(),
        ];
        match parse_command(&args).unwrap() {
            Command::Duplicates { path, recursive } => {
                assert_eq!(path, PathBuf::from("C:\\Downloads"));
                assert!(recursive);
            }
            _ => panic!("Expected Command::Duplicates"),
        }
    }

    #[test]
    fn test_parse_old_subcommand() {
        let args = vec![
            "cleansweep".into(),
            "old".into(),
            "C:\\Downloads".into(),
            "--days".into(),
            "180".into(),
        ];
        match parse_command(&args).unwrap() {
            Command::Old {
                path,
                days,
                limit,
                recursive,
            } => {
                assert_eq!(path, PathBuf::from("C:\\Downloads"));
                assert_eq!(days, 180);
                assert_eq!(limit, 10);
                assert!(!recursive);
            }
            _ => panic!("Expected Command::Old"),
        }
    }

    #[test]
    fn test_parse_backward_compatibility() {
        // Direct folder path without explicit "scan" prefix
        let args = vec!["cleansweep".into(), "C:\\Downloads".into(), "5".into()];
        match parse_command(&args).unwrap() {
            Command::Scan { path, limit, .. } => {
                assert_eq!(path, PathBuf::from("C:\\Downloads"));
                assert_eq!(limit, 5);
            }
            _ => panic!("Expected Command::Scan"),
        }
    }
}
