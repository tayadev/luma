/// Creates a parser for match statements
pub fn match_stmt<'a, WS, E, S, P>(
    ws: WS,
    expr: E,
    stmt: S,
    pattern: P,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
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
            (
                pattern
                    .then_ignore(ws.clone())
                    .then_ignore(just("do").padded_by(ws.clone()))
                    .then(stmt.repeated().collect::<Vec<Stmt>>())
                    .then_ignore(just("end").padded_by(ws.clone()))
            )
            .repeated()
            .collect::<Vec<(Pattern, Vec<Stmt>)>>()
        )
        .then_ignore(just("end").padded_by(ws))
        .map(|(expr, arms)| Stmt::Match { expr, arms, span: None })
        .boxed()
}
use chumsky::prelude::*;
use crate::ast::{Stmt, Expr, Pattern, Type, Span};
use crate::parser::operators;
use super::utils::apply_implicit_return_stmts;

/// Creates a parser for return statements
pub fn return_stmt<'a, WS, E>(ws: WS, expr: E) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("return")
        .padded_by(ws)
        .ignore_then(expr)
        .try_map(|value, span| Ok(Stmt::Return { value, span: Some(Span::from_chumsky(span)) }))
        .boxed()
}

/// Creates a parser for variable declarations (let/var)
pub fn var_decl<'a, WS, P, T, E>(
    ws: WS,
    pattern: P,
    type_parser: T,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    P: Parser<'a, &'a str, Pattern, extra::Err<Rich<'a, char>>> + Clone + 'a,
    T: Parser<'a, &'a str, Type, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let var_decl_token = choice((just("let").to(false), just("var").to(true)))
        .padded_by(ws.clone())
        .then(choice((
            pattern.map(|p| match p {
                Pattern::Ident(name) => (None, Some(name)),
                _ => (Some(p), None),
            }),
        )))
        .then(just(':').padded_by(ws.clone()).ignore_then(type_parser).or_not())
        .then_ignore(just('=').padded_by(ws));

    var_decl_token
        .then(expr)
        .try_map(|(((mutable, (pattern, name)), opt_type), value), span| {
            Ok(if let Some(pattern) = pattern {
                Stmt::DestructuringVarDecl { mutable, pattern, value, span: Some(Span::from_chumsky(span)) }
            } else {
                Stmt::VarDecl { mutable, name: name.unwrap(), r#type: opt_type, value, span: Some(Span::from_chumsky(span)) }
            })
        })
        .boxed()
}

/// Creates a parser for assignment statements (x = value, x += value, etc.)
pub fn assignment<'a, WS, E>(
    ws: WS,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let assign_op = operators::assign_op(ws.clone());
    
    expr.clone()
        .then(assign_op)
        .then(expr)
        .try_map(|((target, op), value), span| Ok(Stmt::Assignment { target, op, value, span: Some(Span::from_chumsky(span)) }))
        .boxed()
}

/// Creates a parser for if/elif/else statements
pub fn if_stmt<'a, WS, E, S>(
    ws: WS,
    expr: E,
    stmt: S,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let elif_block = just("else")
        .padded_by(ws.clone())
        .then_ignore(just("if").padded_by(ws.clone()))
        .ignore_then(expr.clone())
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(stmt.clone().repeated().collect::<Vec<Stmt>>())
        .map(|(cond, body)| (cond, apply_implicit_return_stmts(body)));
    
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
        .then(elif_block.repeated().collect::<Vec<(Expr, Vec<Stmt>)>>())
        .then(else_block.or_not())
        .then_ignore(just("end").padded_by(ws))
        .try_map(|(((condition, then_block), elif_blocks), else_block), span| {
            Ok(Stmt::If { 
                condition, 
                then_block: apply_implicit_return_stmts(then_block), 
                elif_blocks, 
                else_block,
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}

/// Creates a parser for while loop statements
pub fn while_stmt<'a, WS, E, S>(
    ws: WS,
    expr: E,
    stmt: S,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("while")
        .padded_by(ws.clone())
        .ignore_then(expr)
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(stmt.repeated().collect::<Vec<Stmt>>())
        .then_ignore(just("end").padded_by(ws))
        .try_map(|(condition, body), span| Ok(Stmt::While { condition, body, span: Some(Span::from_chumsky(span)) }))
        .boxed()
}

/// Creates a parser for do-while loop statements
pub fn do_while_stmt<'a, WS, E, S>(
    ws: WS,
    expr: E,
    stmt: S,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("do")
        .padded_by(ws.clone())
        .ignore_then(stmt.repeated().collect::<Vec<Stmt>>())
        .then_ignore(just("while").padded_by(ws.clone()))
        .then(expr)
        .then_ignore(just("end").padded_by(ws))
        .try_map(|(body, condition), span| Ok(Stmt::DoWhile { body, condition, span: Some(Span::from_chumsky(span)) }))
        .boxed()
}

/// Creates a parser for for loop statements
pub fn for_stmt<'a, WS, P, E, S>(
    ws: WS,
    pattern: P,
    expr: E,
    stmt: S,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    P: Parser<'a, &'a str, Pattern, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("for")
        .padded_by(ws.clone())
        .ignore_then(pattern)
        .then_ignore(just("in").padded_by(ws.clone()))
        .then(expr)
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(stmt.repeated().collect::<Vec<Stmt>>())
        .then_ignore(just("end").padded_by(ws))
        .try_map(|((pattern, iterator), body), span| Ok(Stmt::For { pattern, iterator, body, span: Some(Span::from_chumsky(span)) }))
        .boxed()
}

/// Creates a parser for break statements
pub fn break_stmt<'a, WS>(ws: WS) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("break")
        .padded_by(ws.clone())
        .then(
            text::int(10)
                .padded_by(ws.clone())
                .try_map(|s: &str, span| {
                    s.parse::<u32>()
                        .map_err(|e| Rich::custom(span, format!("Invalid break level: {}", e)))
                })
                .or_not()
        )
        .map(|(_, level)| Stmt::Break(level))
        .boxed()
}

/// Creates a parser for continue statements
pub fn continue_stmt<'a, WS>(ws: WS) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("continue")
        .padded_by(ws.clone())
        .then(
            text::int(10)
                .padded_by(ws.clone())
                .try_map(|s: &str, span| {
                    s.parse::<u32>()
                        .map_err(|e| Rich::custom(span, format!("Invalid continue level: {}", e)))
                })
                .or_not()
        )
        .map(|(_, level)| Stmt::Continue(level))
        .boxed()
}

/// Creates a parser for expression statements
pub fn expr_stmt<'a, E>(expr: E) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    expr.map(Stmt::ExprStmt).boxed()
}
