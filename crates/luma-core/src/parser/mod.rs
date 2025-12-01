use crate::ast::{CallArgument, Expr, Program, Stmt};
use crate::diagnostics::Diagnostic;
use chumsky::prelude::*;

mod errors;
mod expressions;
mod lexer;
mod literals;
mod operators;
mod patterns;
pub mod recovery;
mod statements;
mod string;
mod types;
mod utils;

use string::string_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Program, extra::Err<Rich<'a, char>>> {
    // Comments and whitespace
    let ws = lexer::ws();

    let ident = text::ident()
        .try_map(move |s: &str, span| {
            if lexer::KEYWORDS.contains(&s) {
                Err(Rich::custom(
                    span,
                    format!("'{s}' is a keyword and cannot be used as an identifier"),
                ))
            } else {
                Ok(s)
            }
        })
        .padded_by(ws.clone());

    // Type parser
    let type_parser = types::type_parser(ws.clone());

    let number = literals::number(ws.clone());

    // Recursive expression and statement placeholders
    let mut expr_ref = Recursive::declare();
    let mut stmt_ref = Recursive::declare();

    // Boolean and Null literals
    let boolean = literals::boolean(ws.clone());
    let null = literals::null(ws.clone());

    // List and Table literals
    let list = literals::list(ws.clone(), expr_ref.clone());
    let table = literals::table(
        ws.clone(),
        ident.clone(),
        expr_ref.clone(),
        string_parser(ws.clone(), expr_ref.clone()).boxed(),
        type_parser.clone(),
    );

    // Pattern parsing for destructuring
    let pattern = patterns::pattern(ws.clone(), ident.clone());

    // Expression parsers (blocks, functions, and if expressions)
    let block_expr = expressions::block(ws.clone(), stmt_ref.clone(), expr_ref.clone());
    let function = expressions::function(
        ws.clone(),
        ident.clone(),
        type_parser.clone(),
        stmt_ref.clone(),
        expr_ref.clone(),
    );
    let if_expr = expressions::if_expr(ws.clone(), stmt_ref.clone(), expr_ref.clone());
    let import_expr = expressions::import(ws.clone(), expr_ref.clone());
    let match_expression = expressions::match_expr(
        ws.clone(),
        expr_ref.clone(),
        stmt_ref.clone(),
        pattern.clone(),
    );

    // Parenthesized expressions - allows precedence override
    let paren_expr = expr_ref
        .clone()
        .delimited_by(
            just('(').padded_by(ws.clone()),
            just(')').padded_by(ws.clone()),
        )
        .boxed();

    // Primary expressions (atoms)
    let primary = choice((
        number.boxed(),
        string_parser(ws.clone(), expr_ref.clone()).boxed(),
        boolean,
        null,
        list,
        table,
        if_expr,
        block_expr,
        match_expression,
        function,
        import_expr,
        paren_expr,
        ident
            .clone()
            .try_map(|s: &str, span| {
                Ok(Expr::Identifier {
                    name: s.to_string(),
                    span: Some(crate::ast::Span::from_chumsky(span)),
                })
            })
            .boxed(),
    ))
    .boxed();

    // Postfix operators: function calls, member access, and indexing
    // Parse call arguments - can be positional (expr) or named (name = expr)
    let call_arg = {
        let named = ident
            .clone()
            .then_ignore(just('=').padded_by(ws.clone()))
            .then(expr_ref.clone())
            .map(|(name, value): (&str, Expr)| CallArgument::Named {
                name: name.to_string(),
                value,
            });

        let positional = expr_ref.clone().map(CallArgument::Positional);

        choice((named, positional))
    };

    let call_args = call_arg
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<CallArgument>>()
        .delimited_by(
            just('(').padded_by(ws.clone()),
            just(')').padded_by(ws.clone()),
        );

    let member_access = just('.').padded_by(ws.clone()).ignore_then(ident.clone());

    let method_call = just(':')
        .padded_by(ws.clone())
        .ignore_then(ident.clone())
        .then(call_args.clone());

    let index = expr_ref.clone().delimited_by(
        just('[').padded_by(ws.clone()),
        just(']').padded_by(ws.clone()),
    );

    #[derive(Clone)]
    enum PostfixOp {
        Call(Vec<CallArgument>),
        Member(String),
        MethodCall(String, Vec<CallArgument>),
        Index(Box<Expr>),
    }

    let postfix_op = choice((
        method_call.map(|(method, args): (&str, Vec<CallArgument>)| {
            PostfixOp::MethodCall(method.to_string(), args)
        }),
        call_args.map(PostfixOp::Call),
        member_access.map(|m: &str| PostfixOp::Member(m.to_string())),
        index.map(|e| PostfixOp::Index(Box::new(e))),
    ));

    let postfix = primary
        .clone()
        .then(postfix_op.repeated().collect::<Vec<_>>())
        .try_map(|(mut expr, ops), span| {
            let start = span.start;
            for op in ops {
                let expr_span = expr.span().map(|s| s.start).unwrap_or(start);
                expr = match op {
                    PostfixOp::Call(arguments) => Expr::Call {
                        callee: Box::new(expr),
                        arguments,
                        span: Some(crate::ast::Span::new(expr_span, span.end)),
                    },
                    PostfixOp::Member(member) => Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        span: Some(crate::ast::Span::new(expr_span, span.end)),
                    },
                    PostfixOp::MethodCall(method, arguments) => Expr::MethodCall {
                        object: Box::new(expr),
                        method,
                        arguments,
                        span: Some(crate::ast::Span::new(expr_span, span.end)),
                    },
                    PostfixOp::Index(index) => Expr::Index {
                        object: Box::new(expr),
                        index,
                        span: Some(crate::ast::Span::new(expr_span, span.end)),
                    },
                };
            }
            Ok(expr)
        })
        .boxed();

    // Unary operators (not, -)
    // These have lower precedence than postfix, so they operate on postfix expressions
    let unary_op = operators::unary_op(ws.clone());
    let unary_expr = unary_op
        .repeated()
        .collect::<Vec<_>>()
        .then(postfix.clone())
        .try_map(|(ops, mut operand), span| {
            // Apply operators right-to-left
            for op in ops.into_iter().rev() {
                let operand_span = operand.span().map(|s| s.end).unwrap_or(span.end);
                operand = Expr::Unary {
                    op,
                    operand: Box::new(operand),
                    span: Some(crate::ast::Span::new(span.start, operand_span)),
                };
            }
            Ok(operand)
        })
        .boxed();

    // Binary operators with precedence
    let mul_op = operators::mul_op(ws.clone());
    let add_op = operators::add_op(ws.clone());
    let cmp_op = operators::cmp_op(ws.clone());
    let eq_op = operators::eq_op(ws.clone());
    let and_op = operators::and_op(ws.clone());
    let or_op = operators::or_op(ws.clone());

    // Build expression with precedence: || > && > == != > < <= > >= > + - > * / % > unary > postfix
    let mul_expr = unary_expr
        .clone()
        .then(
            mul_op
                .then(unary_expr.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .try_map(|(mut left, ops), span| {
            for (op, right) in ops {
                let left_span = left.span().map(|s| s.start).unwrap_or(span.start);
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span: Some(crate::ast::Span::new(left_span, span.end)),
                };
            }
            Ok(left)
        })
        .boxed();

    let add_expr = mul_expr
        .clone()
        .then(add_op.then(mul_expr.clone()).repeated().collect::<Vec<_>>())
        .try_map(|(mut left, ops), span| {
            for (op, right) in ops {
                let left_span = left.span().map(|s| s.start).unwrap_or(span.start);
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span: Some(crate::ast::Span::new(left_span, span.end)),
                };
            }
            Ok(left)
        })
        .boxed();

    let cmp_expr = add_expr
        .clone()
        .then(cmp_op.then(add_expr.clone()).repeated().collect::<Vec<_>>())
        .try_map(|(mut left, ops), span| {
            for (op, right) in ops {
                let left_span = left.span().map(|s| s.start).unwrap_or(span.start);
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span: Some(crate::ast::Span::new(left_span, span.end)),
                };
            }
            Ok(left)
        })
        .boxed();

    let eq_expr = cmp_expr
        .clone()
        .then(eq_op.then(cmp_expr.clone()).repeated().collect::<Vec<_>>())
        .try_map(|(mut left, ops), span| {
            for (op, right) in ops {
                let left_span = left.span().map(|s| s.start).unwrap_or(span.start);
                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span: Some(crate::ast::Span::new(left_span, span.end)),
                };
            }
            Ok(left)
        })
        .boxed();

    let and_expr = eq_expr
        .clone()
        .then(and_op.then(eq_expr.clone()).repeated().collect::<Vec<_>>())
        .try_map(|(mut left, ops), span| {
            for (op, right) in ops {
                let left_span = left.span().map(|s| s.start).unwrap_or(span.start);
                left = Expr::Logical {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span: Some(crate::ast::Span::new(left_span, span.end)),
                };
            }
            Ok(left)
        })
        .boxed();

    let or_expr = and_expr
        .clone()
        .then(or_op.then(and_expr.clone()).repeated().collect::<Vec<_>>())
        .try_map(|(mut left, ops), span| {
            for (op, right) in ops {
                let left_span = left.span().map(|s| s.start).unwrap_or(span.start);
                left = Expr::Logical {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span: Some(crate::ast::Span::new(left_span, span.end)),
                };
            }
            Ok(left)
        })
        .boxed();

    expr_ref.define(or_expr);

    // Statement parsers

    let var_decl = statements::var_decl(
        ws.clone(),
        pattern.clone(),
        type_parser.clone(),
        expr_ref.clone(),
    );
    let return_stmt = statements::return_stmt(ws.clone(), expr_ref.clone());
    let if_stmt = statements::if_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone());
    let while_stmt = statements::while_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone());
    let do_while_stmt = statements::do_while_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone());
    let for_stmt = statements::for_stmt(
        ws.clone(),
        pattern.clone(),
        expr_ref.clone(),
        stmt_ref.clone(),
    );
    let break_stmt = statements::break_stmt(ws.clone());
    let continue_stmt = statements::continue_stmt(ws.clone());
    let assignment = statements::assignment(ws.clone(), expr_ref.clone());
    let expr_stmt = statements::expr_stmt(expr_ref.clone());

    let match_stmt = statements::match_stmt(
        ws.clone(),
        expr_ref.clone(),
        stmt_ref.clone(),
        pattern.clone(),
    );

    let stmt = choice((
        match_stmt,
        return_stmt,
        break_stmt,
        continue_stmt,
        var_decl,
        if_stmt,
        do_while_stmt, // Must come before while_stmt to avoid ambiguity with "do"
        while_stmt,
        for_stmt,
        assignment,
        expr_stmt,
    ))
    .boxed();

    // For nested statements (inside blocks), don't use recovery
    // to preserve block structure integrity
    stmt_ref.define(stmt.clone());

    // At the top level, add error recovery to statement parsing.
    // When parsing fails, skip to the next statement boundary and continue.
    let stmt_with_recovery = stmt
        .clone()
        .recover_with(via_parser(recovery::statement_recovery()))
        .boxed();

    // Program: statements with optional trailing expression -> Return
    ws.clone().ignore_then(
        stmt_with_recovery
            .clone()
            .repeated()
            .collect::<Vec<Stmt>>()
            .then(expr_ref.clone().or_not())
            .then_ignore(ws.clone())
            .then_ignore(end())
            .map(|(statements, ret)| Program {
                statements: utils::apply_implicit_return(statements, ret),
            }),
    )
}

