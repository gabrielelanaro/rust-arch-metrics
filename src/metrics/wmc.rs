use crate::models::StructInfo;

/// Calculate Weighted Methods per Class (WMC)
///
/// WMC is the sum of the complexities of all methods in a class.
/// For simplicity, we use cyclomatic complexity as the weight.
///
/// # Arguments
/// * `struct_info` - The struct to analyze
///
/// # Returns
/// The total weighted method count
pub fn calculate(struct_info: &StructInfo) -> usize {
    struct_info
        .methods
        .iter()
        .map(|m| m.cyclomatic_complexity.max(1))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FieldInfo, MethodInfo};

    #[test]
    fn test_wmc_empty_struct() {
        let struct_info = StructInfo {
            name: "Empty".to_string(),
            fields: vec![],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        assert_eq!(calculate(&struct_info), 0);
    }

    #[test]
    fn test_wmc_with_methods() {
        let struct_info = StructInfo {
            name: "User".to_string(),
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    ty: "String".to_string(),
                },
            ],
            methods: vec![
                MethodInfo {
                    fields_accessed: vec!["name".to_string()],
                    cyclomatic_complexity: 1,
                },
                MethodInfo {
                    fields_accessed: vec!["name".to_string()],
                    cyclomatic_complexity: 1,
                },
                MethodInfo {
                    fields_accessed: vec![],
                    cyclomatic_complexity: 3,
                },
            ],
            external_types: vec![],
            traits: vec![],
        };

        assert_eq!(calculate(&struct_info), 5); // 1 + 1 + 3
    }
}
