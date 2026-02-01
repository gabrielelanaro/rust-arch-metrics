use crate::models::StructInfo;

/// Calculate Lack of Cohesion in Methods (LCOM)
///
/// LCOM measures the degree to which methods in a class are related to each other.
/// Higher LCOM indicates lower cohesion (worse design).
///
/// This implementation uses the Henderson-Sellers variant:
/// LCOM = (m - sum(mA) / a) / (m - 1)
/// where:
/// - m = number of methods
/// - a = number of attributes (fields)
/// - mA = number of methods that access each attribute
///
/// # Arguments
/// * `struct_info` - The struct to analyze
///
/// # Returns
/// LCOM value between 0 and 1 (higher = less cohesive)
pub fn calculate(struct_info: &StructInfo) -> f64 {
    let method_count = struct_info.methods.len();
    let field_count = struct_info.fields.len();

    // Handle edge cases
    if method_count <= 1 || field_count == 0 {
        return 0.0; // Perfectly cohesive by definition
    }

    // Count how many methods access each field
    let mut field_access_counts: Vec<usize> = vec![0; field_count];

    for method in &struct_info.methods {
        for (idx, field) in struct_info.fields.iter().enumerate() {
            if method.fields_accessed.contains(&field.name) {
                field_access_counts[idx] += 1;
            }
        }
    }

    // Calculate sum of method accesses across all fields
    let sum_m_a: usize = field_access_counts.iter().sum();

    // Calculate average methods per attribute
    let avg_methods_per_attr = sum_m_a as f64 / field_count as f64;

    // Calculate LCOM using Henderson-Sellers formula
    let numerator = method_count as f64 - avg_methods_per_attr;
    let denominator = method_count as f64 - 1.0;

    let lcom = numerator / denominator;

    // Clamp between 0 and 1
    lcom.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FieldInfo, MethodInfo};

    #[test]
    fn test_lcom_perfectly_cohesive() {
        // All methods access the same field - perfectly cohesive
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
            ],
            external_types: vec![],
        };

        // Should be close to 0 (perfectly cohesive)
        let lcom = calculate(&struct_info);
        assert!(lcom < 0.1, "Expected low LCOM for cohesive struct, got {}", lcom);
    }

    #[test]
    fn test_lcom_low_cohesion() {
        // Methods access different fields - low cohesion
        let struct_info = StructInfo {
            name: "User".to_string(),
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    ty: "String".to_string(),
                },
                FieldInfo {
                    name: "email".to_string(),
                    ty: "String".to_string(),
                },
            ],
            methods: vec![
                MethodInfo {
                    fields_accessed: vec!["name".to_string()],
                    cyclomatic_complexity: 1,
                },
                MethodInfo {
                    fields_accessed: vec!["email".to_string()],
                    cyclomatic_complexity: 1,
                },
            ],
            external_types: vec![],
        };

        // Should be higher (less cohesive)
        let lcom = calculate(&struct_info);
        assert!(lcom > 0.5, "Expected high LCOM for low cohesion struct, got {}", lcom);
    }

    #[test]
    fn test_lcom_empty_struct() {
        let struct_info = StructInfo {
            name: "Empty".to_string(),
            fields: vec![],
            methods: vec![],
            external_types: vec![],
        };

        assert_eq!(calculate(&struct_info), 0.0);
    }
}