pub fn parse(source: &str, filename: &str) -> Result<Program, Vec<Diagnostic>> {
    let (output, errs) = parser().parse(source).into_output_errors();

    if errs.is_empty() {
        // No errors, return the parsed program
        Ok(output.expect("Parser should produce output when no errors"))
    } else {
        // Errors occurred - return all accumulated errors
        Err(errors::errors_to_diagnostics(errs, filename, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr, LogicalOp, Stmt, UnaryOp};

    fn parse_expr(source: &str) -> Expr {
        let program = parse(source, "test.luma").expect("Parse failed");
        // The parser converts trailing expressions into Return statements
        match program.statements.last() {
            Some(Stmt::Return { value, .. }) => value.clone(),
            Some(Stmt::ExprStmt { expr, .. }) => expr.clone(),
            _ => panic!("Expected expression in program"),
        }
    }

    fn parse_stmt(source: &str) -> Stmt {
        let program = parse(source, "test.luma").expect("Parse failed");
        program.statements[0].clone()
    }

    // ===== Literal Tests =====

    #[test]
    fn test_parse_number_literal() {
        let expr = parse_expr("42");
        assert!(matches!(expr, Expr::Number { value, .. } if (value - 42.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_parse_float_literal() {
        let expr = parse_expr("3.14159");
        assert!(
            matches!(expr, Expr::Number { value, .. } if (value - std::f64::consts::PI).abs() < 0.00001)
        );
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = parse_expr(r#""hello world""#);
        assert!(matches!(expr, Expr::String { value, .. } if value == "hello world"));
    }

    #[test]
    fn test_parse_boolean_true() {
        let expr = parse_expr("true");
        assert!(matches!(expr, Expr::Boolean { value: true, .. }));
    }

    #[test]
    fn test_parse_boolean_false() {
        let expr = parse_expr("false");
        assert!(matches!(expr, Expr::Boolean { value: false, .. }));
    }

    #[test]
    fn test_parse_null() {
        let expr = parse_expr("null");
        assert!(matches!(expr, Expr::Null { .. }));
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_expr("myVar");
        assert!(matches!(expr, Expr::Identifier { name, .. } if name == "myVar"));
    }

    // ===== Binary Operator Tests =====

    #[test]
    fn test_parse_addition() {
        let expr = parse_expr("1 + 2");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_subtraction() {
        let expr = parse_expr("5 - 3");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Sub,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_multiplication() {
        let expr = parse_expr("4 * 5");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_division() {
        let expr = parse_expr("10 / 2");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Div,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_modulo() {
        let expr = parse_expr("10 % 3");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Mod,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_operator_precedence_mul_before_add() {
        let expr = parse_expr("1 + 2 * 3");
        match expr {
            Expr::Binary {
                left,
                op: BinaryOp::Add,
                right,
                ..
            } => {
                assert!(
                    matches!(*left, Expr::Number { value, .. } if (value - 1.0).abs() < f64::EPSILON)
                );
                assert!(matches!(
                    *right,
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            }
            _ => panic!("Expected addition with multiplication on right"),
        }
    }

    #[test]
    fn test_parse_operator_precedence_parentheses() {
        let expr = parse_expr("(1 + 2) * 3");
        match expr {
            Expr::Binary {
                left,
                op: BinaryOp::Mul,
                right,
                ..
            } => {
                assert!(matches!(
                    *left,
                    Expr::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
                assert!(
                    matches!(*right, Expr::Number { value, .. } if (value - 3.0).abs() < f64::EPSILON)
                );
            }
            _ => panic!("Expected multiplication with addition on left"),
        }
    }

    // ===== Comparison Operator Tests =====

    #[test]
    fn test_parse_less_than() {
        let expr = parse_expr("x < 10");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Lt,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_greater_than() {
        let expr = parse_expr("x > 10");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Gt,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_less_or_equal() {
        let expr = parse_expr("x <= 10");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Le,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_greater_or_equal() {
        let expr = parse_expr("x >= 10");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Ge,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_equality() {
        let expr = parse_expr("x == 10");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Eq,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_inequality() {
        let expr = parse_expr("x != 10");
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Ne,
                ..
            }
        ));
    }

    // ===== Unary Operator Tests =====

    #[test]
    fn test_parse_unary_negation() {
        let expr = parse_expr("-42");
        assert!(matches!(
            expr,
            Expr::Unary {
                op: UnaryOp::Neg,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_unary_not() {
        let expr = parse_expr("!true");
        assert!(matches!(
            expr,
            Expr::Unary {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_double_negation() {
        // Double negation with parentheses (-- would be a comment)
        let expr = parse_expr("-(-5)");
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg,
                operand,
                ..
            } => {
                assert!(matches!(
                    *operand,
                    Expr::Unary {
                        op: UnaryOp::Neg,
                        ..
                    }
                ));
            }
            _ => panic!("Expected double negation"),
        }
    }

    #[test]
    fn test_parse_unary_not_function_call() {
        // Test that !fn() parses as !(fn()), not (!fn)()
        let expr = parse_expr("!foo()");
        match expr {
            Expr::Unary {
                op: UnaryOp::Not,
                operand,
                ..
            } => {
                // The operand should be a function call
                assert!(matches!(*operand, Expr::Call { .. }));
            }
            _ => panic!("Expected unary not with function call as operand"),
        }
    }

    #[test]
    fn test_parse_negation_function_call() {
        // Test that -fn() parses as -(fn()), not (-fn)()
        let expr = parse_expr("-foo()");
        match expr {
            Expr::Unary {
                op: UnaryOp::Neg,
                operand,
                ..
            } => {
                // The operand should be a function call
                assert!(matches!(*operand, Expr::Call { .. }));
            }
            _ => panic!("Expected negation with function call as operand"),
        }
    }

    // ===== Logical Operator Tests =====

    #[test]
    fn test_parse_logical_and() {
        let expr = parse_expr("true && false");
        assert!(matches!(
            expr,
            Expr::Logical {
                op: LogicalOp::And,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_logical_or() {
        let expr = parse_expr("true || false");
        assert!(matches!(
            expr,
            Expr::Logical {
                op: LogicalOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_logical_precedence() {
        let expr = parse_expr("a || b && c");
        match expr {
            Expr::Logical {
                left,
                op: LogicalOp::Or,
                right,
                ..
            } => {
                assert!(matches!(*left, Expr::Identifier { .. }));
                assert!(matches!(
                    *right,
                    Expr::Logical {
                        op: LogicalOp::And,
                        ..
                    }
                ));
            }
            _ => panic!("Expected || with && on right"),
        }
    }

    // ===== List Tests =====

    #[test]
    fn test_parse_empty_list() {
        let expr = parse_expr("[]");
        assert!(matches!(expr, Expr::List { elements, .. } if elements.is_empty()));
    }

    #[test]
    fn test_parse_list_with_elements() {
        let expr = parse_expr("[1, 2, 3]");
        assert!(matches!(expr, Expr::List { elements, .. } if elements.len() == 3));
    }

    #[test]
    fn test_parse_nested_lists() {
        let expr = parse_expr("[[1, 2], [3, 4]]");
        match expr {
            Expr::List { elements, .. } => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], Expr::List { .. }));
                assert!(matches!(elements[1], Expr::List { .. }));
            }
            _ => panic!("Expected list"),
        }
    }

    // ===== Table Tests =====

    #[test]
    fn test_parse_empty_table() {
        let expr = parse_expr("{}");
        assert!(matches!(expr, Expr::Table { fields, .. } if fields.is_empty()));
    }

    #[test]
    fn test_parse_table_with_identifier_keys() {
        let expr = parse_expr("{x = 1, y = 2}");
        match expr {
            Expr::Table { fields, .. } => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected table"),
        }
    }

    // ===== Function Call Tests =====

    #[test]
    fn test_parse_function_call_no_args() {
        let expr = parse_expr("foo()");
        assert!(matches!(expr, Expr::Call { arguments, .. } if arguments.is_empty()));
    }

    #[test]
    fn test_parse_function_call_with_args() {
        let expr = parse_expr("add(1, 2)");
        assert!(matches!(expr, Expr::Call { arguments, .. } if arguments.len() == 2));
    }

    #[test]
    fn test_parse_function_call_named_args() {
        let expr = parse_expr("func(x=1, y=2)");
        match expr {
            Expr::Call { arguments, .. } => {
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], CallArgument::Named { .. }));
                assert!(matches!(arguments[1], CallArgument::Named { .. }));
            }
            _ => panic!("Expected call with named arguments"),
        }
    }

    #[test]
    fn test_parse_function_call_mixed_args() {
        let expr = parse_expr("func(1, y=2)");
        match expr {
            Expr::Call { arguments, .. } => {
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], CallArgument::Positional(_)));
                assert!(matches!(arguments[1], CallArgument::Named { .. }));
            }
            _ => panic!("Expected call with mixed arguments"),
        }
    }

    // ===== Member Access Tests =====

    #[test]
    fn test_parse_member_access() {
        let expr = parse_expr("obj.field");
        assert!(matches!(expr, Expr::MemberAccess { member, .. } if member == "field"));
    }

    #[test]
    fn test_parse_chained_member_access() {
        let expr = parse_expr("obj.field.subfield");
        match expr {
            Expr::MemberAccess { object, member, .. } => {
                assert_eq!(member, "subfield");
                assert!(matches!(*object, Expr::MemberAccess { .. }));
            }
            _ => panic!("Expected chained member access"),
        }
    }

    // ===== Index Access Tests =====

    #[test]
    fn test_parse_index_access() {
        let expr = parse_expr("arr[0]");
        assert!(matches!(expr, Expr::Index { .. }));
    }

    #[test]
    fn test_parse_nested_index_access() {
        let expr = parse_expr("arr[0][1]");
        match expr {
            Expr::Index { object, .. } => {
                assert!(matches!(*object, Expr::Index { .. }));
            }
            _ => panic!("Expected nested index access"),
        }
    }

    // ===== Variable Declaration Tests =====

    #[test]
    fn test_parse_var_decl_immutable() {
        let stmt = parse_stmt("let x = 5");
        assert!(matches!(stmt, Stmt::VarDecl { mutable: false, .. }));
    }

    #[test]
    fn test_parse_var_decl_mutable() {
        let stmt = parse_stmt("var x = 5");
        assert!(matches!(stmt, Stmt::VarDecl { mutable: true, .. }));
    }

    #[test]
    fn test_parse_var_decl_with_type() {
        let stmt = parse_stmt("let x: Number = 5");
        match stmt {
            Stmt::VarDecl { name, r#type, .. } => {
                assert_eq!(name, "x");
                assert!(r#type.is_some());
            }
            _ => panic!("Expected var decl"),
        }
    }

    // ===== If Statement Tests =====

    #[test]
    fn test_parse_if_stmt() {
        let stmt = parse_stmt("if x > 5 do return x end");
        assert!(matches!(stmt, Stmt::If { .. }));
    }

    #[test]
    fn test_parse_if_else_stmt() {
        let stmt = parse_stmt("if x > 5 do x else do 0 end");
        match stmt {
            Stmt::If { else_block, .. } => {
                assert!(else_block.is_some());
            }
            _ => panic!("Expected if with else"),
        }
    }

    // ===== While Loop Tests =====

    #[test]
    fn test_parse_while_loop() {
        let stmt = parse_stmt("while x < 10 do x = x + 1 end");
        assert!(matches!(stmt, Stmt::While { .. }));
    }

    #[test]
    fn test_parse_do_while_loop() {
        let stmt = parse_stmt("do x = x + 1 while x < 10 end");
        assert!(matches!(stmt, Stmt::DoWhile { .. }));
    }

    // ===== For Loop Tests =====

    #[test]
    fn test_parse_for_loop() {
        let stmt = parse_stmt("for x in list do print(x) end");
        assert!(matches!(stmt, Stmt::For { .. }));
    }

    // ===== Function Definition Tests =====

    #[test]
    fn test_parse_function_no_args() {
        let expr = parse_expr("fn() do return 42 end");
        match expr {
            Expr::Function { arguments, .. } => {
                assert!(arguments.is_empty());
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_function_with_args() {
        let expr = parse_expr("fn(x: Number, y: Number): Number do return x + y end");
        match expr {
            Expr::Function {
                arguments,
                return_type,
                ..
            } => {
                assert_eq!(arguments.len(), 2);
                assert!(return_type.is_some());
            }
            _ => panic!("Expected function with arguments"),
        }
    }

    // ===== Return Statement Tests =====

    #[test]
    fn test_parse_return_stmt() {
        let stmt = parse_stmt("return 42");
        assert!(matches!(stmt, Stmt::Return { .. }));
    }

    // ===== Break/Continue Tests =====

    #[test]
    fn test_parse_break_stmt() {
        let stmt = parse_stmt("break");
        assert!(matches!(stmt, Stmt::Break { level: None, .. }));
    }

    #[test]
    fn test_parse_break_with_level() {
        let stmt = parse_stmt("break 2");
        assert!(matches!(stmt, Stmt::Break { level: Some(2), .. }));
    }

    #[test]
    fn test_parse_continue_stmt() {
        let stmt = parse_stmt("continue");
        assert!(matches!(stmt, Stmt::Continue { level: None, .. }));
    }

    #[test]
    fn test_parse_continue_with_level() {
        let stmt = parse_stmt("continue 3");
        assert!(matches!(stmt, Stmt::Continue { level: Some(3), .. }));
    }

    // ===== Assignment Tests =====

    #[test]
    fn test_parse_assignment() {
        let stmt = parse_stmt("x = 10");
        assert!(matches!(stmt, Stmt::Assignment { .. }));
    }

    // ===== Error Tests =====

    #[test]
    fn test_parse_error_invalid_syntax() {
        let result = parse("let x = ", "test.luma");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_keyword_as_identifier() {
        let result = parse("let if = 5", "test.luma");
        assert!(result.is_err());
    }

    // ===== Complex Expression Tests =====

    #[test]
    fn test_parse_complex_nested_expression() {
        let source = "((1 + 2) * 3 - 4) / 5";
        let result = parse(source, "test.luma");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call() {
        let expr = parse_expr("obj:method(1, 2)");
        assert!(matches!(expr, Expr::MethodCall { .. }));
    }

    #[test]
    fn test_parse_import() {
        let expr = parse_expr("import(\"./module.luma\")");
        assert!(matches!(expr, Expr::Import { .. }));
    }

    #[test]
    fn test_parse_unexpected_eof_suggestion() {
        let source = "if true do";
        let result = parse(source, "test.luma");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        // Expect at least one diagnostic with either a suggestion or a fix-it
        let has_suggestion = diags
            .iter()
            .any(|d| !d.suggestions.is_empty() || !d.fixits.is_empty());
        assert!(has_suggestion);
    }

    #[test]
    fn test_parse_missing_paren_fixit() {
        let source = "let x = (1 + 2";
        let result = parse(source, "test.luma");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        let has_paren_insert = diags
            .iter()
            .any(|d| d.fixits.iter().any(|f| f.replacement().contains(')')));
        assert!(has_paren_insert, "expected a fix-it to insert ')'");
    }

    #[test]
    fn test_parse_extra_closer_delete_fixit() {
        let source = "let x = 1 + 2))";
        let result = parse(source, "test.luma");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        let has_delete = diags
            .iter()
            .any(|d| d.fixits.iter().any(|f| f.replacement().is_empty()));
        assert!(has_delete, "expected a fix-it to remove an extra closer");
    }
}
