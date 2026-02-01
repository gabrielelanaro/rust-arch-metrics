# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust CLI tool that calculates architectural metrics for Rust codebases. It analyzes Rust source files and computes three key object-oriented metrics for each struct:

- **LCOM** (Lack of Cohesion in Methods): Measures how closely related methods are to each other within a struct (0-1 scale, lower is better)
- **CBO** (Coupling Between Objects): Counts dependencies on other structs in the codebase
- **WMC** (Weighted Methods per Class): Sum of cyclomatic complexities across all methods

## Build and Development Commands

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_lcom_perfectly_cohesive

# Run the CLI on the current codebase
cargo run -- src/

# Run with JSON output
cargo run -- src/ --format json

# Run with output to file
cargo run -- src/ --output metrics.json --format json

# Exclude specific patterns (e.g., test files)
cargo run -- src/ --exclude test

# Debug a specific struct's parsed data
cargo run -- src/ --debug-struct StructName
```

**Note**: The project is configured to treat warnings as errors via `.cargo/config.toml`.

## Architecture

The codebase follows a simple pipeline architecture:

### Main Flow ([src/main.rs](src/main.rs))
1. Parse CLI arguments with `clap`
2. Collect Rust files from the provided path using `walkdir`
3. Parse each file using `syn` to extract struct and method information
4. Calculate metrics for each struct
5. Generate output report (table, JSON, or CSV)

### Core Modules

**[src/parser.rs](src/parser.rs)**: Uses `syn`'s visitor pattern to parse Rust source files
- `StructVisitor` implements `Visit` to traverse the AST
- Extracts struct definitions, fields, and impl blocks
- Tracks field access patterns in methods (detects `self.field` access)
- Calculates cyclomatic complexity by counting branches (if, match, while, for, loop)
- Records traits implemented by each struct

**[src/metrics/](src/metrics/)**: Individual metric calculations
- `lcom.rs`: Henderson-Sellers LCOM formula - measures cohesion based on field access overlap between methods
- `cbo.rs`: Counts unique external struct dependencies from field types and external type references
- `wmc.rs`: Sums cyclomatic complexity of all methods (minimum 1 per method)

**[src/models.rs](src/models.rs)**: Core data structures
- `StructInfo`: Contains struct name, fields, methods, external types, and implemented traits
- `MethodInfo`: Tracks fields accessed and cyclomatic complexity
- `AnalysisResult`: Final output structure with LCOM, CBO, and WMC values

**[src/report.rs](src/report.rs)**: Output formatting (table, JSON, CSV)

### Key Dependencies

- `syn` (with "full", "visit" features): Rust parsing
- `clap`: CLI argument parsing
- `walkdir`: Directory traversal
- `serde`/`serde_json`: JSON serialization
- `csv`: CSV output

## Notes

- The parser only analyzes struct impl blocks (inherent and trait), not free functions
- Field access detection looks for `self.field_name` patterns
- CBO only counts couplings to other structs defined in the analyzed codebase, not external types like `String` or `Vec`
- The `--debug-struct` flag is useful for understanding how a specific struct was parsed
