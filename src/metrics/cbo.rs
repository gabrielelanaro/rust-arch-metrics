use crate::models::StructInfo;

/// Calculate Coupling Between Objects (CBO)
///
/// CBO measures the number of other classes/structs that a class/struct
/// is coupled to (depends on). Higher CBO indicates more dependencies
/// and potentially harder to maintain code.
///
/// Coupling includes:
/// - Field types that are other structs
/// - Trait implementations
/// - Generic type parameters with trait bounds
///
/// # Arguments
/// * `struct_info` - The struct to analyze
/// * `all_structs` - All structs in the codebase for reference
///
/// # Returns
/// The number of distinct external types this struct depends on
pub fn calculate(struct_info: &StructInfo, all_structs: &[StructInfo]) -> usize {
    let mut coupled_types: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Collect all external types from the struct
    for ext_type in &struct_info.external_types {
        // Only count if it's another struct in our codebase
        if all_structs.iter().any(|s| s.name == *ext_type) {
            coupled_types.insert(ext_type.clone());
        }
    }

    // Collect types from field types
    for field in &struct_info.fields {
        // Check the base type and all generic type parameters
        let type_names = extract_all_types(&field.ty);
        for type_name in type_names {
            if all_structs.iter().any(|s| s.name == type_name) && type_name != struct_info.name {
                coupled_types.insert(type_name);
            }
        }
    }

    // Count trait implementations as coupling
    for trait_name in &struct_info.traits {
        coupled_types.insert(trait_name.clone());
    }

    coupled_types.len()
}

/// Extract all type names from a type string
/// e.g., ["String"] from "String", ["Vec", "Item"] from "Vec < Item >"
fn extract_all_types(ty: &str) -> Vec<String> {
    let mut types = Vec::new();
    let ty = ty.trim();

    // Handle reference types like &T, &mut T
    if ty.starts_with('&') {
        let inner = ty[1..].trim();
        if inner.starts_with("mut ") {
            return extract_all_types(&inner[4..]);
        }
        return extract_all_types(inner);
    }

    // Handle generic types like Vec<T>, Option<T>, HashMap<K, V>, etc.
    if let Some(start) = ty.find('<') {
        let base = ty[..start].trim().to_string();
        types.push(base);

        // Extract inner types from the generic parameters
        let end = ty.rfind('>').unwrap_or(ty.len());
        let inner = &ty[start + 1..end];

        // Split by comma to handle multiple type parameters like HashMap<K, V>
        for part in inner.split(',') {
            let inner_types = extract_all_types(part.trim());
            types.extend(inner_types);
        }
    } else {
        // Simple type
        types.push(ty.to_string());
    }

    types
}

/// Extract the base type name from a type string (deprecated, use extract_all_types)
#[allow(dead_code)]
fn extract_type_name(ty: &str) -> Option<String> {
    extract_all_types(ty).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FieldInfo;

    #[test]
    fn test_cbo_no_coupling() {
        let struct_a = StructInfo {
            name: "User".to_string(),
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    ty: "String".to_string(),
                },
            ],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        let all_structs = vec![struct_a.clone()];

        assert_eq!(calculate(&struct_a, &all_structs), 0);
    }

    #[test]
    fn test_cbo_with_coupling() {
        let struct_a = StructInfo {
            name: "User".to_string(),
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    ty: "String".to_string(),
                },
                FieldInfo {
                    name: "address".to_string(),
                    ty: "Address".to_string(),
                },
            ],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        let struct_b = StructInfo {
            name: "Address".to_string(),
            fields: vec![
                FieldInfo {
                    name: "street".to_string(),
                    ty: "String".to_string(),
                },
            ],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        let all_structs = vec![struct_a.clone(), struct_b];

        // User is coupled to Address
        assert_eq!(calculate(&struct_a, &all_structs), 1);
    }

    #[test]
    fn test_cbo_multiple_couplings() {
        let struct_a = StructInfo {
            name: "Order".to_string(),
            fields: vec![
                FieldInfo {
                    name: "user".to_string(),
                    ty: "User".to_string(),
                },
                FieldInfo {
                    name: "product".to_string(),
                    ty: "Product".to_string(),
                },
            ],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        let struct_b = StructInfo {
            name: "User".to_string(),
            fields: vec![],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        let struct_c = StructInfo {
            name: "Product".to_string(),
            fields: vec![],
            methods: vec![],
            external_types: vec![],
            traits: vec![],
        };

        let all_structs = vec![struct_a.clone(), struct_b, struct_c];

        // Order is coupled to both User and Product
        assert_eq!(calculate(&struct_a, &all_structs), 2);
    }

    #[test]
    fn test_extract_type_name() {
        assert_eq!(extract_type_name("String"), Some("String".to_string()));
        assert_eq!(extract_type_name("Vec<String>"), Some("Vec".to_string()));
        assert_eq!(extract_type_name("&str"), Some("str".to_string()));
        assert_eq!(extract_type_name("&mut String"), Some("String".to_string()));
    }
}
