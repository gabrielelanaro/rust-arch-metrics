use clap::Parser;
use std::path::Path;
use walkdir::WalkDir;

mod metrics;
mod models;
mod parser;
mod report;

use models::{AnalysisResult, OutputFormat, StructInfo};

#[derive(Parser)]
#[command(name = "rust-arch-metrics")]
#[command(about = "CLI tool for measuring architectural metrics in Rust code")]
#[command(version)]
struct Cli {
    /// Path to the Rust project or file to analyze
    #[arg(value_name = "PATH")]
    path: String,

    /// Output format: table, json, csv
    #[arg(short, long, value_name = "FORMAT", default_value = "table")]
    format: String,

    /// Comma-separated metrics to calculate: lcom,cbo,wmc or "all"
    #[arg(short, long, value_name = "METRICS", default_value = "all")]
    metrics: String,

    /// Glob pattern to exclude files
    #[arg(long, value_name = "PATTERN")]
    exclude: Option<String>,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let output_format: OutputFormat = cli.format.parse()?;

    // Collect all Rust files
    let rust_files = collect_rust_files(&cli.path, cli.exclude.as_deref())?;

    if rust_files.is_empty() {
        eprintln!("No Rust files found in {}", cli.path);
        std::process::exit(1);
    }

    // Parse all files and collect struct information
    let mut all_structs: Vec<StructInfo> = Vec::new();

    for file_path in &rust_files {
        let content = std::fs::read_to_string(file_path)?;

        match parser::parse_file(&content) {
            Ok(structs) => {
                all_structs.extend(structs);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", file_path.display(), e);
            }
        }
    }

    if all_structs.is_empty() {
        eprintln!("No structs found in the analyzed files.");
        std::process::exit(0);
    }

    // Calculate metrics for each struct
    let results: Vec<AnalysisResult> = all_structs
        .iter()
        .map(|s| metrics::analyze_struct(s, &all_structs))
        .collect();

    // Generate report
    report::generate_report(&results, output_format, cli.output.as_deref())?;

    Ok(())
}

fn collect_rust_files(
    path: &str,
    exclude_pattern: Option<&str>,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let path = Path::new(path);

    if path.is_file() {
        if path.extension().map_or(false, |e| e == "rs") {
            files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                if let Some(pattern) = exclude_pattern {
                    let name = e.file_name().to_string_lossy();
                    !name.contains(pattern)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}
