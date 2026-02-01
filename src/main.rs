use clap::Parser;
use std::path::Path;
use walkdir::WalkDir;

mod metrics;
mod models;
mod parser;
mod report;

use models::{AnalysisResult, OutputFormat, StructInfo};

const AFTER_HELP: &str = "\nMETRICS EXPLAINED:
    LCOM (Lack of Cohesion in Methods) - Range: 0.0 to 1.0 (lower is better)
        Measures how closely related methods are within a struct.
        • 0.0    = Perfect cohesion - all methods use all fields
        • 0.0-0.5 = Good cohesion - methods work on related field subsets
        • 0.5-0.8 = Low cohesion - may indicate multiple responsibilities
        • 1.0    = No cohesion - methods share no fields (consider splitting)

    CBO (Coupling Between Objects) - Range: 0+ (lower is better)
        Counts dependencies on other structs defined in the analyzed codebase.
        Does not count external types (String, Vec, etc.) or primitives.
        • 0-2  = Low coupling, easy to test and reuse
        • 3-5  = Moderate coupling, acceptable
        • 6+   = High coupling, difficult to maintain

    WMC (Weighted Methods per Class) - Range: 0+ (lower is better)
        Sum of cyclomatic complexities across all methods.
        Complexity is 1 + number of branches (if, match, while, for, loop).
        • 0-10  = Simple, easy to understand
        • 11-20 = Moderate complexity
        • 21-40 = Complex, consider refactoring
        • 40+   = God class, needs decomposition

EXAMPLES:
    # Analyze current project with table output
    rust-arch-metrics src/

    # Export metrics for further processing
    rust-arch-metrics src/ --format json --output metrics.json

    # Import into spreadsheet
    rust-arch-metrics src/ --format csv --output metrics.csv

    # Exclude test files
    rust-arch-metrics src/ --exclude test

    # Focus on high-complexity structs
    rust-arch-metrics src/ --format json | jq '.[] | select(.wmc > 40)'

    # Debug parsing of a specific struct
    rust-arch-metrics src/ --debug-struct MyStruct

SEE ALSO:
    https://en.wikipedia.org/wiki/Lack_of_cohesion_in_methods
    https://en.wikipedia.org/wiki/Coupling_(computer_programming)";

#[derive(Parser)]
#[command(name = "rust-arch-metrics")]
#[command(about = "Calculate architectural metrics (LCOM, CBO, WMC) for Rust code")]
#[command(
    long_about = "Analyzes Rust source files and calculates three key object-oriented metrics:
\n\
  • LCOM - Lack of Cohesion in Methods (how related are the methods)\n\
  • CBO  - Coupling Between Objects (dependencies on other structs)\n\
  • WMC  - Weighted Methods per Class (cyclomatic complexity sum)\n\
\n\
These metrics help identify architectural issues like God Classes, Feature Envy, \
and low cohesion that make code harder to maintain.",
    after_help = AFTER_HELP,
    version
)]
struct Cli {
    /// Path to the Rust project directory or single .rs file to analyze
    #[arg(value_name = "PATH")]
    path: String,

    /// Output format
    #[arg(short, long, value_name = "FORMAT", default_value = "table",
          help = "Output format: table, json, or csv\n\
                  • table - Human-readable aligned columns (default)\n\
                  • json  - Machine-readable with full precision\n\
                  • csv   - Spreadsheet-compatible")]
    format: String,

    /// Comma-separated list of metrics to include
    #[arg(short, long, value_name = "METRICS", default_value = "all",
          help = "Metrics to calculate: lcom,cbo,wmc or all (default)")]
    metrics: String,

    /// Pattern to exclude files/directories from analysis
    #[arg(long, value_name = "PATTERN",
          help = "Skip files/directories matching this substring\n\
                  Example: --exclude test (skips files with 'test' in name)")]
    exclude: Option<String>,

    /// Output file path (default: print to stdout)
    #[arg(short, long, value_name = "FILE",
          help = "Write output to file instead of stdout")]
    output: Option<String>,

    /// Debug a specific struct's parsed data
    #[arg(long, value_name = "STRUCT_NAME",
          help = "Print detailed parsing info for a struct\n\
                  Shows fields, methods, field access patterns, and traits")]
    debug_struct: Option<String>,
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

    // Handle debug output if requested
    if let Some(debug_name) = cli.debug_struct {
        for s in &all_structs {
            if s.name == debug_name {
                println!("=== Debug: {} ===", s.name);
                println!("Fields ({}):", s.fields.len());
                for f in &s.fields {
                    println!("  - {}: {}", f.name, f.ty);
                }
                println!("\nMethods ({}):", s.methods.len());
                for (i, m) in s.methods.iter().enumerate() {
                    println!("  Method {}: fields_accessed={:?}, complexity={}",
                        i, m.fields_accessed, m.cyclomatic_complexity);
                }
                println!("\nExternal types: {:?}", s.external_types);
                println!("Traits implemented: {:?}", s.traits);
            }
        }
        return Ok(());
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
