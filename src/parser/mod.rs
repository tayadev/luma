use chumsky::prelude::*;
use crate::ast::{Program, Stmt, Expr, Type};

mod string;
mod lexer;
mod operators;
mod literals;
mod patterns;
mod statements;
mod expressions;

use string::string_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Program, extra::Err<Rich<'a, char>>> {
    // Comments and whitespace
    let ws = lexer::ws();
    
    let ident = text::ident()
        .try_map(move |s: &str, span| {
            if lexer::KEYWORDS.contains(&s) {
                Err(Rich::custom(span, format!("'{}' is a keyword and cannot be used as an identifier", s)))
            } else {
                Ok(s)
            }
        })
        .padded_by(ws.clone());
    let type_parser = ident.clone().map(|s: &str| Type::TypeIdent(s.to_string()));
    let number = text::int(10).padded_by(ws.clone()).map(|s: &str| s.parse::<f64>().unwrap());

    // Recursive expression placeholder
    let mut expr_ref = Recursive::declare();
    let mut stmt_ref = Recursive::declare();

    // Boolean and Null literals
    let boolean = literals::boolean(ws.clone());
    let null = literals::null(ws.clone());

    // Array and Table literals
    let array = literals::array(ws.clone(), expr_ref.clone());
    let table = literals::table(ws.clone(), ident.clone(), expr_ref.clone());

    // Pattern parsing for destructuring
    let pattern = patterns::pattern(ws.clone(), ident.clone());

    // Statement parsers
    let var_decl = statements::var_decl(ws.clone(), pattern, type_parser.clone(), expr_ref.clone());
    let return_stmt = statements::return_stmt(ws.clone(), expr_ref.clone());
    
    let stmt = choice((return_stmt, var_decl)).boxed();
    stmt_ref.define(stmt.clone());

    // Expression parsers (blocks and functions)
    let block_expr = expressions::block(ws.clone(), stmt_ref.clone(), expr_ref.clone());
    let function = expressions::function(ws.clone(), ident.clone(), type_parser.clone(), stmt_ref.clone(), expr_ref.clone());

    // Primary expressions (atoms)
    let primary = choice((
        number.map(Expr::Number).boxed(),
        string_parser(ws.clone()).boxed(),
        boolean,
        null,
        array,
        table,
        block_expr,
        function,
        ident.map(|s: &str| Expr::Identifier(s.to_string())).boxed(),
    )).boxed();

    // Binary operators with precedence
    let mul_op = operators::mul_op(ws.clone());
    let add_op = operators::add_op(ws.clone());
    let cmp_op = operators::cmp_op(ws.clone());

    // Build expression with precedence: comparison > addition > multiplication
    let mul_expr = primary.clone()
        .foldl(mul_op.then(primary.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();
    
    let add_expr = mul_expr.clone()
        .foldl(add_op.then(mul_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();
    
    let cmp_expr = add_expr.clone()
        .foldl(cmp_op.then(add_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();

    expr_ref.define(cmp_expr);

    // Program: statements with optional trailing expression -> Return
    ws.clone().ignore_then(
        stmt.clone()
            .repeated()
            .collect::<Vec<Stmt>>()
            .then(expr_ref.clone().or_not())
            .then_ignore(ws.clone())
            .then_ignore(end())
            .map(|(mut statements, ret)| { if let Some(expr) = ret { statements.push(Stmt::Return(expr)); } Program { statements } })
    )
}

pub fn parse(source: &str) -> Result<Program, Vec<Rich<'_, char>>> {
    parser().parse(source).into_result()
}
