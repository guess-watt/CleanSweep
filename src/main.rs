
use std::fs;

fn main() {
    let folder = r"C:\Users\anwin\Downloads";

    println!("CleanSweep - Largest Files\n");

    match fs::read_dir(folder) {
        Ok(entries) => {
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        files.push((entry.file_name(), metadata.len()));
                    }
                }
            }

            files.sort_by(|a, b| b.1.cmp(&a.1));

            for (name, size) in files.iter().take(10) {
                println!(
                    "{:.2} MB - {}",
                    *size as f64 / (1024.0 * 1024.0),
                    name.to_string_lossy()
                );
            }
        }
        Err(error) => println!("Error: {error}"),
    }
}
