pub mod cbo;
pub mod lcom;
pub mod wmc;

use crate::models::{AnalysisResult, StructInfo};

pub fn analyze_struct(struct_info: &StructInfo, all_structs: &[StructInfo]) -> AnalysisResult {
    AnalysisResult {
        struct_name: struct_info.name.clone(),
        lcom: lcom::calculate(struct_info),
        cbo: cbo::calculate(struct_info, all_structs),
        wmc: wmc::calculate(struct_info),
    }
}
