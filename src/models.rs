/// Represents information about a struct field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
}

/// Represents information about a method
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub fields_accessed: Vec<String>,
    pub cyclomatic_complexity: usize,
}

/// Represents information about a struct and its methods
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub external_types: Vec<String>,
}

/// Represents the analysis result for a struct
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub struct_name: String,
    pub lcom: f64,
    pub cbo: usize,
    pub wmc: usize,
}

/// Output format options
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}
