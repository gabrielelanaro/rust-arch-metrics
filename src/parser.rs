use std::collections::HashSet;
use syn::{visit::Visit, File, ItemStruct, ItemImpl, ImplItemFn};
use crate::models::{FieldInfo, MethodInfo, StructInfo};

pub struct StructVisitor {
    pub structs: Vec<StructInfo>,
    current_struct: Option<String>,
}

impl StructVisitor {
    pub fn new() -> Self {
        Self {
            structs: Vec::new(),
            current_struct: None,
        }
    }
}

impl<'ast> Visit<'ast> for StructVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let struct_name = node.ident.to_string();
        let mut fields = Vec::new();

        for field in &node.fields {
            if let Some(ident) = &field.ident {
                fields.push(FieldInfo {
                    name: ident.to_string(),
                    ty: quote::quote!(#field.ty).to_string(),
                });
            }
        }

        self.structs.push(StructInfo {
            name: struct_name.clone(),
            fields,
            methods: Vec::new(),
            external_types: Vec::new(),
        });

        self.current_struct = Some(struct_name);
        syn::visit::visit_item_struct(self, node);
        self.current_struct = None;
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if let Some((_, _path, _)) = &node.trait_ {
            // Trait implementation - skip for now
            return;
        }

        if let syn::Type::Path(type_path) = &*node.self_ty {
            if let Some(seg) = type_path.path.segments.last() {
                let struct_name = seg.ident.to_string();

                // Find the struct in our list
                if let Some(struct_info) = self.structs.iter_mut().find(|s| s.name == struct_name) {
                    for item in &node.items {
                        if let syn::ImplItem::Fn(method) = item {
                            let method_info = analyze_method(method, struct_info);
                            struct_info.methods.push(method_info);
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_impl(self, node);
    }
}

fn analyze_method(method: &ImplItemFn, struct_info: &StructInfo) -> MethodInfo {
    let name = method.sig.ident.to_string();
    let mut fields_accessed = HashSet::new();
    let mut external_types = HashSet::new();

    // Analyze method body for field access
    analyze_expr(&method.block, struct_info, &mut fields_accessed, &mut external_types);

    // Calculate cyclomatic complexity (basic version)
    let cyclomatic_complexity = calculate_cyclomatic_complexity(&method.block);

    MethodInfo {
        name,
        fields_accessed: fields_accessed.into_iter().collect(),
        cyclomatic_complexity,
    }
}

fn analyze_expr(
    expr: &syn::Block,
    struct_info: &StructInfo,
    fields_accessed: &mut HashSet<String>,
    external_types: &mut HashSet<String>,
) {
    for stmt in &expr.stmts {
        analyze_stmt(stmt, struct_info, fields_accessed, external_types);
    }
}

fn analyze_stmt(
    stmt: &syn::Stmt,
    struct_info: &StructInfo,
    fields_accessed: &mut HashSet<String>,
    external_types: &mut HashSet<String>,
) {
    match stmt {
        syn::Stmt::Local(local) => {
            if let Some(init) = &local.init {
                analyze_expr_expr(&init.expr, struct_info, fields_accessed, external_types);
            }
        }
        syn::Stmt::Expr(expr, _) => {
            analyze_expr_expr(expr, struct_info, fields_accessed, external_types);
        }
        _ => {}
    }
}

fn analyze_expr_expr(
    expr: &syn::Expr,
    struct_info: &StructInfo,
    fields_accessed: &mut HashSet<String>,
    external_types: &mut HashSet<String>,
) {
    match expr {
        syn::Expr::Field(field_expr) => {
            // Check if accessing self.field
            if let syn::Expr::Path(path) = &*field_expr.base {
                if path.path.is_ident("self") {
                    if let syn::Member::Named(ident) = &field_expr.member {
                        fields_accessed.insert(ident.to_string());
                    }
                }
            }
        }
        syn::Expr::MethodCall(call) => {
            analyze_expr_expr(&call.receiver, struct_info, fields_accessed, external_types);
            for arg in &call.args {
                analyze_expr_expr(arg, struct_info, fields_accessed, external_types);
            }
        }
        syn::Expr::Call(call) => {
            analyze_expr_expr(&call.func, struct_info, fields_accessed, external_types);
            for arg in &call.args {
                analyze_expr_expr(arg, struct_info, fields_accessed, external_types);
            }
        }
        syn::Expr::Binary(bin) => {
            analyze_expr_expr(&bin.left, struct_info, fields_accessed, external_types);
            analyze_expr_expr(&bin.right, struct_info, fields_accessed, external_types);
        }
        syn::Expr::Unary(unary) => {
            analyze_expr_expr(&unary.expr, struct_info, fields_accessed, external_types);
        }
        syn::Expr::Reference(ref_expr) => {
            analyze_expr_expr(&ref_expr.expr, struct_info, fields_accessed, external_types);
        }
        syn::Expr::Block(block) => {
            analyze_expr(&block.block, struct_info, fields_accessed, external_types);
        }
        syn::Expr::If(if_expr) => {
            analyze_expr_expr(&if_expr.cond, struct_info, fields_accessed, external_types);
            analyze_expr(&if_expr.then_branch, struct_info, fields_accessed, external_types);
            if let Some((_, else_branch)) = &if_expr.else_branch {
                analyze_expr_expr(else_branch, struct_info, fields_accessed, external_types);
            }
        }
        syn::Expr::While(while_expr) => {
            analyze_expr_expr(&while_expr.cond, struct_info, fields_accessed, external_types);
            analyze_expr(&while_expr.body, struct_info, fields_accessed, external_types);
        }
        syn::Expr::ForLoop(for_expr) => {
            analyze_expr_expr(&for_expr.expr, struct_info, fields_accessed, external_types);
            analyze_expr(&for_expr.body, struct_info, fields_accessed, external_types);
        }
        syn::Expr::Match(match_expr) => {
            analyze_expr_expr(&match_expr.expr, struct_info, fields_accessed, external_types);
            for arm in &match_expr.arms {
                if let Some((_, guard)) = &arm.guard {
                    analyze_expr_expr(guard, struct_info, fields_accessed, external_types);
                }
                analyze_expr_expr(&arm.body, struct_info, fields_accessed, external_types);
            }
        }
        syn::Expr::Struct(struct_expr) => {
            let type_name = quote::quote!(#struct_expr.path).to_string();
            if !struct_info.fields.iter().any(|f| type_name.contains(&f.name)) {
                external_types.insert(type_name);
            }
            for field in &struct_expr.fields {
                analyze_expr_expr(&field.expr, struct_info, fields_accessed, external_types);
            }
        }
        syn::Expr::Path(path) => {
            let path_str = quote::quote!(#path).to_string();
            // Check if it's a type that might be external
            if path_str.contains("::") && !path_str.starts_with("self") && !path_str.starts_with("crate") {
                external_types.insert(path_str);
            }
        }
        _ => {}
    }
}

fn calculate_cyclomatic_complexity(block: &syn::Block) -> usize {
    let mut complexity = 1; // Base complexity

    for stmt in &block.stmts {
        complexity += stmt_complexity(stmt);
    }

    complexity
}

fn stmt_complexity(stmt: &syn::Stmt) -> usize {
    match stmt {
        syn::Stmt::Expr(expr, _) => expr_complexity(expr),
        syn::Stmt::Local(local) => {
            if let Some(init) = &local.init {
                expr_complexity(&init.expr)
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn expr_complexity(expr: &syn::Expr) -> usize {
    match expr {
        syn::Expr::If(if_expr) => {
            let mut complexity = 1; // if statement
            complexity += expr_complexity(&if_expr.cond);
            for stmt in &if_expr.then_branch.stmts {
                complexity += stmt_complexity(stmt);
            }
            if let Some((_, else_branch)) = &if_expr.else_branch {
                complexity += expr_complexity(else_branch);
            }
            complexity
        }
        syn::Expr::Match(_) => 1, // match statement
        syn::Expr::While(_) => 1, // while loop
        syn::Expr::ForLoop(_) => 1, // for loop
        syn::Expr::Loop(_) => 1, // loop
        syn::Expr::Block(block) => {
            let mut complexity = 0;
            for stmt in &block.block.stmts {
                complexity += stmt_complexity(stmt);
            }
            complexity
        }
        _ => 0,
    }
}

pub fn parse_file(content: &str) -> Result<Vec<StructInfo>, syn::Error> {
    let file: File = syn::parse_str(content)?;
    let mut visitor = StructVisitor::new();
    visitor.visit_file(&file);
    Ok(visitor.structs)
}

pub fn extract_external_types(content: &str) -> Result<HashSet<String>, syn::Error> {
    let file: File = syn::parse_str(content)?;
    let mut types = HashSet::new();

    for item in &file.items {
        match item {
            syn::Item::Use(use_item) => {
                extract_types_from_use(&use_item.tree, &mut types);
            }
            _ => {}
        }
    }

    Ok(types)
}

fn extract_types_from_use(tree: &syn::UseTree, types: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Path(path) => {
            extract_types_from_use(&path.tree, types);
        }
        syn::UseTree::Name(name) => {
            types.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            types.insert(rename.rename.to_string());
        }
        syn::UseTree::Glob(_) => {}
        syn::UseTree::Group(group) => {
            for item in &group.items {
                extract_types_from_use(item, types);
            }
        }
    }
}
