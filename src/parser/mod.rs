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

    // Unary operators (not, -)
    let unary_op = operators::unary_op(ws.clone());
    let unary_expr = unary_op
        .repeated()
        .collect::<Vec<_>>()
        .then(primary.clone())
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

    let postfix = unary_expr
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

    // Binary operators with precedence
    let mul_op = operators::mul_op(ws.clone());
    let add_op = operators::add_op(ws.clone());
    let cmp_op = operators::cmp_op(ws.clone());
    let eq_op = operators::eq_op(ws.clone());
    let and_op = operators::and_op(ws.clone());
    let or_op = operators::or_op(ws.clone());

    // Build expression with precedence: || > && > == != > < <= > >= > + - > * / % > postfix > unary
    let mul_expr = postfix
        .clone()
        .then(mul_op.then(postfix.clone()).repeated().collect::<Vec<_>>())
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
        Err(errors::errors_to_diagnostics(errs, filename))
    }
}
