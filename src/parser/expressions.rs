use super::utils::{apply_implicit_return, apply_implicit_return_stmts};
use crate::ast::{Argument, Expr, Pattern, Span, Stmt, Type};
use chumsky::prelude::*;

/// Creates a parser for block expressions (do...end)
pub fn block<'a, WS, S, E>(
    ws: WS,
    stmt: S,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("do")
        .padded_by(ws.clone())
        .ignore_then(stmt.repeated().collect::<Vec<Stmt>>())
        .then(expr.or_not())
        .then_ignore(just("end").padded_by(ws))
        .try_map(|(stmts, ret), span| {
            Ok(Expr::Block {
                statements: apply_implicit_return(stmts, ret),
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}

/// Creates a parser for if expressions (if...do...else...end)
pub fn if_expr<'a, WS, S, E>(
    ws: WS,
    stmt: S,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let else_block = just("else")
        .padded_by(ws.clone())
        .then_ignore(just("do").padded_by(ws.clone()))
        .ignore_then(stmt.clone().repeated().collect::<Vec<Stmt>>())
        .map(apply_implicit_return_stmts);

    just("if")
        .padded_by(ws.clone())
        .ignore_then(expr)
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(stmt.repeated().collect::<Vec<Stmt>>())
        .then(else_block.or_not())
        .then_ignore(just("end").padded_by(ws))
        .try_map(|((condition, then_block), else_block), span| {
            Ok(Expr::If {
                condition: Box::new(condition),
                then_block: apply_implicit_return_stmts(then_block),
                else_block,
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}

/// Creates a parser for function expressions
pub fn function<'a, WS, I, T, S, E>(
    ws: WS,
    ident: I,
    type_parser: T,
    stmt: S,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
    T: Parser<'a, &'a str, Type, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    // Argument parsing with default values
    let argument = ident
        .clone()
        .then_ignore(just(':').padded_by(ws.clone()))
        .then(type_parser.clone())
        .then(
            just('=')
                .padded_by(ws.clone())
                .ignore_then(expr.clone())
                .or_not(),
        )
        .map(
            |((name, t), default): ((&str, Type), Option<Expr>)| Argument {
                name: name.to_string(),
                r#type: t,
                default,
                span: None,
            },
        );

    let arg_list = argument
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<Argument>>()
        .delimited_by(
            just('(').padded_by(ws.clone()),
            just(')').padded_by(ws.clone()),
        );

    // Function body: statements + optional trailing expression as Return
    let body_block = stmt
        .repeated()
        .collect::<Vec<Stmt>>()
        .then(expr.or_not())
        .then_ignore(just("end").padded_by(ws.clone()))
        .map(|(stmts, ret)| apply_implicit_return(stmts, ret));

    just("fn")
        .padded_by(ws.clone())
        .ignore_then(arg_list)
        .then(
            just(':')
                .padded_by(ws.clone())
                .ignore_then(type_parser)
                .or_not(),
        )
        .then_ignore(just("do").padded_by(ws))
        .then(body_block)
        .try_map(
            |((arguments, return_type), body): ((Vec<Argument>, Option<Type>), Vec<Stmt>), span| {
                Ok(Expr::Function {
                    arguments,
                    return_type,
                    body,
                    span: Some(Span::from_chumsky(span)),
                })
            },
        )
        .boxed()
}

/// Creates a parser for match expressions (match expr do pattern do ... end ... end)
pub fn match_expr<'a, WS, E, S, P>(
    ws: WS,
    expr: E,
    stmt: S,
    pattern: P,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    P: Parser<'a, &'a str, Pattern, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("match")
        .padded_by(ws.clone())
        .ignore_then(expr)
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(
            (pattern
                .then_ignore(ws.clone())
                .then_ignore(just("do").padded_by(ws.clone()))
                .then(stmt.repeated().collect::<Vec<Stmt>>())
                .then_ignore(just("end").padded_by(ws.clone())))
            .repeated()
            .collect::<Vec<(Pattern, Vec<Stmt>)>>(),
        )
        .then_ignore(just("end").padded_by(ws))
        .try_map(|(expr, arms), span| {
            Ok(Expr::Match {
                expr: Box::new(expr),
                arms,
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}

/// Creates a parser for import expressions (import("path"))
pub fn import<'a, WS, E>(
    ws: WS,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("import")
        .padded_by(ws.clone())
        .ignore_then(expr.delimited_by(just('(').padded_by(ws.clone()), just(')').padded_by(ws)))
        .try_map(|path, span| {
            Ok(Expr::Import {
                path: Box::new(path),
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}
