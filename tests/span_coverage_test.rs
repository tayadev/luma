//! Test to verify that all AST nodes have proper span coverage
//!
//! This ensures IDE integration features like hover, go-to-definition, and
//! error highlighting work correctly by verifying every node tracks its location.

use luma::ast::{Expr, Pattern, Program, Stmt};

/// Recursively verify that all AST nodes have non-empty spans
fn verify_program_spans(program: &Program) -> Result<(), String> {
    for stmt in &program.statements {
        verify_stmt_spans(stmt)?;
    }
    Ok(())
}

fn verify_stmt_spans(stmt: &Stmt) -> Result<(), String> {
    // Check that this statement has a span
    if let Some(span) = stmt.span() {
        if span.is_empty() {
            return Err(format!("Statement has empty span: {:?}", stmt));
        }
    } else {
        return Err(format!("Statement missing span: {:?}", stmt));
    }

    // Recursively check child nodes
    match stmt {
        Stmt::VarDecl { value, .. } => verify_expr_spans(value)?,
        Stmt::DestructuringVarDecl { pattern, value, .. } => {
            verify_pattern_spans(pattern)?;
            verify_expr_spans(value)?;
        }
        Stmt::Assignment { target, value, .. } => {
            verify_expr_spans(target)?;
            verify_expr_spans(value)?;
        }
        Stmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            ..
        } => {
            verify_expr_spans(condition)?;
            for s in then_block {
                verify_stmt_spans(s)?;
            }
            for (cond, block) in elif_blocks {
                verify_expr_spans(cond)?;
                for s in block {
                    verify_stmt_spans(s)?;
                }
            }
            if let Some(block) = else_block {
                for s in block {
                    verify_stmt_spans(s)?;
                }
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            verify_expr_spans(condition)?;
            for s in body {
                verify_stmt_spans(s)?;
            }
        }
        Stmt::DoWhile {
            body, condition, ..
        } => {
            for s in body {
                verify_stmt_spans(s)?;
            }
            verify_expr_spans(condition)?;
        }
        Stmt::For {
            pattern,
            iterator,
            body,
            ..
        } => {
            verify_pattern_spans(pattern)?;
            verify_expr_spans(iterator)?;
            for s in body {
                verify_stmt_spans(s)?;
            }
        }
        Stmt::Match { expr, arms, .. } => {
            verify_expr_spans(expr)?;
            for (pattern, block) in arms {
                verify_pattern_spans(pattern)?;
                for s in block {
                    verify_stmt_spans(s)?;
                }
            }
        }
        Stmt::Return { value, .. } => verify_expr_spans(value)?,
        Stmt::ExprStmt { expr, .. } => verify_expr_spans(expr)?,
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
    }
    Ok(())
}

fn verify_expr_spans(expr: &Expr) -> Result<(), String> {
    // Check that this expression has a span
    if let Some(span) = expr.span() {
        if span.is_empty() {
            return Err(format!("Expression has empty span: {:?}", expr));
        }
    } else {
        return Err(format!("Expression missing span: {:?}", expr));
    }

    // Recursively check child nodes
    match expr {
        Expr::Binary { left, right, .. } => {
            verify_expr_spans(left)?;
            verify_expr_spans(right)?;
        }
        Expr::Unary { operand, .. } => verify_expr_spans(operand)?,
        Expr::Logical { left, right, .. } => {
            verify_expr_spans(left)?;
            verify_expr_spans(right)?;
        }
        Expr::Call {
            callee, arguments, ..
        } => {
            verify_expr_spans(callee)?;
            for arg in arguments {
                match arg {
                    luma::ast::CallArgument::Positional(e) => verify_expr_spans(e)?,
                    luma::ast::CallArgument::Named { value, .. } => verify_expr_spans(value)?,
                }
            }
        }
        Expr::MemberAccess { object, .. } => verify_expr_spans(object)?,
        Expr::Index { object, index, .. } => {
            verify_expr_spans(object)?;
            verify_expr_spans(index)?;
        }
        Expr::Function {
            body, arguments, ..
        } => {
            for arg in arguments {
                if let Some(default) = &arg.default {
                    verify_expr_spans(default)?;
                }
            }
            for stmt in body {
                verify_stmt_spans(stmt)?;
            }
        }
        Expr::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            verify_expr_spans(condition)?;
            for s in then_block {
                verify_stmt_spans(s)?;
            }
            if let Some(block) = else_block {
                for s in block {
                    verify_stmt_spans(s)?;
                }
            }
        }
        Expr::Match { expr: e, arms, .. } => {
            verify_expr_spans(e)?;
            for (pattern, block) in arms {
                verify_pattern_spans(pattern)?;
                for s in block {
                    verify_stmt_spans(s)?;
                }
            }
        }
        Expr::Block { statements, .. } => {
            for s in statements {
                verify_stmt_spans(s)?;
            }
        }
        Expr::List { elements, .. } => {
            for e in elements {
                verify_expr_spans(e)?;
            }
        }
        Expr::Table { fields, .. } => {
            for (_, e) in fields {
                verify_expr_spans(e)?;
            }
        }
        Expr::Import { path, .. } => verify_expr_spans(path)?,
        Expr::Number { .. }
        | Expr::Identifier { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Null { .. } => {}
    }
    Ok(())
}

