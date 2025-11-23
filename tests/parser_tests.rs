use luma::ast::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

// Helper to strip spans from AST for comparison
fn strip_spans_program(mut prog: Program) -> Program {
    prog.statements = prog.statements.into_iter().map(strip_spans_stmt).collect();
    prog
}

fn strip_spans_stmt(stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::VarDecl {
            mutable,
            name,
            r#type,
            value,
            ..
        } => Stmt::VarDecl {
            mutable,
            name,
            r#type,
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::DestructuringVarDecl {
            mutable,
            pattern,
            value,
            ..
        } => Stmt::DestructuringVarDecl {
            mutable,
            pattern: strip_spans_pattern(pattern),
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::Assignment {
            target, op, value, ..
        } => Stmt::Assignment {
            target: strip_spans_expr(target),
            op,
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            ..
        } => Stmt::If {
            condition: strip_spans_expr(condition),
            then_block: then_block.into_iter().map(strip_spans_stmt).collect(),
            elif_blocks: elif_blocks
                .into_iter()
                .map(|(c, b)| {
                    (
                        strip_spans_expr(c),
                        b.into_iter().map(strip_spans_stmt).collect(),
                    )
                })
                .collect(),
            else_block: else_block.map(|b| b.into_iter().map(strip_spans_stmt).collect()),
            span: None,
        },
        Stmt::While {
            condition, body, ..
        } => Stmt::While {
            condition: strip_spans_expr(condition),
            body: body.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Stmt::DoWhile {
            body, condition, ..
        } => Stmt::DoWhile {
            body: body.into_iter().map(strip_spans_stmt).collect(),
            condition: strip_spans_expr(condition),
            span: None,
        },
        Stmt::For {
            pattern,
            iterator,
            body,
            ..
        } => Stmt::For {
            pattern: strip_spans_pattern(pattern),
            iterator: strip_spans_expr(iterator),
            body: body.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Stmt::Match { expr, arms, .. } => Stmt::Match {
            expr: strip_spans_expr(expr),
            arms: arms
                .into_iter()
                .map(|(p, b)| {
                    (
                        strip_spans_pattern(p),
                        b.into_iter().map(strip_spans_stmt).collect(),
                    )
                })
                .collect(),
            span: None,
        },
        Stmt::Return { value, .. } => Stmt::Return {
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::Break { level, .. } => Stmt::Break { level, span: None },
        Stmt::Continue { level, .. } => Stmt::Continue { level, span: None },
        Stmt::ExprStmt { expr, .. } => Stmt::ExprStmt {
            expr: strip_spans_expr(expr),
            span: None,
        },
    }
}

fn strip_spans_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Binary {
            left, op, right, ..
        } => Expr::Binary {
            left: Box::new(strip_spans_expr(*left)),
            op,
            right: Box::new(strip_spans_expr(*right)),
            span: None,
        },
        Expr::Unary { op, operand, .. } => Expr::Unary {
            op,
            operand: Box::new(strip_spans_expr(*operand)),
            span: None,
        },
        Expr::Logical {
            left, op, right, ..
        } => Expr::Logical {
            left: Box::new(strip_spans_expr(*left)),
            op,
            right: Box::new(strip_spans_expr(*right)),
            span: None,
        },
        Expr::Call {
            callee, arguments, ..
        } => Expr::Call {
            callee: Box::new(strip_spans_expr(*callee)),
            arguments: arguments
                .into_iter()
                .map(|a| match a {
                    CallArgument::Positional(e) => CallArgument::Positional(strip_spans_expr(e)),
                    CallArgument::Named { name, value } => CallArgument::Named {
                        name,
                        value: strip_spans_expr(value),
                    },
                })
                .collect(),
            span: None,
        },
        Expr::MemberAccess { object, member, .. } => Expr::MemberAccess {
            object: Box::new(strip_spans_expr(*object)),
            member,
            span: None,
        },
        Expr::Index { object, index, .. } => Expr::Index {
            object: Box::new(strip_spans_expr(*object)),
            index: Box::new(strip_spans_expr(*index)),
            span: None,
        },
        Expr::Function {
            arguments,
            return_type,
            body,
            ..
        } => Expr::Function {
            arguments,
            return_type,
            body: body.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Expr::If {
            condition,
            then_block,
            else_block,
            ..
        } => Expr::If {
            condition: Box::new(strip_spans_expr(*condition)),
            then_block: then_block.into_iter().map(strip_spans_stmt).collect(),
            else_block: else_block.map(|b| b.into_iter().map(strip_spans_stmt).collect()),
            span: None,
        },
        Expr::Match { expr, arms, .. } => Expr::Match {
            expr: Box::new(strip_spans_expr(*expr)),
            arms: arms
                .into_iter()
                .map(|(p, b)| {
                    (
                        strip_spans_pattern(p),
                        b.into_iter().map(strip_spans_stmt).collect(),
                    )
                })
                .collect(),
            span: None,
        },
        Expr::Block { statements, .. } => Expr::Block {
            statements: statements.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Expr::List { elements, .. } => Expr::List {
            elements: elements.into_iter().map(strip_spans_expr).collect(),
            span: None,
        },
        Expr::Table { fields, .. } => Expr::Table {
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, strip_spans_expr(v)))
                .collect(),
            span: None,
        },
        Expr::Import { path, .. } => Expr::Import { path, span: None },
        other => other,
    }
}

fn strip_spans_pattern(pat: Pattern) -> Pattern {
    match pat {
        Pattern::ListPattern { elements, rest, .. } => Pattern::ListPattern {
            elements: elements.into_iter().map(strip_spans_pattern).collect(),
            rest,
            span: None,
        },
        Pattern::TablePattern { fields, .. } => Pattern::TablePattern { fields, span: None },
        other => other,
    }
}

#[test]
fn test_parser_fixtures() {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    // Recursively collect .luma/.ron fixture pairs
    fn collect(dir: &Path, fixtures: &mut Vec<(PathBuf, PathBuf)>) {
        for entry in fs::read_dir(dir).expect("Failed to read fixtures directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                collect(&path, fixtures);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) == Some("luma") {
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let ron_path = path.parent().unwrap().join(format!("{}.ron", stem));
                if ron_path.exists() {
                    fixtures.push((path.clone(), ron_path));
                }
            }
        }
    }
    let mut fixtures = Vec::new();
    collect(&fixtures_dir, &mut fixtures);

    // Sort for consistent test ordering
    fixtures.sort_by(|a, b| a.0.cmp(&b.0));

    let mut failed_tests = Vec::new();

    for (luma_path, ron_path) in fixtures {
        let test_name = luma_path.file_stem().unwrap().to_str().unwrap();

        // Read the luma source
        let source_raw = fs::read_to_string(&luma_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", luma_path.display(), e));
        // Normalize Windows CRLF line endings for parser (treat CR as whitespace)
        let source = source_raw.replace("\r\n", "\n").replace("\r", "\n");

        // Read the expected RON output
        let expected_ron = fs::read_to_string(&ron_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", ron_path.display(), e));

        // Parse the source
        let ast = match luma::parser::parse(source.as_str(), luma_path.to_str().unwrap()) {
            Ok(ast) => ast,
            Err(errors) => {
                failed_tests.push(format!(
                    "❌ {}: Parse failed with errors:\n{}",
                    test_name,
                    errors
                        .iter()
                        .map(|e| format!("  {}", e))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
                continue;
            }
        };

        // Serialize to RON
        let actual_ron = ron::ser::to_string_pretty(&ast, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize AST to RON");

        // Parse both RON strings for comparison (to normalize formatting)
        let expected_ast: luma::ast::Program = ron::from_str(&expected_ron)
            .unwrap_or_else(|e| panic!("Failed to parse expected RON for {}: {}", test_name, e));

        // Strip spans from both ASTs before comparison
        let ast_without_spans = strip_spans_program(ast.clone());
        let expected_without_spans = strip_spans_program(expected_ast);

        if ast_without_spans != expected_without_spans {
            failed_tests.push(format!(
                "❌ {}: AST mismatch\nExpected:\n{}\n\nActual:\n{}\n",
                test_name, expected_ron, actual_ron
            ));
        } else {
            println!("✓ {}", test_name);
        }
    }

    if !failed_tests.is_empty() {
        panic!(
            "\n{} test(s) failed:\n\n{}",
            failed_tests.len(),
            failed_tests.join("\n")
        );
    }
}
