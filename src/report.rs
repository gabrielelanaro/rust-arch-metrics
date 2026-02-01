use crate::models::{AnalysisResult, OutputFormat};

pub fn generate_report(
    results: &[AnalysisResult],
    format: OutputFormat,
    output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match format {
        OutputFormat::Table => generate_table(results),
        OutputFormat::Json => generate_json(results)?,
        OutputFormat::Csv => generate_csv(results)?,
    };

    if let Some(file_path) = output {
        std::fs::write(file_path, content)?;
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn generate_table(results: &[AnalysisResult]) -> String {
    if results.is_empty() {
        return "No structs found to analyze.".to_string();
    }

    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{:<30} {:>10} {:>10} {:>10}\n",
        "Struct Name", "LCOM", "CBO", "WMC"
    ));
    output.push_str(&"-".repeat(62));
    output.push('\n');

    // Rows
    for result in results {
        output.push_str(&format!(
            "{:<30} {:>10.3} {:>10} {:>10}\n",
            result.struct_name, result.lcom, result.cbo, result.wmc
        ));
    }

    // Summary
    output.push('\n');
    output.push_str("Metric Explanations:\n");
    output.push_str("  LCOM (0-1): Lack of Cohesion in Methods (lower is better)\n");
    output.push_str("  CBO:        Coupling Between Objects (lower is better)\n");
    output.push_str("  WMC:        Weighted Methods per Class (complexity)\n");

    output
}

fn generate_json(results: &[AnalysisResult]) -> Result<String, serde_json::Error> {
    #[derive(serde::Serialize)]
    struct JsonResult {
        struct_name: String,
        lcom: f64,
        cbo: usize,
        wmc: usize,
    }

    let json_results: Vec<JsonResult> = results
        .iter()
        .map(|r| JsonResult {
            struct_name: r.struct_name.clone(),
            lcom: r.lcom,
            cbo: r.cbo,
            wmc: r.wmc,
        })
        .collect();

    serde_json::to_string_pretty(&json_results)
}

fn generate_csv(results: &[AnalysisResult]) -> Result<String, csv::Error> {
    let mut writer = csv::Writer::from_writer(Vec::new());

    // Header
    writer.write_record(["struct_name", "lcom", "cbo", "wmc"])?;

    // Data
    for result in results {
        writer.write_record([
            &result.struct_name,
            &result.lcom.to_string(),
            &result.cbo.to_string(),
            &result.wmc.to_string(),
        ])?;
    }

    writer.flush()?;
    let inner = writer.into_inner().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("CSV error: {:?}", e))
    })?;
    let data = String::from_utf8(inner).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    Ok(data)
}