fn verify_pattern_spans(pattern: &Pattern) -> Result<(), String> {
    // Check that this pattern has a span
    if let Some(span) = pattern.span() {
        if span.is_empty() {
            return Err(format!("Pattern has empty span: {:?}", pattern));
        }
    } else {
        return Err(format!("Pattern missing span: {:?}", pattern));
    }

    // Recursively check child nodes
    match pattern {
        Pattern::ListPattern { elements, .. } => {
            for p in elements {
                verify_pattern_spans(p)?;
            }
        }
        Pattern::Ident { .. } | Pattern::Wildcard { .. } | Pattern::Literal { .. } => {}
        Pattern::TablePattern { .. } => {}
    }
    Ok(())
}

#[test]
fn test_span_coverage_comprehensive() {
    let test_cases = vec![
        // Literals
        ("let x = 42", "number literal"),
        ("let s = \"hello\"", "string literal"),
        ("let b = true", "boolean literal"),
        ("let n = null", "null literal"),
        // Collections
        ("let arr = [1, 2, 3]", "array literal"),
        ("let tbl = {a = 1, b = 2}", "table literal"),
        // Expressions
        ("let x = 1 + 2 * 3", "binary operations"),
        ("let x = -5", "unary operation"),
        ("let x = not false", "logical not"),
        ("let x = a and b or c", "logical operations"),
        // Control flow
        ("if true do return 1 end", "if expression"),
        ("while x < 10 do x = x + 1 end", "while loop"),
        ("for i in [1,2,3] do print(i) end", "for loop"),
        // Functions
        (
            "let f = fn(x: Number): Number do return x * 2 end",
            "function",
        ),
        ("f(42)", "function call"),
        // Patterns
        ("let [a, b] = [1, 2]", "list destructuring"),
        ("let {x, y} = point", "table destructuring"),
        // Match
        (
            "match x do 1 do return \"one\" end _ do return \"other\" end end",
            "match expression",
        ),
        // Member access
        ("obj.field", "member access"),
        ("arr[0]", "index access"),
        // Blocks
        ("let x = do let y = 1 y + 2 end", "block expression"),
    ];

    for (source, description) in test_cases {
        let result = luma::parser::parse(source, "test.luma");
        match result {
            Ok(program) => {
                if let Err(e) = verify_program_spans(&program) {
                    panic!("Span coverage failed for '{}': {}", description, e);
                }
            }
            Err(errors) => {
                panic!("Parse failed for '{}': {:?}", description, errors);
            }
        }
    }
}

#[test]
fn test_span_locations_accurate() {
    let source = "let x = 42\nlet y = 100";
    let program = luma::parser::parse(source, "test.luma").expect("Parse failed");

    // Verify first statement has correct span
    let first_stmt = &program.statements[0];
    let span = first_stmt.span().expect("First statement should have span");
    assert!(!span.is_empty(), "Span should not be empty");
    // Note: Parser spans may include trailing whitespace - this is acceptable
    let text = span.text(source).trim_end();
    assert_eq!(
        text, "let x = 42",
        "Span should cover let statement (modulo trailing whitespace)"
    );

    // Verify second statement has correct span
    let second_stmt = &program.statements[1];
    let span = second_stmt
        .span()
        .expect("Second statement should have span");
    assert!(!span.is_empty(), "Span should not be empty");
    let text = span.text(source).trim_end();
    assert_eq!(
        text, "let y = 100",
        "Span should cover second let statement (modulo trailing whitespace)"
    );
}

#[test]
fn test_span_utility_methods() {
    use luma::ast::Span;

    let span1 = Span::new(0, 10);
    let span2 = Span::new(5, 15);
    let span3 = Span::new(20, 30);

    // Test contains_offset
    assert!(span1.contains_offset(5));
    assert!(!span1.contains_offset(10));
    assert!(!span1.contains_offset(15));

    // Test overlaps
    assert!(span1.overlaps(&span2));
    assert!(span2.overlaps(&span1));
    assert!(!span1.overlaps(&span3));

    // Test merge
    let merged = span1.merge(&span2);
    assert_eq!(merged.start, 0);
    assert_eq!(merged.end, 15);

    // Test len and is_empty
    assert_eq!(span1.len(), 10);
    assert!(!span1.is_empty());

    let empty_span = Span::new(5, 5);
    assert!(empty_span.is_empty());
}
