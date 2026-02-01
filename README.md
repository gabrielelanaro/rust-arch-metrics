# rust-arch-metrics

A Rust CLI tool that calculates architectural metrics for Rust codebases. It analyzes Rust source files and computes three key object-oriented metrics for each struct to help identify architectural issues like God Classes, Feature Envy, and low cohesion.

## Metrics

| Metric | Description | Range | Interpretation |
|--------|-------------|-------|----------------|
| **LCOM** | Lack of Cohesion in Methods | 0.0 - 1.0 (lower is better) | Measures how closely related methods are within a struct |
| **CBO** | Coupling Between Objects | 0+ (lower is better) | Counts dependencies on other structs in the codebase |
| **WMC** | Weighted Methods per Class | 0+ (lower is better) | Sum of cyclomatic complexities across all methods |

### LCOM (Lack of Cohesion in Methods)

Measures how closely related methods are to each other within a struct using the Henderson-Sellers formula.

- **0.0** = Perfect cohesion - all methods use all fields
- **0.0-0.5** = Good cohesion - methods work on related field subsets
- **0.5-0.8** = Low cohesion - may indicate multiple responsibilities
- **1.0** = No cohesion - methods share no fields (consider splitting)

### CBO (Coupling Between Objects)

Counts dependencies on other structs defined in the analyzed codebase. External types like `String` or `Vec` are not counted.

- **0-2** = Low coupling, easy to test and reuse
- **3-5** = Moderate coupling, acceptable
- **6+** = High coupling, difficult to maintain

### WMC (Weighted Methods per Class)

Sum of cyclomatic complexities across all methods. Complexity is calculated as 1 + number of branches (if, match, while, for, loop).

- **0-10** = Simple, easy to understand
- **11-20** = Moderate complexity
- **21-40** = Complex, consider refactoring
- **40+** = God class, needs decomposition

## Installation

### From Source

```bash
git clone https://github.com/gabrielelanaro/rust-arch-metrics.git
cd rust-arch-metrics
cargo build --release
```

The binary will be available at `target/release/rust-arch-metrics`.

### Prerequisites

- Rust 1.70+ (for building from source)

## Usage

```bash
rust-arch-metrics [OPTIONS] <PATH>
```

### Arguments

- `<PATH>` - Path to the Rust project directory or single .rs file to analyze

### Options

| Option | Description |
|--------|-------------|
| `-f, --format <FORMAT>` | Output format: `table` (default), `json`, or `csv` |
| `-o, --output <FILE>` | Write output to file instead of stdout |
| `-m, --metrics <METRICS>` | Metrics to calculate: `lcom`, `cbo`, `wmc` or `all` (default) |
| `--exclude <PATTERN>` | Skip files/directories matching this substring |
| `--debug-struct <STRUCT_NAME>` | Print detailed parsing info for a specific struct |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

### Examples

```bash
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
```

## Example Output

### Table Format (default)

```
+---------------+------+-----+-----+
| Struct        | LCOM | CBO | WMC |
+---------------+------+-----+-----+
| Parser        | 0.25 |   3 |  12 |
| Analyzer      | 0.50 |   2 |   8 |
| ReportGenerator| 0.00 |   1 |   5 |
+---------------+------+-----+-----+
```

### JSON Format

```json
[
  {
    "struct_name": "Parser",
    "lcom": 0.25,
    "cbo": 3,
    "wmc": 12
  }
]
```

## How It Works

The tool uses the [`syn`](https://docs.rs/syn) crate to parse Rust source files and extract:

- Struct definitions and their fields
- Methods (inherent impl blocks and trait implementations)
- Field access patterns (`self.field` references)
- Cyclomatic complexity by counting branches
- External type dependencies

The parser only analyzes struct impl blocks, not free functions.

## Development

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run tests
cargo test

# Run on the current codebase
cargo run -- src/

# Run with JSON output
cargo run -- src/ --format json
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## References

- [Lack of Cohesion in Methods (Wikipedia)](https://en.wikipedia.org/wiki/Lack_of_cohesion_in_methods)
- [Coupling (computer programming) (Wikipedia)](https://en.wikipedia.org/wiki/Coupling_(computer_programming))
- Henderson-Sellers, B. (1996). *Object-Oriented Metrics: Measures of Complexity*. Prentice Hall.
- Chidamber, S. R., & Kemerer, C. F. (1994). A metrics suite for object oriented design. *IEEE Transactions on Software Engineering*, 20(6), 476-493.
